use std::{str::from_utf8};

use crate::error::ChannelError;
use crate::verifying_key::VERIFYINGKEY;
use anagram_bonsol_schema::{
    parse_ix_data, root_as_execution_request_v1,
    ChannelInstructionIxType, ExecutionRequestV1, StatusTypes, StatusV1,
};
use groth16_solana::groth16::Groth16Verifier;

use solana_program::{
    account_info::AccountInfo,
    bpf_loader_upgradeable,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_memory::sol_memcpy,
    pubkey::{Pubkey, PUBKEY_BYTES},
    rent::Rent,
    system_instruction, system_program,
};
type G1 = ark_bn254::g1::G1Affine;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use solana_program::program_error::ProgramError;
use solana_program::{entrypoint, program_memory};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_memory::sol_memcmp,
};
use std::ops::Neg;

fn err<T>(i: Result<T, ChannelError>, err: ChannelError) -> Result<T, ChannelError> {
    i.map_err(|_| err)
}

fn or(res: &[Result<(), ChannelError>], error: ChannelError) -> Result<(), ChannelError> {
    for r in res {
        if r.is_ok() {
            return Ok(());
        }
    }
    Err(error)
}

fn and(res: &[Result<(), ChannelError>], error: ChannelError) -> Result<(), ChannelError> {
    for r in res {
        if r.is_err() {
            return Err(error);
        }
    }
    Ok(())
}

fn check_pda(seeds: Vec<&[u8]>, tg: &Pubkey, error: ChannelError) -> Result<u8, ChannelError> {
    let (pda, _bump_seed) = Pubkey::find_program_address(&seeds, &crate::id());
    if program_memory::sol_memcmp(&pda.to_bytes(), &tg.to_bytes(), PUBKEY_BYTES) != 0 {
        return Err(error);
    }
    Ok(_bump_seed)
}

fn check_writable_signer(account: &AccountInfo, error: ChannelError) -> Result<(), ChannelError> {
    if !account.is_writable || !account.is_signer {
        return Err(error);
    }
    Ok(())
}

fn ensure_0(account: &AccountInfo, error: ChannelError) -> Result<(), ChannelError> {
    if account.lamports() != 0 {
        return Err(error);
    }
    if account.data_len() != 0 {
        return Err(error);
    }
    Ok(())
}

fn check_writeable(account: &AccountInfo, error: ChannelError) -> Result<(), ChannelError> {
    if !account.is_writable {
        return Err(error);
    }
    Ok(())
}

fn check_key_match(
    account: &AccountInfo,
    key: &Pubkey,
    error: ChannelError,
) -> Result<(), ChannelError> {
    if program_memory::sol_memcmp(&account.key.to_bytes(), &key.to_bytes(), PUBKEY_BYTES) != 0 {
        return Err(error);
    }
    Ok(())
}

fn check_bytes_match(
    bytes1: &[u8],
    bytes2: &[u8],
    error: ChannelError,
) -> Result<(), ChannelError> {
    if bytes1.len() != bytes2.len() || program_memory::sol_memcmp(bytes1, bytes2, bytes1.len()) != 0
    {
        return Err(error);
    }
    Ok(())
}

fn check_owner(
    account: &AccountInfo,
    owner: &Pubkey,
    error: ChannelError,
) -> Result<(), ChannelError> {
    if account.owner != owner {
        return Err(error);
    }
    Ok(())
}
pub struct ExecuteAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub execution_id: &'b [u8],
    pub exec_bump: Option<u8>,
}

impl<'a, 'b> ExecuteAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ExecutionRequestV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let evec = executionid.bytes();
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
                vec![
                    "execution".as_bytes(),
                    &ea.requester.key.to_bytes(),
                    ea.execution_id,
                ],
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
    pub eid: &'a [u8],
}

impl<'a> StatusAccounts<'a> {
    fn from_instruction<'b>(
        accounts: &'a [AccountInfo<'a>],
        _data: &'b StatusV1<'b>,
    ) -> Result<Self, ChannelError> {
        let ea = accounts[1].clone();
        let prover = &accounts[3];
        let eadata = ea.data.take();
        let er = root_as_execution_request_v1(eadata).unwrap();
        let eid = er
            .execution_id()
            .ok_or(ChannelError::InvalidExecutionAccount)?;
        let bmp = Some(check_pda(
            vec![
                "execution".as_bytes(),
                &accounts[0].key.as_ref(),
                eid.bytes(),
            ],
            ea.key,
            ChannelError::InvalidExecutionAccount,
        )?);
        let cbp = er
            .callback_program_id()
            .map(|b| b.bytes())
            .unwrap_or(crate::ID.as_ref());
        check_bytes_match(
            cbp,
            prover.key.as_ref(),
            ChannelError::InvalidCallbackProgram,
        )?;

        let stat: StatusAccounts<'_> = StatusAccounts {
            requester: &accounts[0],
            exec: &accounts[1],
            callback_program: &accounts[2],
            prover,
            extra_accounts: &accounts[4..],
            exec_bump: bmp,
            er,
            eid: eid.bytes(),
        };

        Ok(stat)
    }
}

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    for b in bytes.chunks(32) {
        for byte in b.iter().rev() {
            vec.push(*byte);
        }
    }
    vec
}

fn sized_range<const N: usize>(slice: &[u8]) -> Result<[u8; N], ChannelError> {
    slice
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)
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

fn refund(exec: &AccountInfo, requester: &AccountInfo) -> Result<(), ProgramError> {
    let refund = exec.lamports();
    **exec.try_borrow_mut_lamports()? = 0;
    **requester.try_borrow_mut_lamports()? += refund;
    Ok(())
}

fn payout_tip(exec: &AccountInfo, prover: &AccountInfo, tip: u64) -> Result<(), ProgramError> {
    **exec.try_borrow_mut_lamports()? -= tip;
    **prover.try_borrow_mut_lamports()? += tip;
    Ok(())
}

entrypoint!(process_instruction);
pub fn process_instruction<'a>(
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
            let seeds = vec![
                "execution".as_bytes(),
                ea.requester.key.as_ref(),
                ea.execution_id,
                &b,
            ];
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
            if st.status() == StatusTypes::Failed {
                let eid = from_utf8(sa.eid).unwrap();
                msg!("{} Proving Failed Cleaning up", eid);
                refund(sa.exec, sa.requester)?;
            }
            let pr = st.proof().filter(|x| x.len() == 256);
            let input = st.input().filter(|x| x.len() == 128);
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
                let proof_a: G1 = G1::deserialize_with_mode(
                    &*[&change_endianness(&proof[0..64]), &[0u8][..]].concat(),
                    Compress::No,
                    Validate::Yes,
                )
                .unwrap();
                let mut proof_a_neg = [0u8; 65];
                proof_a
                    .neg()
                    .x
                    .serialize_with_mode(&mut proof_a_neg[..32], Compress::No)
                    .unwrap();
                proof_a
                    .neg()
                    .y
                    .serialize_with_mode(&mut proof_a_neg[32..], Compress::No)
                    .unwrap();

                let proof_a = change_endianness(&proof_a_neg[..64])
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                let proof_b = proof[64..192]
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                let proof_c = proof[192..256]
                    .try_into()
                    .map_err(|_| ChannelError::InvalidInstruction)?;

                let ins: [[u8; 32]; 4] = [
                    sized_range::<32>(&inputs[0..32])?,
                    sized_range::<32>(&inputs[32..64])?,
                    sized_range::<32>(&inputs[64..96])?,
                    sized_range::<32>(&inputs[96..128])?,
                ];
                let mut verifier: Groth16Verifier<4> =
                    Groth16Verifier::new(&proof_a, &proof_b, &proof_c, &ins, &VERIFYINGKEY)
                        .map_err(|_| ChannelError::InvalidInstruction)?;
                let verified = verifier
                    .verify()
                    .map_err(|_| ChannelError::InvalidInstruction)?;
                if verified {
                    let callback_program_set =
                        sol_memcmp(sa.callback_program.key.as_ref(), crate::ID.as_ref(), 32) != 0;
                    let ix_prefix_set = sa.er.callback_instruction_prefix().is_some();
                    if callback_program_set && ix_prefix_set {
                        let b = [sa.exec_bump.unwrap()];
                        let seeds = vec![
                            "execution".as_bytes(),
                            sa.requester.key.as_ref(),
                            sa.eid,
                            &b,
                        ];
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
                            Ok(_) => {
                                let tip = sa.er.tip();
                                payout_tip(sa.exec, sa.prover, tip)?;
                            }
                            Err(e) => {
                                let eid = from_utf8(sa.eid).unwrap();
                                msg!("{} Callback Failed: {:?}", eid, e);
                            }
                        }
                    }
                    let tip = sa.er.tip();
                    payout_tip(sa.exec, sa.prover, tip)?;
                    refund(sa.exec, sa.requester)?;
                } else {
                    let eid = from_utf8(sa.eid).unwrap();
                    msg!("{} Verifying Failed Cleaning up", eid);
                    refund(sa.exec, sa.requester)?;
                }
            } else {
                let eid = from_utf8(sa.eid).unwrap();
                msg!("{} Proving Failed", eid);
                refund(sa.exec, sa.requester)?;
            }
        }
        _ => return Err(ChannelError::InvalidInstruction.into()),
    };
    Ok(())
}
