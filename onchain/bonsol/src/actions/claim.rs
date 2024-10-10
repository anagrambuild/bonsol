use bonsol_interface::{
    bonsol_schema::{root_as_execution_request_v1, ChannelInstruction, ClaimV1},
    claim_state::ClaimStateV1,
    util::{execution_address_seeds, execution_claim_address_seeds},
};

use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, system_program, sysvar::Sysvar,
};

use crate::{assertions::*, error::ChannelError, utilities::*};

pub struct ClaimAccounts<'a, 'b> {
    pub exec: &'a AccountInfo<'a>,
    pub requester: &'a AccountInfo<'a>,
    pub exec_claim: &'a AccountInfo<'a>,
    pub claimer: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub execution_id: &'b str,
    pub block_commitment: u64,
    pub existing_claim: bool,
    pub stake: u64,
    pub expired: bool,
}

impl<'a, 'b> ClaimAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ClaimV1<'b>,
        current_block: u64,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let mut ca = ClaimAccounts {
                exec: &accounts[0],
                requester: &accounts[1],
                exec_claim: &accounts[2],
                claimer: &accounts[3],
                payer: &accounts[4],
                system_program: &accounts[5],
                execution_id: executionid,
                block_commitment: data.block_commitment(),
                existing_claim: false,
                stake: 0,
                expired: false,
            };
            check_writable_signer(ca.payer, ChannelError::InvalidPayerAccount)?;
            check_writable_signer(ca.claimer, ChannelError::InvalidClaimerAccount)?;
            check_writeable(ca.exec_claim, ChannelError::InvalidClaimAccount)?;
            check_writeable(ca.exec, ChannelError::InvalidExecutionAccount)?;
            check_owner(ca.exec, &crate::ID, ChannelError::InvalidExecutionAccount)?;
            let exec_seeds = execution_address_seeds(ca.requester.key, executionid.as_bytes());
            check_pda(
                &exec_seeds,
                ca.exec.key,
                ChannelError::InvalidExecutionAccount,
            )?;
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
            if execution_request.max_block_height() < current_block {
                ca.expired = true;
            }
            // make this more dynamic
            ca.stake = tip / 2;
            let mut exec_claim_seeds = execution_claim_address_seeds(ca.exec.key.as_ref());
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
                    std::mem::size_of::<ClaimStateV1>() as u64,
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
    let current_block = solana_program::clock::Clock::get()?.slot;
    let ca = ClaimAccounts::from_instruction(accounts, &cl, current_block)?;
    if ca.expired {
        cleanup_execution_account(ca.exec, ca.claimer, ChannelError::ExecutionExpired as u8)?;
        msg!("Execution expired");
        return Ok(());
    }
    if ca.existing_claim {
        let mut data = ca.exec_claim.try_borrow_mut_data()?;
        let current_claim =
            ClaimStateV1::load_claim(*data).map_err(|_| ChannelError::InvalidClaimAccount)?;
        transfer_owned(ca.exec_claim, ca.claimer, ca.stake)?;
        if current_block > current_claim.block_commitment {
            let claim =
                ClaimStateV1::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
            drop(data);
            ClaimStateV1::save_claim(&claim, ca.exec_claim);
            transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)
        } else {
            Err(ChannelError::ActiveClaimExists.into())
        }
    } else {
        let claim =
            ClaimStateV1::from_claim_ix(&ca.claimer.key, current_block, ca.block_commitment);
        transfer_unowned(ca.claimer, ca.exec_claim, ca.stake)?;
        ClaimStateV1::save_claim(&claim, ca.exec_claim);
        Ok(())
    }
}
