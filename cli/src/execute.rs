use crate::common::*;
use anyhow::Result;
use bonsol_sdk::{
    execution_address, execution_claim_address, input_resolver::{DefaultInputResolver, InputResolver, ProgramInput}, instructions::execute_v1, BonsolClient, ExecutionAccountStatus, ExitCode, InputType
};
use indicatif::ProgressBar;
use num_traits::FromPrimitive;
use sha2::{Digest, Sha256};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::{fs::File, sync::Arc};
use tokio::time::Instant;

pub async fn execution_waiter(
    sdk: &BonsolClient,
    requester: Pubkey,
    execution_id: String,
    expiry: u64,
    timeout: Option<u64>,
) -> Result<()> {
    let indicator = ProgressBar::new_spinner();

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let now = Instant::now();
    loop {
        if let Some(timeout) = timeout {
            if now.elapsed().as_secs() > timeout {
                return Err(anyhow::anyhow!("Timeout"));
            }
        }
        interval.tick().await;

        let current_block = sdk.get_block_height().await?;
        indicator.set_message(format!(
            "Waiting for execution to be claimed, current block {}",
            current_block
        ));
        if current_block > expiry {
            indicator.finish_with_message("Execution expired");
            return Err(anyhow::anyhow!("Execution expired"));
        }

        let claim_state = sdk.get_claim_state_v1(&requester, &execution_id).await?;
        let claim = claim_state.claim()?;
        indicator.finish_with_message(format!(
            "Claimed by {} at slot {}, committed {}",
            bs58::encode(claim.claimer).into_string(),
            claim.claimed_at,
            claim.block_commitment
        ));
        break;
    }
    //now we are looking for execution request finished
    loop {
        if let Some(timeout) = timeout {
            if now.elapsed().as_secs() > timeout {
                indicator.finish_with_message("Execution timed out");
                return Err(anyhow::anyhow!("Timeout"));
            }
        }
        interval.tick().await;
        indicator.set_message("Checking status of execution");
        let exec_status = sdk.get_execution_request_v1(&requester, &execution_id).await?;
        match exec_status {
            ExecutionAccountStatus::Completed(ec) => {
                indicator.finish_with_message(format!("Execution completed with exit code {}", ec));
                return Ok(());
            }
            ExecutionAccountStatus::Pending(_) => {
                indicator.finish_with_message("Execution Still pending");
                continue;
            }
        }
    }
}

pub async fn execute(
    sdk: &BonsolClient,
    rpc_url: String,
    keypair: &impl Signer,
    execution_request_file: Option<String>,
    image_id: Option<String>,
    execution_id: Option<String>,
    timeout: Option<u64>,
    inputs_file: Option<String>,
    tip: Option<u64>,
    expiry: Option<u64>,
    stdin: Option<String>,
    wait: bool,
) -> Result<()> {
    // dont need async here.
    let erstr =
        execution_request_file.ok_or(anyhow::anyhow!("Execution request file not provided"))?;
    let erfile = File::open(erstr)?;
    let execution_request_file: ExecutionRequestFile = serde_json::from_reader(erfile)?;
    let inputs = if let Some(inputs) = execution_request_file.inputs {
        inputs
    } else {
        execute_get_inputs(inputs_file, stdin)?
    };
    let execution_id = execution_id
        .or(execution_request_file.execution_id)
        .ok_or(anyhow::anyhow!("Execution id not provided"))?;
    let image_id = image_id
        .or(execution_request_file.image_id)
        .ok_or(anyhow::anyhow!("Image id not provided"))?;
    let tip = tip
        .or(execution_request_file.tip)
        .ok_or(anyhow::anyhow!("Tip not provided"))?;
    let expiry = expiry
        .or(execution_request_file.max_block_height)
        .ok_or(anyhow::anyhow!("Expiry not provided"))?;
    let callback_config = execution_request_file.callback_config;
    let mut execution_config = execution_request_file.execution_config;
    let signer = keypair.pubkey();
    let inputs = execute_transform_cli_inputs(inputs)?;
    let hash_inputs = execution_config.verify_input_hash
        && inputs.iter().all(|i| i.input_type != InputType::Private);
    if hash_inputs {
        let rpc_client = Arc::new(RpcClient::new(rpc_url.clone()));
        let input_resolver =
            DefaultInputResolver::new(Arc::new(reqwest::Client::new()), rpc_client);
        let inputs = input_resolver.resolve_public_inputs(inputs.clone()).await?;
        let mut hash = Sha256::new();
        for input in inputs {
            if let ProgramInput::Resolved(ri) = input {
                hash.update(&ri.data);
            } else {
                return Err(anyhow::anyhow!("Unresolved input"));
            }
        }
        let digest = hash.finalize();
        execution_config.input_hash = Some(digest.to_vec());
    }
    execute_v1(
        &signer,
        &image_id,
        &execution_id,
        inputs,
        tip,
        expiry,
        execution_config,
        callback_config,
    )?;

    if wait {
        execution_waiter(sdk, keypair.pubkey(), execution_id, expiry, timeout).await?;
    }
    Ok(())
}
