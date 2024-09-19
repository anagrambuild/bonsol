pub mod channel_instruction_generated;
pub mod claim_v1_generated;
pub mod deploy_v1_generated;
pub mod execution_request_v1_generated;
pub mod input_set_generated;
pub mod input_type_generated;
pub mod status_v1_generated;
pub mod input_set_op_v1_generated;
use std::fmt::Display;

use error::ChannelSchemaError;
use num_derive::{FromPrimitive, ToPrimitive};
pub mod error;
pub use {
    channel_instruction_generated::*, claim_v1_generated::*, deploy_v1_generated::*,
    execution_request_v1_generated::*, input_set_generated::*, input_type_generated::*,
    status_v1_generated::*,
};
pub fn parse_ix_data<'a>(ix_data: &'a [u8]) -> Result<ChannelInstruction, ChannelSchemaError> {
    let instruction =
        root_as_channel_instruction(ix_data).map_err(|_| ChannelSchemaError::InvalidInstruction)?;
    Ok(instruction)
}

#[derive(ToPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum ExitCode {
    Success = 0,
    VerifyError = 1,
    ProvingError = 2,
    InputError = 3,
    Expired = 4,
}

impl Display for ExitCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ExitCode::Success => write!(f, "Success"),
      ExitCode::VerifyError => write!(f, "VerifyError"),
      ExitCode::ProvingError => write!(f, "ProvingError"),
      ExitCode::InputError => write!(f, "InputError"),
      ExitCode::Expired => write!(f, "Expired"),
    }
  }
}


impl InputT {
    pub fn new(input_type: InputType, data: Option<Vec<u8>>) -> Self {
        Self {
            input_type,
            data
        }
    }

    pub fn public(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicData,
            data: Some(data),
        }
    }
    pub fn private(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::Private,
            data: Some(data),
        }
    }
    pub fn public_proof(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicProof,
            data: Some(data),
        }
    }
    pub fn url(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicUrl,
            data: Some(data),
        }
    }
    pub fn input_set(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::InputSet,
            data: Some(data)
        }
    }
    pub fn public_account(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicAccountData,
            data: Some(data),
        }
    }
}