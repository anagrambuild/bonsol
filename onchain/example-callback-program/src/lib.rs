use bonsol_channel_interface::callback::handle_callback;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{declare_id, entrypoint, msg};

declare_id!("exay1T7QqsJPNcwzMiWubR6vZnqrgM16jZRraHgqBGG");
const SIMPLE_IMAGE_ID: &str = "68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960";

entrypoint!(main);
fn main(_program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (ix, data) = instruction_data.split_at(1);
    match ix[0] {
        1 => {
            let execution_account = accounts[0].key; // in most cases you will store this somewhere else to ensure it matches with some user storage
            let output = handle_callback(SIMPLE_IMAGE_ID, execution_account, accounts, data)?;
            if output.len() == 1 && output[0] == 1 {
                msg!("Correct Json Attestation");
            }
            Ok(())
        }
        _ => return Err(ProgramError::InvalidInstructionData.into()),
    }
}
