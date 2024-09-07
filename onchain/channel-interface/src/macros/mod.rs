

/// Macro to request execution via cpi of a bonsol program from within a solana program
#[macro_export(local_inner_macros)]
macro_rules! execute {
    (
        $signer:expr,
        $signer_seeds:expr,
        $accounts:expr,
        $image_id:expr,
        $execution_id:expr,
        $inputs:expr,
        $tip:expr,
        $expiration:expr,
        $execution_config:expr,
        $callback_config:expr,
      ) => {
      use $crate::instructions::execute_v1;
      use solana_program::program::invoke_signed
      let ix = execute_v1(
          &signer,
          image_id,
          execution_id, // execution id can be any thing but has to be unique and the matching execution id needs to be sent in the request
          inputs,
          tip,
          expiration,
          execution_config,
          Some(callback_config),
      ).map_err(|e| Into::<ProgramError>::into(e))?;
      invoke_signed(&ix, accounts, &[signer_seeds])
    };
}

macro_rules! callback {
    (
        $accounts:expr,
        $instruction_data:expr,
    ) => {
      use anagram_bonsol_schema::{ChannelInstruction, ChannelInstructionIxType};
      
        
    };
}


