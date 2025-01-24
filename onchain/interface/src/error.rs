#[cfg(feature = "on-chain")]
use solana_program::program_error::ProgramError;

#[cfg(not(feature = "on-chain"))]
use {solana_sdk::msg, solana_sdk::program_error::ProgramError};

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("InvalidInput")]
    InvalidInput,
    #[error("InvalidCallbackExtraAccounts")]
    InvalidCallbackExtraAccounts,
    #[error("InvalidCallbackProgram")]
    InvalidCallbackProgram,
    #[error("InvalidInstructionAccounts")]
    InvalidCallbackInstructionAccounts,
    #[error("InvalidCallbackSignature")]
    InvalidCallbackSignature,
    #[error("InvalidCallbackData")]
    InvalidCallbackData,
    #[error("InvalidClaimAccount")]
    InvalidClaimAccount,
    #[error("InvalidCallbackImageId")]
    InvalidCallbackImageId,
    #[error("Execution Request Reused")]
    ExecutionRequestReused,
}

impl From<ClientError> for ProgramError {
    fn from(val: ClientError) -> Self {
        ProgramError::Custom(val as u32)
    }
}
