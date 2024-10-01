use bonsol_channel_utils::execution_claim_address_seeds;
use bonsol_schema::root_as_execution_request_v1;
use bonsol_schema::{ChannelInstruction, ClaimV1};
use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memcpy;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program::sysvar::Sysvar;

use crate::assertions::*;
use crate::error::ChannelError;
use crate::utilities::*;

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
            let execution_request = root_as_execution_request_v1(&*exec_data)
                .map_err(|_| ChannelError::InvalidExecutionAccount)?;
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

pub fn process_claim_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    ix: ChannelInstruction<'a>,
) -> Result<(), ProgramError> {
    let cl = ix.claim_v1_nested_flatbuffer();
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
            let claim = Claim::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
            drop(data);
            Claim::save_claim(&claim, ca.exec_claim);
            transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)
        } else {
            Err(ChannelError::ActiveClaimExists.into())
        }
    } else {
        let claim = Claim::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
        transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)?;
        Claim::save_claim(&claim, ca.exec_claim);
        Ok(())
    }
}
