#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]

use solana_program::declare_id;
use solana_program::pubkey::Pubkey;
mod assertions;
pub mod error;
pub mod program;
pub mod proof_handling;
mod verifying_keys;

declare_id!("BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn");
mod entrypoint;

pub fn execution_address_seeds<'a>(requester: &'a Pubkey, execution_id: &'a [u8]) -> Vec<&'a [u8]> {
    vec!["execution".as_bytes(), requester.as_ref(), execution_id]
}

pub fn execution_claim_address_seeds<'a>(
    requester: &'a Pubkey,
    execution_id: &'a [u8],
    claimer: &'a Pubkey,
) -> Vec<&'a [u8]> {
    vec![
        "execution_claim".as_bytes(),
        requester.as_ref(),
        execution_id,
        claimer.as_ref(),
    ]
}

pub fn execution_address(requester: &Pubkey, execution_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &execution_address_seeds(requester, execution_id),
        &crate::ID,
    )
}

pub fn execution_claim_address(
    requester: &Pubkey,
    execution_id: &[u8],
    claimer: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &execution_claim_address_seeds(requester, execution_id, claimer),
        &crate::ID,
    )
}