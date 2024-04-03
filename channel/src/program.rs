use std::cell::RefMut;

use crate::error::ChannelError;
use crate::proof_handling::{prepare_inputs, verify_risc0};
use crate::{assertions::*, deployment_address_seeds};
use crate::{execution_address_seeds, execution_claim_address_seeds, img_id_hash};
use anagram_bonsol_schema::{
    parse_ix_data, root_as_deploy_v1, root_as_execution_request_v1, root_as_input_set,
    ChannelInstructionIxType, ClaimV1, DeployV1, ExecutionRequestV1, ExitCode, InputType,
    StatusTypes, StatusV1,
};
use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::{sol_memcmp, sol_memcpy, sol_memset};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{bpf_loader_upgradeable, msg, system_instruction, system_program};

pub struct ClaimAccounts<'a, 'b> {
    pub exec: &'a AccountInfo<'a>,
    pub exec_claim: &'a AccountInfo<'a>,
    pub claimer: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub execution_id: &'b str,
    pub block_commitment: u64,
    pub existing_claim: bool,
    pub stake: u64,
}

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct Claim {
    pub claimer: [u8; 32],
    pub claimed_at: u64,
    pub block_commitment: u64,
}

impl Claim {
    pub fn load_claim<'a>(ca_data: &'a mut [u8]) -> Result<&'a Self, ChannelError> {
        bytemuck::try_from_bytes::<Claim>(ca_data).map_err(|_| ChannelError::InvalidClaimAccount)
    }

    pub fn from_claim_ix(claimer: &Pubkey, slot: u64, block_commitment: u64) -> Self {
        Claim {
            claimer: claimer.to_bytes(),
            claimed_at: slot,
            block_commitment,
        }
    }

    pub fn save_claim(claim: &Claim, ca: &AccountInfo) {
        let claim_data = bytemuck::bytes_of(claim);
        sol_memcpy(&mut ca.data.borrow_mut(), &claim_data, claim_data.len());
    }
}

impl<'a, 'b> ClaimAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ClaimV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let mut ca = ClaimAccounts {
                exec: &accounts[0],
                exec_claim: &accounts[1],
                claimer: &accounts[2],
                payer: &accounts[3],
                system_program: &accounts[4],
                execution_id: executionid,
                block_commitment: data.block_commitment(),
                existing_claim: false,
                stake: 0,
            };
            check_writable_signer(ca.payer, ChannelError::InvalidPayerAccount)?;
            check_writable_signer(ca.claimer, ChannelError::InvalidClaimerAccount)?;
            check_writeable(ca.exec_claim, ChannelError::InvalidClaimAccount)?;
            check_owner(ca.exec, &crate::ID, ChannelError::InvalidExecutionAccount)?;
            let exec_data = ca
                .exec
                .try_borrow_data()
                .map_err(|_| ChannelError::CannotBorrowData)?;
            msg!("here");
            let execution_request = root_as_execution_request_v1(&*exec_data)
                .map_err(|_| ChannelError::InvalidExecutionAccount)?;
            msg!("here");
            let expected_eid = execution_request
                .execution_id()
                .ok_or(ChannelError::InvalidExecutionAccount)?;
            if expected_eid != executionid {
                return Err(ChannelError::InvalidExecutionAccount);
            }
            let tip = execution_request.tip();
            if ca.claimer.lamports() < tip {
                return Err(ChannelError::InsufficientStake.into());
            }
            ca.stake = tip / 2;
            let mut exec_claim_seeds = execution_claim_address_seeds(executionid.as_bytes());
            let bump = [check_pda(
                &exec_claim_seeds,
                ca.exec_claim.key,
                ChannelError::InvalidClaimAccount,
            )?];
            exec_claim_seeds.push(&bump);
            if ca.exec_claim.data_len() == 0 && ca.exec_claim.owner == &system_program::ID {
                create_program_account(
                    ca.exec_claim,
                    &exec_claim_seeds,
                    std::mem::size_of::<Claim>() as u64,
                    ca.payer,
                    ca.system_program,
                    None,
                )?;
            } else {
                check_owner(ca.exec_claim, &crate::ID, ChannelError::InvalidClaimAccount)?;
                ca.existing_claim = true;
            }
            return Ok(ca);
        }

        Err(ChannelError::InvalidInstruction)
    }
}

pub struct DeployAccounts<'a, 'b> {
    pub deployer: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub deployment: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub deployment_bump: Option<u8>,
    pub image_id: &'b str,
}

impl<'a, 'b> DeployAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b DeployV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(imageid) = data.image_id() {
            let mut da = DeployAccounts {
                deployer: &accounts[0],
                payer: &accounts[1],
                deployment: &accounts[2],
                system_program: &accounts[3],
                extra_accounts: &accounts[4..],
                deployment_bump: None,
                image_id: imageid,
            };
            let owner = data
                .owner()
                .map(|b| b.bytes())
                .ok_or(ChannelError::InvalidInstruction)?;
            check_writable_signer(da.payer, ChannelError::InvalidPayerAccount)?;
            check_writable_signer(da.deployer, ChannelError::InvalidDeployerAccount)?;
            check_bytes_match(
                da.deployer.key.as_ref(),
                owner,
                ChannelError::InvalidDeployerAccount,
            )?;
            check_writeable(da.deployment, ChannelError::InvalidDeploymentAccount)?;
            check_owner(
                da.deployment,
                &system_program::ID,
                ChannelError::InvalidDeploymentAccount,
            )?;
            ensure_0(da.deployment, ChannelError::InvalidDeploymentAccount)?;
            check_key_match(
                da.system_program,
                &system_program::ID,
                ChannelError::InvalidInstruction,
            )?;

            da.deployment_bump = Some(check_pda(
                &deployment_address_seeds(&img_id_hash(imageid)),
                da.deployment.key,
                ChannelError::InvalidDeploymentAccount,
            )?);
            return Ok(da);
        }

        Err(ChannelError::InvalidInstruction)
    }
}

pub struct ExecuteAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub deployment: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub execution_id: &'b str,
    pub exec_bump: Option<u8>,
}

impl<'a, 'b> ExecuteAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ExecutionRequestV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let evec = executionid;
            let mut ea = ExecuteAccounts {
                requester: &accounts[0],
                payer: &accounts[1],
                exec: &accounts[2],
                deployment: &accounts[3],
                callback_program: &accounts[4],
                system_program: &accounts[5],
                extra_accounts: &accounts[6..],
                execution_id: evec,
                exec_bump: None,
            };

            check_writable_signer(ea.requester, ChannelError::InvalidRequesterAccount)?;
            check_writable_signer(ea.payer, ChannelError::InvalidPayerAccount)?;
            check_writeable(ea.exec, ChannelError::InvalidExecutionAccount)?;
            check_owner(
                ea.exec,
                &system_program::ID,
                ChannelError::InvalidExecutionAccount,
            )?;
            ensure_0(ea.exec, ChannelError::InvalidExecutionAccount)?;
            check_owner(
                ea.deployment,
                &crate::ID,
                ChannelError::InvalidDeploymentAccount,
            )?;
            let deploy_data = &*ea
                .deployment
                .try_borrow_data()
                .map_err(|_| ChannelError::InvalidDeploymentAccount)?;
            let deploy = root_as_deploy_v1(&*&deploy_data)
                .map_err(|_| ChannelError::InvalidDeploymentAccount)?;

            let inputs = data.input().ok_or(ChannelError::InvalidInputs)?;
            // this should never be less than 1
            let required_input_size = deploy.inputs().map(|x| x.len()).unwrap_or(1);
            let mut num_sets = 0;
            let input_set: usize = inputs
                .iter()
                .filter(|i| {
                    // these must be changed on client to reference account index, the will be 1 byte
                    i.data().is_some() && i.input_type() == InputType::InputSet
                })
                .flat_map(|i| {
                    num_sets += 1;
                    // can panic here
                    let index = i.data().map(|x| x.bytes().get(0)).flatten().unwrap();
                    let rel_index = index - 6;
                    let account = ea
                        .extra_accounts
                        .get(rel_index as usize)
                        .ok_or(ChannelError::InvalidInputs)
                        .unwrap();
                    let data = account.data.borrow();
                    let input_set =
                        root_as_input_set(&*data).map_err(|_| ChannelError::InvalidInputs)?;
                    input_set
                        .inputs()
                        .map(|x| x.len())
                        .ok_or(ChannelError::InvalidInputs)
                })
                .fold(0, |acc, x| acc + x);

            if inputs.len() - num_sets + input_set != required_input_size {
                return Err(ChannelError::InvalidInputs);
            }
            //todo ensure inputs ar correct types/public private on chain, provers do this onsite so its low priority
            ea.exec_bump = Some(check_pda(
                &execution_address_seeds(&ea.requester.key, evec.as_bytes()),
                ea.exec.key,
                ChannelError::InvalidExecutionAccount,
            )?);

            if data.max_block_height() == 0 {
                return Err(ChannelError::MaxBlockHeightRequired);
            }

            if data.verify_input_hash() && data.input_digest().is_none() {
                return Err(ChannelError::InputDigestRequired);
            }

            or(
                &[
                    check_key_match(
                        ea.callback_program,
                        &crate::ID,
                        ChannelError::InvalidCallbackAccount,
                    ),
                    check_owner(
                        ea.callback_program,
                        &bpf_loader_upgradeable::ID,
                        ChannelError::InvalidCallbackAccount,
                    ),
                ],
                ChannelError::InvalidCallbackAccount,
            )?;
            check_key_match(
                ea.system_program,
                &system_program::ID,
                ChannelError::InvalidInstruction,
            )?;
            return Ok(ea);
        }

        Err(ChannelError::InvalidInstruction)
    }
}

struct StatusAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub prover: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub exec_bump: Option<u8>,
    pub eid: &'b str,
}

impl<'a, 'b> StatusAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b StatusV1<'b>,
    ) -> Result<Self, ChannelError> {
        let ea = &accounts[1];
        let prover = &accounts[3];
        let callback_program = &accounts[2];
        let eid = data
            .execution_id()
            .ok_or(ChannelError::InvalidExecutionAccount)?;
        let bmp = Some(check_pda(
            &execution_address_seeds(&accounts[0].key, eid.as_bytes()),
            ea.key,
            ChannelError::InvalidExecutionAccount,
        )?);
        let stat = StatusAccounts {
            requester: &accounts[0],
            exec: &accounts[1],
            callback_program,
            prover,
            extra_accounts: &accounts[4..],
            exec_bump: bmp,
            eid: eid,
        };
        Ok(stat)
    }
}

fn create_program_account<'a>(
    account: &'a AccountInfo<'a>,
    seeds: &[&[u8]],
    space: u64,
    payer: &'a AccountInfo<'a>,
    system: &'a AccountInfo<'a>,
    additional_lamports: Option<u64>,
) -> Result<(), ChannelError> {
    let lamports =
        Rent::default().minimum_balance(space as usize) + additional_lamports.unwrap_or(0);
    let create_pda_account_ix =
        system_instruction::create_account(&payer.key, &account.key, lamports, space, &crate::id());
    invoke_signed(
        &create_pda_account_ix,
        &[account.clone(), payer.clone(), system.clone()],
        &[seeds],
    )
    .map_err(|_e| ChannelError::InvalidSystemProgram)
}

fn save_structure<'a>(
    account: &'a AccountInfo<'a>,
    seeds: &[&[u8]],
    bytes: &[u8],
    payer: &'a AccountInfo<'a>,
    system: &'a AccountInfo<'a>,
    additional_lamports: Option<u64>,
) -> Result<(), ChannelError> {
    let space = bytes.len() as u64;
    create_program_account(account, seeds, space, payer, system, additional_lamports)?;
    sol_memcpy(&mut account.data.borrow_mut(), bytes, space as usize);
    Ok(())
}

fn cleanup_execution_account(
    exec: &AccountInfo,
    requester: &AccountInfo,
    exit_code: u8,
) -> Result<(), ProgramError> {
    exec.realloc(1, false)?;
    sol_memset(&mut exec.data.borrow_mut(), exit_code, 1);
    refund(exec, requester)
}

fn refund(exec: &AccountInfo, requester: &AccountInfo) -> Result<(), ProgramError> {
    //leave min lamports in the account so that account reuse is not possible
    let lamports = Rent::default().minimum_balance(1);
    let refund = exec.lamports();
    **exec.try_borrow_mut_lamports()? = lamports;
    **requester.try_borrow_mut_lamports()? += refund - lamports;
    Ok(())
}

fn payout_tip(exec: &AccountInfo, prover: &AccountInfo, tip: u64) -> Result<(), ProgramError> {
    **exec.try_borrow_mut_lamports()? -= tip;
    **prover.try_borrow_mut_lamports()? += tip;
    Ok(())
}

fn transfer_unowned<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
) -> Result<(), ProgramError> {
    let ix = system_instruction::transfer(from.key, to.key, lamports);
    invoke(&ix, &[from.clone(), to.clone()])
}

fn transfer_owned(from: &AccountInfo, to: &AccountInfo, lamports: u64) -> Result<(), ProgramError> {
    **from.try_borrow_mut_lamports()? -= lamports;
    **to.try_borrow_mut_lamports()? += lamports;
    Ok(())
}

#[inline]
pub fn program<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let ix = parse_ix_data(instruction_data).map_err(|_| ChannelError::InvalidInstructionParse)?;
    // borrow issues if these arent out in this block
    let er = ix.execute_v1_nested_flatbuffer();
    let st = ix.status_v1_nested_flatbuffer();
    let dp = ix.deploy_v1_nested_flatbuffer();
    let cl = ix.claim_v1_nested_flatbuffer();
    match ix.ix_type() {
        ChannelInstructionIxType::ClaimV1 => {
            if cl.is_none() {
                return Err(ChannelError::InvalidInstruction.into());
            }
            let cl = cl.unwrap();
            let ca = ClaimAccounts::from_instruction(accounts, &cl)?;
            let current_block = solana_program::clock::Clock::get()?.slot;

            if ca.existing_claim {
                let mut data = ca.exec_claim.try_borrow_mut_data()?;
                let current_claim = Claim::load_claim(*data)?;
                transfer_owned(ca.exec_claim, ca.claimer, ca.stake)?;
                if current_block > current_claim.block_commitment {
                    let claim =
                        Claim::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
                    drop(data);
                    Claim::save_claim(&claim, ca.exec_claim);
                    transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)?;
                } else {
                    return Err(ChannelError::ActiveClaimExists.into());
                }
            } else {
                let claim =
                    Claim::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
                transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)?;
                Claim::save_claim(&claim, ca.exec_claim);
            }
        }
        ChannelInstructionIxType::DeployV1 => {
            if dp.is_none() {
                return Err(ChannelError::InvalidInstruction.into());
            }
            let dp = dp.unwrap();
            let da = DeployAccounts::from_instruction(accounts, &dp)?;
            let b = [da.deployment_bump.unwrap()];
            let imghash = img_id_hash(da.image_id);
            let mut seeds = deployment_address_seeds(&imghash);
            seeds.push(&b);
            let bytes = ix.deploy_v1().unwrap().bytes();
            save_structure(
                da.deployment,
                &seeds,
                bytes,
                da.payer,
                da.system_program,
                None,
            )?;
        }
        ChannelInstructionIxType::ExecuteV1 => {
            if er.is_none() {
                return Err(ChannelError::InvalidInstruction.into());
            }
            let er = er.unwrap();
            let ea = ExecuteAccounts::from_instruction(accounts, &er)?;
            let b = [ea.exec_bump.unwrap()];
            let mut seeds = execution_address_seeds(ea.requester.key, ea.execution_id.as_bytes());
            seeds.push(&b);
            let bytes = ix.execute_v1().unwrap().bytes();
            save_structure(ea.exec, &seeds, bytes, ea.payer, ea.system_program, None)?;
        }
        ChannelInstructionIxType::StatusV1 => {
            if st.is_none() {
                return Err(ChannelError::InvalidInstruction.into());
            }
            let st = st.unwrap();
            let sa = StatusAccounts::from_instruction(accounts, &st)?;
            let er_ref = sa.exec.try_borrow_data()?;
            let er = root_as_execution_request_v1(&*er_ref)
                .map_err(|_| ChannelError::InvalidExecutionAccount)?;
            let pr = st.proof().filter(|x| x.len() == 256);
            let execution_digest = st.execution_digest();
            let output_digest = st.output_digest();
            if let (Some(proof), Some(exed), Some(outd)) = (pr, execution_digest, output_digest) {
                let proof: &[u8; 256] = proof
                    .bytes()
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                let inputs = prepare_inputs(
                    er.image_id().unwrap().as_bytes(),
                    exed.bytes(),
                    outd.bytes(),
                    st.exit_code_system(),
                    st.exit_code_user(),
                )?;
                msg!("here lasy");
                let verified = verify_risc0(proof, &inputs)?;
                if verified {
                    let callback_program_set =
                        sol_memcmp(sa.callback_program.key.as_ref(), crate::ID.as_ref(), 32) != 0;
                    let ix_prefix_set = er.callback_instruction_prefix().is_some();
                    if callback_program_set && ix_prefix_set {
                        let cbp = er
                            .callback_program_id()
                            .map(|b| b.bytes())
                            .unwrap_or(crate::ID.as_ref());
                        check_bytes_match(
                            cbp,
                            sa.callback_program.key.as_ref(),
                            ChannelError::InvalidCallbackProgram,
                        )?;

                        let b = [sa.exec_bump.unwrap()];
                        let mut seeds =
                            execution_address_seeds(sa.requester.key, sa.eid.as_bytes());
                        seeds.push(&b);
                        let mut ainfos = vec![sa.exec.clone(), sa.callback_program.clone()];
                        ainfos.extend(sa.extra_accounts.iter().cloned());
                        let mut accounts = vec![AccountMeta::new(*sa.exec.key, true)];
                        for a in sa.extra_accounts {
                            // dont cary feepayer signature through to callback
                            let _signer =
                                if sol_memcmp(a.key.as_ref(), sa.prover.key.as_ref(), 32) == 0 {
                                    false
                                } else {
                                    a.is_signer
                                };
                            if a.is_writable {
                                accounts.push(AccountMeta::new(*a.key, a.is_signer));
                            } else {
                                accounts.push(AccountMeta::new_readonly(*a.key, a.is_signer));
                            }
                        }
                        let payload = if er.forward_output() && st.committed_outputs().is_some() {
                            [
                                er.callback_instruction_prefix().unwrap().bytes(),
                                st.committed_outputs().unwrap().bytes(),
                            ]
                            .concat()
                        } else {
                            er.callback_instruction_prefix().unwrap().bytes().to_vec()
                        };
                        let callback_ix = Instruction::new_with_bytes(
                            *sa.callback_program.key,
                            &payload,
                            accounts,
                        );
                        let res = invoke_signed(&callback_ix, &ainfos, &[&seeds]);
                        match res {
                            Ok(_) => {}
                            Err(e) => {
                                msg!("{} Callback Failed: {:?}", sa.eid, e);
                            }
                        }
                    }
                    let tip = er.tip();
                    drop(er_ref);
                    payout_tip(sa.exec, sa.prover, tip)?;
                    cleanup_execution_account(sa.exec, sa.requester, ExitCode::Success as u8)?;
                } else {
                    msg!("{} Verifying Failed Cleaning up", sa.eid);
                    cleanup_execution_account(sa.exec, sa.requester, ExitCode::VerifyError as u8)?;
                }
            } else {
                msg!("{} Proving Failed Cleaning up", sa.eid);
                cleanup_execution_account(sa.exec, sa.requester, ExitCode::ProvingError as u8)?;
            }
        }
        _ => return Err(ChannelError::InvalidInstruction.into()),
    };
    Ok(())
}
