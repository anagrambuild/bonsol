use solana_program::account_info::AccountInfo;
use solana_program::program_memory;
use solana_program::pubkey::{Pubkey, PUBKEY_BYTES};

use crate::error::ChannelError;

pub fn err<T>(i: Result<T, ChannelError>, err: ChannelError) -> Result<T, ChannelError> {
    i.map_err(|_| err)
}

pub fn or(res: &[Result<(), ChannelError>], error: ChannelError) -> Result<(), ChannelError> {
    for r in res {
        if r.is_ok() {
            return Ok(());
        }
    }
    Err(error)
}

pub fn and(res: &[Result<(), ChannelError>], error: ChannelError) -> Result<(), ChannelError> {
    for r in res {
        if r.is_err() {
            return Err(error);
        }
    }
    Ok(())
}

pub fn check_pda(seeds: Vec<&[u8]>, tg: &Pubkey, error: ChannelError) -> Result<u8, ChannelError> {
    let (pda, _bump_seed) = Pubkey::find_program_address(&seeds, &crate::id());
    if program_memory::sol_memcmp(&pda.to_bytes(), &tg.to_bytes(), PUBKEY_BYTES) != 0 {
        return Err(error);
    }
    Ok(_bump_seed)
}

pub fn check_writable_signer(
    account: &AccountInfo,
    error: ChannelError,
) -> Result<(), ChannelError> {
    if !account.is_writable || !account.is_signer {
        return Err(error);
    }
    Ok(())
}

pub fn ensure_0(account: &AccountInfo, error: ChannelError) -> Result<(), ChannelError> {
    if account.lamports() != 0 {
        return Err(error);
    }
    if account.data_len() != 0 {
        return Err(error);
    }
    Ok(())
}

pub fn check_writeable(account: &AccountInfo, error: ChannelError) -> Result<(), ChannelError> {
    if !account.is_writable {
        return Err(error);
    }
    Ok(())
}

pub fn check_key_match(
    account: &AccountInfo,
    key: &Pubkey,
    error: ChannelError,
) -> Result<(), ChannelError> {
    if program_memory::sol_memcmp(&account.key.to_bytes(), &key.to_bytes(), PUBKEY_BYTES) != 0 {
        return Err(error);
    }
    Ok(())
}

pub fn check_bytes_match(
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

pub fn check_owner(
    account: &AccountInfo,
    owner: &Pubkey,
    error: ChannelError,
) -> Result<(), ChannelError> {
    if account.owner != owner {
        return Err(error);
    }
    Ok(())
}
