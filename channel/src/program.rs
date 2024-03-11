use crate::assertions::*;
use crate::error::ChannelError;
use crate::execution_address_seeds;
use crate::proof_handling::verify_risc0;
use anagram_bonsol_schema::{
    parse_ix_data, root_as_execution_request_v1, ChannelInstructionIxType, ExecutionRequestV1,
    ExitCode, StatusTypes, StatusV1,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::{sol_memcmp, sol_memcpy, sol_memset};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::{bpf_loader_upgradeable, msg, system_instruction, system_program};

pub struct ExecuteAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
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
                exec: &accounts[1],
                callback_program: &accounts[2],
                system_program: &accounts[3],
                extra_accounts: &accounts[4..],
                execution_id: evec,
                exec_bump: None,
            };
            check_writable_signer(ea.requester, ChannelError::InvalidRequesterAccount)?;
            and(
                &[
                    check_writeable(ea.exec, ChannelError::InvalidExecutionAccount),
                    check_owner(
                        ea.exec,
                        &system_program::ID,
                        ChannelError::InvalidExecutionAccount,
                    ),
                    ensure_0(ea.exec, ChannelError::InvalidExecutionAccount),
                ],
                ChannelError::InvalidExecutionAccount,
            )?;

            ea.exec_bump = Some(check_pda(
                execution_address_seeds(&ea.requester.key, evec.as_bytes()),
                ea.exec.key,
                ChannelError::InvalidExecutionAccount,
            )?);

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

struct StatusAccounts<'a> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub prover: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub exec_bump: Option<u8>,
    pub er: ExecutionRequestV1<'a>,
    pub eid: &'a str,
}

impl<'a> StatusAccounts<'a> {
    fn from_instruction<'b>(
        accounts: &'a [AccountInfo<'a>],
        _data: &'b StatusV1<'b>,
    ) -> Result<Self, ChannelError> {
        let ea = accounts[1].clone();
        let prover = &accounts[3];
        let callback_program = &accounts[2];
        let eadata = ea.data.take();
        let er = root_as_execution_request_v1(eadata).unwrap();
        let eid = er
            .execution_id()
            .ok_or(ChannelError::InvalidExecutionAccount)?;
        let bmp = Some(check_pda(
            execution_address_seeds(&accounts[0].key, eid.as_bytes()),
            ea.key,
            ChannelError::InvalidExecutionAccount,
        )?);
        let cbp = er
            .callback_program_id()
            .map(|b| b.bytes())
            .unwrap_or(crate::ID.as_ref());
        check_bytes_match(
            cbp,
            callback_program.key.as_ref(),
            ChannelError::InvalidCallbackProgram,
        )?;

        let stat: StatusAccounts<'_> = StatusAccounts {
            requester: &accounts[0],
            exec: &accounts[1],
            callback_program,
            prover,
            extra_accounts: &accounts[4..],
            exec_bump: bmp,
            er,
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

#[inline]
pub fn program<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let ix = parse_ix_data(instruction_data).map_err(|_| ChannelError::InvalidInstructionParse)?;
    let er = ix.execute_v1_nested_flatbuffer();
    let st = ix.status_v1_nested_flatbuffer();
    match ix.ix_type() {
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
            let space = bytes.len() as u64;
            let tip = er.tip();
            create_program_account(
                ea.exec,
                &seeds,
                space,
                ea.requester,
                ea.system_program,
                Some(tip),
            )?;
            sol_memcpy(&mut ea.exec.data.borrow_mut(), bytes, bytes.len());
        }
        ChannelInstructionIxType::StatusV1 => {
            if st.is_none() {
                return Err(ChannelError::InvalidInstruction.into());
            }
            let st = st.unwrap();
            let sa = StatusAccounts::from_instruction(accounts, &st)?;
            let pr = st.proof().filter(|x| x.len() == 256);
            let input = st.inputs().filter(|x| x.len() == 128);
            if st.status() == StatusTypes::Completed && pr.is_some() && input.is_some() {
                let proof: &[u8; 256] = pr
                    .unwrap()
                    .bytes()
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                let inputs: &[u8; 128] = input
                    .unwrap()
                    .bytes()
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                let verified = verify_risc0(proof, inputs)?;
                if verified {
                    let callback_program_set =
                        sol_memcmp(sa.callback_program.key.as_ref(), crate::ID.as_ref(), 32) != 0;
                    let ix_prefix_set = sa.er.callback_instruction_prefix().is_some();
                    if callback_program_set && ix_prefix_set {
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

                        let payload = sa.er.callback_instruction_prefix().unwrap().bytes();
                        let callback_ix = Instruction::new_with_bytes(
                            *sa.callback_program.key,
                            payload,
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
                    let tip = sa.er.tip();
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
