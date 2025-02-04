use bonsol_schema::{
    Account, ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, DeployV1,
    DeployV1Args, ExecutionRequestV1, ExecutionRequestV1Args, InputBuilder, InputType,
    ProgramInputType, ProverVersion,
};
use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::error::ClientError;
use crate::util::{deployment_address, execution_address};

#[cfg(feature = "on-chain")]
use {
    solana_program::instruction::AccountMeta, solana_program::instruction::Instruction,
    solana_program::pubkey::Pubkey, solana_program::system_program,
};

#[cfg(not(feature = "on-chain"))]
use {
    solana_sdk::instruction::AccountMeta, solana_sdk::instruction::Instruction,
    solana_sdk::pubkey::Pubkey, solana_sdk::system_program,
};

pub fn deploy_v1(
    signer: &Pubkey,
    image_id: &str,
    image_size: u64,
    program_name: &str,
    url: &str,
    inputs: Vec<ProgramInputType>,
) -> Result<Instruction, ClientError> {
    let (deployment_account, _) = deployment_address(image_id);
    let accounts = vec![
        AccountMeta::new(signer.to_owned(), true),
        AccountMeta::new(signer.to_owned(), true),
        AccountMeta::new(deployment_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let mut fbb = FlatBufferBuilder::new();
    let url = fbb.create_string(url);
    let image_id = fbb.create_string(image_id);
    let name = fbb.create_string(program_name);
    let owner = fbb.create_vector(signer.as_ref());
    let fb_inputs = fbb.create_vector(inputs.as_slice());
    let fbb_deploy = DeployV1::create(
        &mut fbb,
        &DeployV1Args {
            owner: Some(owner),
            image_id: Some(image_id),
            program_name: Some(name),
            url: Some(url),
            size: image_size,
            inputs: Some(fb_inputs),
        },
    );
    fbb.finish(fbb_deploy, None);
    let ix_data = fbb.finished_data();
    let mut fbb = FlatBufferBuilder::new();
    let ix = fbb.create_vector(ix_data);
    let fbb_ix = ChannelInstruction::create(
        &mut fbb,
        &ChannelInstructionArgs {
            ix_type: ChannelInstructionIxType::DeployV1,
            deploy_v1: Some(ix),
            ..Default::default()
        },
    );
    fbb.finish(fbb_ix, None);
    let ix_data = fbb.finished_data();
    Ok(Instruction::new_with_bytes(crate::ID, ix_data, accounts))
}

// todo hold attributes for scheme and versions selection
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ExecutionConfig<'a> {
    pub verify_input_hash: bool,
    pub input_hash: Option<&'a [u8]>,
    pub forward_output: bool,
}

#[cfg(feature = "serde")]
pub mod serde_helpers {
    pub mod pubkey {
        use std::str::FromStr;

        use serde::{self, Deserialize, Deserializer, Serializer};
        use solana_sdk::pubkey::Pubkey;

        pub fn serialize<S>(value: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&value.to_string())
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Pubkey::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    pub mod optpubkey {
        use std::str::FromStr;

        use serde::{self, Deserialize, Deserializer, Serializer};
        use solana_sdk::pubkey::Pubkey;

        pub fn serialize<S>(value: &Option<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(v) => serializer.serialize_str(&v.to_string()),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Pubkey::from_str(&s)
                .map_err(serde::de::Error::custom)
                .map(Some)
        }
    }
}

impl<'a> ExecutionConfig<'a> {
    pub fn validate(&self) -> Result<(), ClientError> {
        if self.verify_input_hash && self.input_hash.is_none() {
            return Err(ClientError::InvalidInput);
        }
        Ok(())
    }
}

impl Default for ExecutionConfig<'_> {
    fn default() -> Self {
        ExecutionConfig {
            verify_input_hash: true,
            input_hash: None,
            forward_output: false,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CallbackConfig {
    #[cfg_attr(feature = "serde", serde(default, with = "serde_helpers::pubkey"))]
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
    pub extra_accounts: Vec<AccountMeta>,
}

pub struct InputRef<'a> {
    pub input_type: InputType,
    pub data: &'a [u8],
}

impl<'a> InputRef<'a> {
    pub fn new(input_type: InputType, data: &'a [u8]) -> Self {
        Self { input_type, data }
    }

    pub fn public(data: &'a [u8]) -> Self {
        Self {
            input_type: InputType::PublicData,
            data,
        }
    }
    pub fn private(data: &'a [u8]) -> Self {
        Self {
            input_type: InputType::Private,
            data,
        }
    }
    pub fn public_proof(data: &'a [u8]) -> Self {
        Self {
            input_type: InputType::PublicProof,
            data,
        }
    }
    pub fn url(data: &'a [u8]) -> Self {
        Self {
            input_type: InputType::PublicUrl,
            data,
        }
    }
    pub fn public_account(data: &'a [u8]) -> Self {
        Self {
            input_type: InputType::PublicAccountData,
            data,
        }
    }
}

/// Executes a bonsol program.
/// This sends and instruction to the bonsol program which requests execution from the bonsol network
pub fn execute_v1<'a>(
    requester: &Pubkey,
    payer: &Pubkey,
    image_id: &str,
    execution_id: &str,
    inputs: Vec<InputRef<'a>>,
    tip: u64,
    expiration: u64,
    config: ExecutionConfig<'a>,
    callback: Option<CallbackConfig>,
    prover_version: Option<ProverVersion>,
) -> Result<Instruction, ClientError> {
    let (execution_account, _) = execution_address(requester, execution_id.as_bytes());
    let (deployment_account, _) = deployment_address(image_id);
    execute_v1_with_accounts(
        requester,
        payer,
        &execution_account,
        &deployment_account,
        image_id,
        execution_id,
        inputs,
        tip,
        expiration,
        config,
        callback,
        prover_version,
    )
}
/// Executes a bonsol program with the provided accounts
/// This is more efficient than using the execute_v1 function
/// but requires the user to provide the accounts
pub fn execute_v1_with_accounts<'a>(
    requester: &Pubkey,
    payer: &Pubkey,
    execution_account: &Pubkey,
    deployment_account: &Pubkey,
    image_id: &str,
    execution_id: &str,
    inputs: Vec<InputRef>,
    tip: u64,
    expiration: u64,
    config: ExecutionConfig,
    callback: Option<CallbackConfig>,
    prover_version: Option<ProverVersion>,
) -> Result<Instruction, ClientError> {
    config.validate()?;
    let mut fbb = FlatBufferBuilder::new();
    let mut callback_pubkey = None; // aviod clone
    let (callback_program_id, callback_instruction_prefix, extra_accounts) =
        if let Some(cb) = callback {
            callback_pubkey = Some(cb.program_id);
            let cb_program_id = fbb.create_vector(cb.program_id.as_ref());
            let cb_instruction_prefix = fbb.create_vector(cb.instruction_prefix.as_slice());
            let ealen = cb.extra_accounts.len();
            fbb.start_vector::<WIPOffset<Account>>(ealen);
            for ea in cb.extra_accounts.iter().rev() {
                let pkbytes = arrayref::array_ref!(ea.pubkey.as_ref(), 0, 32);
                let eab = Account::new(ea.is_writable as u8, pkbytes);
                fbb.push(eab);
            }
            (
                Some(cb_program_id),
                Some(cb_instruction_prefix),
                Some(fbb.end_vector(ealen)),
            )
        } else {
            (None, None, None)
        };
    let accounts = vec![
        AccountMeta::new(*requester, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new(*execution_account, false),
        AccountMeta::new_readonly(*deployment_account, false),
        AccountMeta::new_readonly(callback_pubkey.unwrap_or(crate::ID), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let inputlen = inputs.len();
    let mut inputs_vec = Vec::with_capacity(inputlen);
    for input in inputs {
        let data_off = fbb.create_vector(input.data);
        let mut ibb = InputBuilder::new(&mut fbb);
        ibb.add_data(data_off);
        ibb.add_input_type(input.input_type);
        let input = ibb.finish();
        inputs_vec.push(input);
    }
    let fb_inputs = fbb.create_vector(&inputs_vec);
    let image_id = fbb.create_string(image_id);
    let execution_id = fbb.create_string(execution_id);

    let input_digest = config.input_hash.map(|ih| fbb.create_vector(ih));

    // typically cli will pass None for the optional prover_version indicating bonsol should handle
    // the default case here
    let prover_version = prover_version.unwrap_or_default();
    let fbb_execute = ExecutionRequestV1::create(
        &mut fbb,
        &ExecutionRequestV1Args {
            tip,
            execution_id: Some(execution_id),
            image_id: Some(image_id),
            callback_program_id,
            callback_instruction_prefix,
            forward_output: config.forward_output,
            verify_input_hash: config.verify_input_hash,
            input: Some(fb_inputs),
            max_block_height: expiration,
            input_digest,
            callback_extra_accounts: extra_accounts,
            prover_version,
        },
    );
    fbb.finish(fbb_execute, None);
    let ix_data = fbb.finished_data();
    let mut fbb = FlatBufferBuilder::new();
    let ix = fbb.create_vector(ix_data);
    let fbb_ix = ChannelInstruction::create(
        &mut fbb,
        &ChannelInstructionArgs {
            ix_type: ChannelInstructionIxType::ExecuteV1,
            execute_v1: Some(ix),
            ..Default::default()
        },
    );
    fbb.finish(fbb_ix, None);
    let ix_data = fbb.finished_data();
    Ok(Instruction::new_with_bytes(crate::ID, ix_data, accounts))
}
