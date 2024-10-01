use {
    crate::error::ClientError,
    bonsol_channel_utils::execution_address,
    bonsol_schema::root_as_execution_request_v1,
    solana_program::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        msg,
        program_error::ProgramError,
        program_memory::sol_memcmp,
        pubkey::Pubkey,
        system_program,
    },
};

/// This is the callback handler for the bonsol program, use this to properly validate an incoming callback from bonsol
/// Ensure you strip the instruction prefix from the data before passing it to this function and that the Execution Id
/// matches the one in the execution request account
pub fn handle_callback<'a>(
    execution_account: Pubkey,
    accounts: &[AccountInfo],
    stripped_data: &'a [u8],
) -> Result<&'a [u8], ProgramError> {
    let er_info = accounts
        .get(0)
        .ok_or::<ProgramError>(ClientError::InvalidCallbackInstructionAccounts.into())?;
    if sol_memcmp(er_info.key.as_ref(), &execution_account.as_ref(), 32) != 0 {
        return Err(ClientError::InvalidCallbackInstructionAccounts.into());
    }
    if !er_info.is_signer {
        return Err(ClientError::InvalidCallbackSignature.into());
    }
    let er_data = &er_info.try_borrow_data()?;
    if er_data.len() < 2 {
        return Err(ClientError::InvalidCallbackInstructionAccounts.into());
    }
    // Ensure this is a valid execution request data
    root_as_execution_request_v1(er_data).map_err(|_| ProgramError::InvalidInstructionData)?;
    Ok(stripped_data)
}

pub fn handle_callback_id<'a>(
    execution_id: &str,
    request_account: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    data: &'a [u8],
) -> Result<&'a [u8], ProgramError> {
    let (execution_account, _) = execution_address(request_account, execution_id.as_bytes());
    handle_callback(execution_account, accounts, data)
}
