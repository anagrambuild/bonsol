use bonsol_interface::callback::handle_callback;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

use solana_program::program_memory::sol_memcmp;
use solana_program::{declare_id, entrypoint, msg};

declare_id!("exay1T7QqsJPNcwzMiWubR6vZnqrgM16jZRraHgqBGG");
const SIMPLE_IMAGE_ID: &str = "68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960";

static EA1: Pubkey = pubkey!("3b6DR2gbTJwrrX27VLEZ2FJcHrDvTSLKEcTLVhdxCoaf");
static EA2: Pubkey = pubkey!("g7dD1FHSemkUQrX1Eak37wzvDjscgBW2pFCENwjLdMX");
static EA3: Pubkey = pubkey!("FHab8zDcP1DooZqXHWQowikqtXJb1eNHc46FEh1KejmX");

entrypoint!(main);
fn main(_program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (ix, data) = instruction_data.split_at(1);
    match ix[0] {
        1 => {
            let execution_account = accounts[0].key; // in most cases you will store this somewhere else to ensure it matches with some user storage
            let output = handle_callback(SIMPLE_IMAGE_ID, execution_account, accounts, data)?;
            if sol_memcmp(accounts[1].key.as_ref(), EA1.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[2].key.as_ref(), EA2.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[3].key.as_ref(), EA3.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }

            assert!(accounts[2].is_writable, "Writable account not found");
            if output.len() == 1 && output[0] == 1 {
                msg!("Correct Json Attestation");
            }
            Ok(())
        }
        _ => return Err(ProgramError::InvalidInstructionData.into()),
    }
}
