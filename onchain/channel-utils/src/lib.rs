#[cfg(feature = "on-chain")]
use {
    solana_program::declare_id,
    solana_program::pubkey::Pubkey,
    solana_program::{keccak, keccak::Hash},
};

#[cfg(not(feature = "on-chain"))]
use {
    solana_sdk::declare_id,
    solana_sdk::pubkey::Pubkey,
    solana_sdk::{keccak, keccak::Hash},
};


declare_id!("BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew");

pub fn execution_address_seeds<'a>(requester: &'a Pubkey, execution_id: &'a [u8]) -> Vec<&'a [u8]> {
    vec!["execution".as_bytes(), requester.as_ref(), execution_id]
}

pub fn deployment_address_seeds<'a>(hash: &'a Hash) -> Vec<&'a [u8]> {
    vec!["deployment".as_bytes(), hash.as_ref()]
}

pub fn execution_claim_address_seeds<'a>(execution_id: &'a [u8]) -> Vec<&'a [u8]> {
    vec!["execution_claim".as_bytes(), execution_id]
}

pub fn execution_address(requester: &Pubkey, execution_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&execution_address_seeds(requester, execution_id), &ID)
}

#[inline]
pub fn img_id_hash(image_id: &str) -> Hash {
    keccak::hash(image_id.as_bytes())
}

pub fn deployment_address(image_id: &str) -> (Pubkey, u8) {
    let hsh = img_id_hash(image_id);
    Pubkey::find_program_address(&deployment_address_seeds(&hsh), &ID)
}

pub fn execution_claim_address(execution_id: &[u8]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&execution_claim_address_seeds(execution_id), &ID)
}
