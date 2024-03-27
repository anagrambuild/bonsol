#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]

use solana_program::declare_id;
use solana_program::{keccak, keccak::Hash};
use solana_program::pubkey::Pubkey;

mod assertions;
pub mod error;
pub mod program;
pub mod proof_handling;
mod verifying_keys;

declare_id!("BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

pub fn execution_address_seeds<'a>(requester: &'a Pubkey, execution_id: &'a [u8]) -> Vec<&'a [u8]> {
    vec!["execution".as_bytes(), requester.as_ref(), execution_id]
}

pub fn deployment_address_seeds<'a>(hash: &'a Hash) -> Vec<&'a [u8]> {
    vec!["deployment".as_bytes(), hash.as_ref()]
}

pub fn execution_claim_address_seeds<'a>(
    execution_id: &'a [u8],
) -> Vec<&'a [u8]> {
    vec![
        "execution_claim".as_bytes(),
        execution_id,
    ]
}

pub fn execution_address(requester: &Pubkey, execution_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &execution_address_seeds(requester, execution_id),
        &crate::ID,
    )
}

#[inline]
pub fn img_id_hash(image_id: &str) -> Hash {
    keccak::hash(image_id.as_bytes())
}

pub fn deployment_address(image_id: &str) -> (Pubkey, u8) {
    let hsh = img_id_hash(image_id);
    Pubkey::find_program_address(&deployment_address_seeds(
        &hsh,
    ), &crate::ID)
}

pub fn execution_claim_address(
    execution_id: &[u8],
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &execution_claim_address_seeds(execution_id),
        &crate::ID,
    )
}