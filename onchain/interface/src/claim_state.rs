use bytemuck::{Pod, Zeroable};

use crate::error::ClientError;

#[cfg(feature = "on-chain")]
use {
    solana_program::account_info::AccountInfo, solana_program::program_memory::sol_memcpy,
    solana_program::pubkey::Pubkey,
};

#[cfg(not(feature = "on-chain"))]
use solana_sdk::pubkey::Pubkey;

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct ClaimStateV1 {
    pub claimer: [u8; 32],
    pub claimed_at: u64,
    pub block_commitment: u64,
}

pub struct ClaimStateHolder {
    data: Vec<u8>,
}

impl ClaimStateHolder {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: data.try_into().unwrap(),
        }
    }

    pub fn claim(&self) -> Result<&ClaimStateV1, ClientError> {
        bytemuck::try_from_bytes(&self.data).map_err(|_| ClientError::InvalidClaimAccount)
    }
}

impl ClaimStateV1 {
    pub fn load_claim(ca_data: &mut [u8]) -> Result<&Self, ClientError> {
        bytemuck::try_from_bytes::<ClaimStateV1>(ca_data)
            .map_err(|_| ClientError::InvalidClaimAccount)
    }

    pub fn load_claim_owned(ca_data: &[u8]) -> Result<Self, ClientError> {
        bytemuck::try_pod_read_unaligned::<ClaimStateV1>(ca_data)
            .map_err(|_| ClientError::InvalidClaimAccount)
    }

    pub fn from_claim_ix(claimer: &Pubkey, slot: u64, block_commitment: u64) -> Self {
        ClaimStateV1 {
            claimer: claimer.to_bytes(),
            claimed_at: slot,
            block_commitment,
        }
    }

    #[cfg(feature = "on-chain")]
    pub fn save_claim(claim: &Self, ca: &AccountInfo) {
        let claim_data = bytemuck::bytes_of(claim);
        sol_memcpy(&mut ca.data.borrow_mut(), claim_data, claim_data.len());
    }
}
