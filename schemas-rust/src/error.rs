use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ChannelSchemaError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
}
