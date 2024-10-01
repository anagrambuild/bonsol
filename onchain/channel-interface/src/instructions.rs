use bonsol_channel_utils::{deployment_address, execution_address};
use bonsol_schema::{
    Account, ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, DeployV1,
    DeployV1Args, ExecutionRequestV1, ExecutionRequestV1Args, Input as FBBInput, InputBuilder,
    InputType, ProgramInputType,
};
use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::error::ClientError;

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
            size_: image_size,
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

pub struct ExecutionConfig {
    pub verify_input_hash: bool,
    pub input_hash: Option<Vec<u8>>,
    pub forward_output: bool,
}

impl ExecutionConfig {
    pub fn validate(&self) -> Result<(), ClientError> {
        if self.verify_input_hash && self.input_hash.is_none() {
            return Err(ClientError::InvalidInput);
        }
        Ok(())
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            verify_input_hash: true,
            input_hash: None,
            forward_output: false,
        }
    }
}
pub struct CallbackConfig {
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
    pub extra_accounts: Vec<AccountMeta>,
}
pub struct Input {
    pub input_type: InputType,
    pub data: Vec<u8>,
}

impl Input {
    pub fn public(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicData,
            data,
        }
    }
    pub fn private(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::Private,
            data,
        }
    }
    pub fn public_proof(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicProof,
            data,
        }
    }
    pub fn url(data: Vec<u8>) -> Self {
        Self {
            input_type: InputType::PublicUrl,
            data,
        }
    }
    pub fn input_set(data: Pubkey) -> Self {
        Self {
            input_type: InputType::InputSet,
            data: data.to_bytes().to_vec(),
        }
    }
    pub fn public_account(data: Pubkey) -> Self {
        Self {
            input_type: InputType::PublicAccountData,
            data: data.to_bytes().to_vec(),
        }
    }
}

pub fn execute_v1(
    signer: &Pubkey,
    image_id: &str,
    execution_id: &str,
    inputs: Vec<Input>,
    tip: u64,
    expiration: u64,
    config: ExecutionConfig,
    callback: Option<CallbackConfig>,
) -> Result<Instruction, ClientError> {
    config.validate()?;
    let (execution_account, _) = execution_address(signer, execution_id.as_bytes());
    let (deployment_account, _) = deployment_address(image_id);
    let mut fbb = FlatBufferBuilder::new();
    let mut callback_pubkey = None; // aviod clone
    let (callback_program_id, callback_instruction_prefix, extra_accounts) =
        if let Some(cb) = callback {
            callback_pubkey = Some(cb.program_id);
            let cb_program_id = fbb.create_vector(cb.program_id.as_ref());
            let cb_instruction_prefix = fbb.create_vector(cb.instruction_prefix.as_slice());
            let ealen = cb.extra_accounts.len();
            fbb.start_vector::<WIPOffset<Account>>(ealen);
            for ea in cb.extra_accounts {
                let pkbytes = arrayref::array_ref!(ea.pubkey.as_ref(), 0, 32);
                let eab = Account::new(ea.is_writable, pkbytes);
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
    let mut accounts = vec![
        AccountMeta::new(signer.to_owned(), true),
        AccountMeta::new(signer.to_owned(), true),
        AccountMeta::new(execution_account, false),
        AccountMeta::new(deployment_account, false),
        AccountMeta::new_readonly(callback_pubkey.unwrap_or(crate::ID), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let inputlen = inputs.len();
    fbb.start_vector::<WIPOffset<FBBInput>>(inputlen);
    for input in inputs {
        match input.input_type {
            InputType::InputSet => {
                let input_set_pubkey = Pubkey::try_from(input.data)
                    .map_err(|_| ClientError::InvalidInputSetAddress)?;
                accounts.push(AccountMeta::new_readonly(input_set_pubkey, false));
                let data_off = fbb.create_vector(&[(accounts.len() - 1) as u8]);
                let mut ibb = InputBuilder::new(&mut fbb);
                ibb.add_input_type(InputType::InputSet);
                // add the index of the account
                ibb.add_data(data_off);
                let input_set = ibb.finish();
                fbb.push(input_set);
            }
            _ => {
                let data_off = fbb.create_vector(&input.data);
                let mut ibb = InputBuilder::new(&mut fbb);
                ibb.add_data(data_off);
                ibb.add_input_type(input.input_type);
                let input = ibb.finish();
                fbb.push(input);
            }
        }
    }
    let fb_inputs = fbb.end_vector(inputlen);
    let image_id = fbb.create_string(image_id);
    let execution_id = fbb.create_string(execution_id);

    let input_digest = if let Some(ih) = config.input_hash {
        Some(fbb.create_vector(ih.as_slice()))
    } else {
        None
    };
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
