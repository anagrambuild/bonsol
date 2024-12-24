use crate::{actions::*, error::ChannelError};
use bonsol_interface::bonsol_schema::{parse_ix_data, ChannelInstructionIxType};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

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
        ChannelInstructionIxType::InputSetOpV1 => {
            process_input_set_v1(accounts, ix)?;
        }
        _ => return Err(ChannelError::InvalidInstruction.into()),
    };
    Ok(())
}
