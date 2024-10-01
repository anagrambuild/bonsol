#[cfg(feature = "on-chain")]
use {solana_program::msg, solana_program::program_error::ProgramError};

#[cfg(not(feature = "on-chain"))]
use {solana_sdk::msg, solana_sdk::program_error::ProgramError};

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("InvalidInput")]
    InvalidInput,
    #[error("InvalidInputSetAddress")]
    InvalidInputSetAddress,
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
}

impl Into<ProgramError> for ClientError {
    fn into(self) -> ProgramError {
        msg!(&self.to_string());
        ProgramError::Custom(self as u32)
    }
}
