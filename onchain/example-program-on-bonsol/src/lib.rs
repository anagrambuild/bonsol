use bonsol_interface::callback::{handle_callback, BonsolCallback};
use bonsol_interface::instructions::{execute_v1, CallbackConfig, ExecutionConfig, InputRef};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;

use solana_program::clock::Clock;
use solana_program::instruction::AccountMeta;
use solana_program::program::invoke_signed;
use solana_program::program_memory::sol_memcmp;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{declare_id, entrypoint, msg, system_instruction};
use std::str::from_utf8;

declare_id!("exay1T7QqsJPNcwzMiWubR6vZnqrgM16jZRraHgqBGG");
const SIMPLE_IMAGE_ID: &str = "68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960";

static EA1: Pubkey = pubkey!("3b6DR2gbTJwrrX27VLEZ2FJcHrDvTSLKEcTLVhdxCoaf");
static EA2: Pubkey = pubkey!("g7dD1FHSemkUQrX1Eak37wzvDjscgBW2pFCENwjLdMX");
static EA3: Pubkey = pubkey!("FHab8zDcP1DooZqXHWQowikqtXJb1eNHc46FEh1KejmX");

entrypoint!(main);
/// This program is used as a testbed for bonsol, to test various scenarios
/// and to test the callback functionality
fn main<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix, data) = instruction_data.split_at(1);
    match ix[0] {
        0 => {
            let payer = &accounts[0]; //any feepayer
            if data.len() < 57 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            let execution_id =
                from_utf8(&data[0..16]).map_err(|_| ProgramError::InvalidInstructionData)?;
            let input_hash = &data[16..48];
            let expiration = u64::from_le_bytes(
                data[48..56]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );
            let bump = data[56];
            let private_input_url = &data[57..];
            let requester = &accounts[1]; //pda of this program
            let system = &accounts[2];
            let execution_account = &accounts[3]; //the pda of bonsol that represents the execution request
            create_program_account(
                requester,
                &[execution_id.as_bytes(), &[bump]],
                32,
                payer,
                system,
                None,
            )?;
            let tip = 1000;
            let expiration = Clock::get()?.slot + expiration; //high experation since we run this on potatoes in CI
            let ix = execute_v1(
                requester.key,
                payer.key,
                SIMPLE_IMAGE_ID,
                &execution_id,
                vec![
                    InputRef::public("{\"attestation\":\"test\"}".as_bytes()),
                    InputRef::private(private_input_url),
                ],
                tip,
                expiration,
                ExecutionConfig {
                    verify_input_hash: true,
                    input_hash: Some(input_hash),
                    forward_output: true,
                },
                Some(CallbackConfig {
                    program_id: crate::id(),
                    instruction_prefix: vec![1],
                    extra_accounts: vec![
                        AccountMeta::new(*requester.key, false),
                        AccountMeta::new(EA1, false),
                        AccountMeta::new_readonly(EA2, false),
                        AccountMeta::new_readonly(EA3, false),
                    ],
                }),
            )
            .map_err(|_| ProgramError::InvalidInstructionData)?;
            invoke_signed(&ix, accounts, &[&[execution_id.as_bytes(), &[bump]]])?;
            let mut data = requester.try_borrow_mut_data()?;
            data.copy_from_slice(&execution_account.key.to_bytes());
            Ok(())
        }
        1 => {
            let requester = &accounts[1];
            let requester_data = requester.try_borrow_data()?;
            let execution_account = Pubkey::try_from(&requester_data[0..32])
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            let callback_output: BonsolCallback =
                handle_callback(SIMPLE_IMAGE_ID, &execution_account, accounts, data)?;
            if sol_memcmp(accounts[2].key.as_ref(), EA1.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[3].key.as_ref(), EA2.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[4].key.as_ref(), EA3.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if !accounts[2].is_writable {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if callback_output.committed_outputs.len() == 1
                && callback_output.committed_outputs[0] == 1
            {
                msg!("Correct Json Attestation");
            }
            Ok(())
        }
        //only callback test
        2 => {
            let callback_output: BonsolCallback =
                handle_callback(SIMPLE_IMAGE_ID, &accounts[0].key, accounts, data)?;
            if sol_memcmp(accounts[1].key.as_ref(), EA1.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[2].key.as_ref(), EA2.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            if sol_memcmp(accounts[3].key.as_ref(), EA3.as_ref(), 32) != 0 {
                return Err(ProgramError::InvalidInstructionData.into());
            }
            assert!(accounts[1].is_writable, "Writable account not found");
            if callback_output.committed_outputs.len() == 1
                && callback_output.committed_outputs[0] == 1
            {
                msg!("Correct Json Attestation");
            }
            Ok(())
        }

        _ => return Err(ProgramError::InvalidInstructionData.into()),
    }
}

pub fn create_program_account<'a>(
    account: &'a AccountInfo<'a>,
    seeds: &[&[u8]],
    space: u64,
    payer: &'a AccountInfo<'a>,
    system: &'a AccountInfo<'a>,
    additional_lamports: Option<u64>,
) -> Result<(), ProgramError> {
    let lamports = Rent::get()?.minimum_balance(space as usize) + additional_lamports.unwrap_or(0);
    let create_pda_account_ix =
        system_instruction::create_account(&payer.key, &account.key, lamports, space, &crate::id());
    invoke_signed(
        &create_pda_account_ix,
        &[account.clone(), payer.clone(), system.clone()],
        &[seeds],
    )
    .map_err(|_e| ProgramError::Custom(0))
}
