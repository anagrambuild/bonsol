use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelError {
    #[error("Invalid Requester Account")]
    InvalidRequesterAccount,
    #[error("Invalid Execution Account")]
    InvalidExecutionAccount,
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Invalid Instruction")]
    InvalidInstructionParse,
    #[error("Invalid Callback Account")]
    InvalidCallbackAccount,
    #[error("Invalid system program")]
    InvalidSystemProgram,
    #[error("Cannot borrow data from account")]
    CannotBorrowData,
    #[error("Invalid Conversion")]
    InvalidConversion,
    #[error("Invalid Callback Program")]
    InvalidCallbackProgram,
}

impl From<ChannelError> for ProgramError {
    fn from(e: ChannelError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
