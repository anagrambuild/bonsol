use crate::actions::*;
use crate::assertions::*;
use crate::error::ChannelError;
use crate::proof_handling::{output_digest, prepare_inputs, verify_risc0};
use crate::utilities::*;
use anagram_bonsol_channel_utils::{
    deployment_address_seeds, execution_address_seeds, execution_claim_address_seeds, img_id_hash,
};
use anagram_bonsol_schema::{
    parse_ix_data, root_as_deploy_v1, root_as_execution_request_v1, root_as_input_set,
    ChannelInstructionIxType, ClaimV1, DeployV1, ExecutionRequestV1, ExitCode, InputType, StatusV1,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_memory::{sol_memcmp, sol_memcpy, sol_memset};
use solana_program::pubkey::Pubkey;


#[inline]
pub fn program<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &'a [u8],
) -> ProgramResult {
    let ix = parse_ix_data(instruction_data).map_err(|_| ChannelError::InvalidInstructionParse)?;
    match ix.ix_type() {
        ChannelInstructionIxType::ClaimV1 => {
            process_claim_v1(accounts, ix)?;
        }
        ChannelInstructionIxType::DeployV1 => {
            process_deploy_v1(accounts, ix)?;
        }
        ChannelInstructionIxType::ExecuteV1 => {
            process_execute_v1(accounts, ix)?; 
        }
        ChannelInstructionIxType::StatusV1 => {
            process_status_v1(accounts, ix)?;
        }
        _ => return Err(ChannelError::InvalidInstruction.into()),
    };
    Ok(())
}