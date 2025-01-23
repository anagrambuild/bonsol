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
    #[error("Invalid Input Data")]
    InvalidInputs,
    #[error("Invalid Input Length")]
    InvalidInputLength,
    #[error("Invalid Instruction Parsing")]
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
    #[error("Invalid Proof")]
    InvalidProof,
    #[error("Proof Verification Failed")]
    ProofVerificationFailed,
    #[error("Invalid Public Inputs")]
    InvalidPublicInputs,
    #[error("Max block height required")]
    MaxBlockHeightRequired,
    #[error("Verify input digest requires digest")]
    InputDigestRequired,
    #[error("Invalid Payer Account")]
    InvalidPayerAccount,
    #[error("Invalid Deployer Account")]
    InvalidDeployerAccount,
    #[error("Invalid Deployment Account")]
    InvalidDeploymentAccount,
    #[error("Invalid Claimer Account")]
    InvalidClaimerAccount,
    #[error("Invalid Claim Account")]
    InvalidClaimAccount,
    #[error("Active claim already exists")]
    ActiveClaimExists,
    #[error("Invalid Stake Account")]
    InvalidStakeAccount,
    #[error("Insufficient Stake")]
    InsufficientStake,
    #[error("Inputs dont match")]
    InputsDontMatch,
    #[error("Invalid Field Element")]
    InvalidFieldElement,
    #[error("Missing Image Checksum")]
    MissingImageChecksum,
    #[error("Invalid Image Checksum")]
    InvalidImageChecksum,
    #[error("Transfer Error")]
    TransferError,
    #[error("Execution expired")]
    ExecutionExpired,
    #[error("Invalid Deployment Account PDA")]
    InvalidDeploymentAccountPDA,
    #[error("Invalid Callback Extra Accounts")]
    InvalidCallbackExtraAccounts,
    #[error("Invalid Input Type")]
    InvalidInputType,
    #[error("Deployment Already Exists")]
    DeploymentAlreadyExists,
    #[error("Invalid Execution Account Data")]
    InvalidExecutionAccountData,
    #[error("Invalid Execution Id")]
    InvalidExecutionId,
    #[error("Invalid Execution Account Owner")]
    InvalidExecutionAccountOwner,
    #[error("Unexpected Proof System")]
    UnexpectedProofSystem,
}

impl From<ChannelError> for ProgramError {
    fn from(e: ChannelError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
