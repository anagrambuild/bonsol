use crate::error::ChannelError;
use crate::verifying_key::VERIFYINGKEY;
use anagram_bonsol_schema::{
    parse_ix_data, ChannelInstruction, ChannelInstructionIxType, ExecutionRequestV1,
};
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
use solana_program::{entrypoint, program_memory};
pub struct ExecuteAccounts<'a> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub execution_id: Vec<u8>,
    pub exec_bump: Option<u8>,
}

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

impl<'a> ExecuteAccounts<'a> {
    fn from_instruction<'b>(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ExecutionRequestV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let evec = executionid.bytes().to_owned();
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
                    ea.execution_id.as_slice(),
                ],
                ea.exec.key,
                ChannelError::InvalidExecutionAccount,
            )?);

            or(
                &[
                    check_key_match(
                        ea.callback_program,
                        &crate::id(),
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
    .map_err(|e| ChannelError::InvalidSystemProgram)
}

entrypoint!(process_instruction);
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let ix = parse_ix_data(instruction_data).map_err(|_| ChannelError::InvalidInstructionParse)?;
    let er = ix.execute_v1_nested_flatbuffer();

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
                ea.execution_id.as_slice(),
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
        ChannelInstructionIxType::StatusV1 => {},
        _ => return {
            Err(ChannelError::InvalidInstruction.into())
        }
    };
    Ok(())
}
