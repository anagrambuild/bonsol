use solana_program::program_memory::{sol_memset, sol_memcpy};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::rent::Rent;
use solana_program::system_instruction;
use solana_program::program::invoke;
use solana_program::program::invoke_signed;

use crate::error::ChannelError;
pub fn cleanup_execution_account(
  exec: &AccountInfo,
  requester: &AccountInfo,
  exit_code: u8,
) -> Result<(), ProgramError> {
  exec.realloc(1, false)?;
  sol_memset(&mut exec.data.borrow_mut(), exit_code, 1);
  refund(exec, requester)
}

pub fn refund(exec: &AccountInfo, requester: &AccountInfo) -> Result<(), ProgramError> {
  //leave min lamports in the account so that account reuse is not possible
  let lamports = Rent::default().minimum_balance(1);
  let refund = exec.lamports();
  **exec.try_borrow_mut_lamports()? = lamports;
  **requester.try_borrow_mut_lamports()? += refund - lamports;
  Ok(())
}

pub fn payout_tip(exec: &AccountInfo, prover: &AccountInfo, tip: u64) -> Result<(), ProgramError> {
  **exec.try_borrow_mut_lamports()? -= tip;
  **prover.try_borrow_mut_lamports()? += tip;
  Ok(())
}

pub fn transfer_unowned<'a>(
  from: &AccountInfo<'a>,
  to: &AccountInfo<'a>,
  lamports: u64,
) -> Result<(), ProgramError> {
  let ix = system_instruction::transfer(from.key, to.key, lamports);
  invoke(&ix, &[from.clone(), to.clone()])
}

pub fn transfer_owned(from: &AccountInfo, to: &AccountInfo, lamports: u64) -> Result<(), ProgramError> {
  **from.try_borrow_mut_lamports()? -= lamports;
  **to.try_borrow_mut_lamports()? += lamports;
  Ok(())
}

pub fn save_structure<'a>(
  account: &'a AccountInfo<'a>,
  seeds: &[&[u8]],
  bytes: &[u8],
  payer: &'a AccountInfo<'a>,
  system: &'a AccountInfo<'a>,
  additional_lamports: Option<u64>,
) -> Result<(), ChannelError> {
  let space = bytes.len() as u64;
  create_program_account(account, seeds, space, payer, system, additional_lamports)?;
  sol_memcpy(&mut account.data.borrow_mut(), bytes, space as usize);
  Ok(())
}

pub fn create_program_account<'a>(
  account: &'a AccountInfo<'a>,
  seeds: &[&[u8]],
  space: u64,
  payer: &'a AccountInfo<'a>,
  system: &'a AccountInfo<'a>,
  additional_lamports: Option<u64>,
) -> Result<(), ChannelError> {
  let lamports =
      Rent::default().minimum_balance(space as usize) + additional_lamports.unwrap_or(0);
  let create_pda_account_ix =
      system_instruction::create_account(&payer.key, &account.key, lamports, space, &crate::id());
  invoke_signed(
      &create_pda_account_ix,
      &[account.clone(), payer.clone(), system.clone()],
      &[seeds],
  )
  .map_err(|_e| ChannelError::InvalidSystemProgram)
}
