use std::time::Duration;

use anyhow::Result;
use bonsol_interface::bonsol_schema::{root_as_deploy_v1, root_as_execution_request_v1};
use bonsol_interface::claim_state::ClaimStateHolder;
use bytes::Bytes;
use futures_util::TryFutureExt;
use instructions::{CallbackConfig, ExecutionConfig};
use num_traits::FromPrimitive;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSendTransactionConfig;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::{v0, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use tokio::time::Instant;

pub use bonsol_interface::bonsol_schema::{
    ClaimV1T, DeployV1T, ExecutionRequestV1T, ExitCode, InputSetT, InputT, InputType,
    ProgramInputType, StatusTypes,
};
pub use bonsol_interface::util::*;
pub use bonsol_interface::{instructions, ID};
pub use flatbuffers;

pub struct BonsolClient {
    rpc_client: RpcClient,
}

pub enum ExecutionAccountStatus {
    Completed(ExitCode),
    Pending(ExecutionRequestV1T),
}

impl BonsolClient {
    pub fn new(rpc_url: String) -> Self {
        BonsolClient {
            rpc_client: RpcClient::new(rpc_url),
        }
    }

    pub async fn get_current_slot(&self) -> Result<u64> {
        self.rpc_client
            .get_slot()
            .map_err(|_| anyhow::anyhow!("Failed to get slot"))
            .await
    }

    pub fn with_rpc_client(rpc_client: RpcClient) -> Self {
        BonsolClient { rpc_client }
    }

    pub async fn get_deployment_v1(&self, image_id: &str) -> Result<DeployV1T> {
        let (deployment_account, _) = deployment_address(image_id);
        let account = self
            .rpc_client
            .get_account_with_commitment(&deployment_account, CommitmentConfig::confirmed())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account: {:?}", e))?
            .value
            .ok_or(anyhow::anyhow!("Invalid deployment account"))?;
        let deployment = root_as_deploy_v1(&account.data)
            .map_err(|_| anyhow::anyhow!("Invalid deployment account"))?;
        Ok(deployment.unpack())
    }

    pub async fn get_execution_request_v1(
        &self,
        requester_pubkey: &Pubkey,
        execution_id: &str,
    ) -> Result<ExecutionAccountStatus> {
        let (er, _) = execution_address(requester_pubkey, execution_id.as_bytes());
        let account = self
            .rpc_client
            .get_account_with_commitment(&er, CommitmentConfig::confirmed())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account: {:?}", e))?
            .value
            .ok_or(anyhow::anyhow!("Invalid execution request account"))?;
        if account.data.len() == 1 {
            let ec =
                ExitCode::from_u8(account.data[0]).ok_or(anyhow::anyhow!("Invalid exit code"))?;
            return Ok(ExecutionAccountStatus::Completed(ec));
        }
        let er = root_as_execution_request_v1(&account.data)
            .map_err(|_| anyhow::anyhow!("Invalid execution request account"))?;
        Ok(ExecutionAccountStatus::Pending(er.unpack()))
    }

    pub async fn get_claim_state_v1<'a>(
        &self,
        requester_pubkey: &Pubkey,
        execution_id: &str,
    ) -> Result<ClaimStateHolder> {
        let (exad, _) = execution_address(requester_pubkey, execution_id.as_bytes());
        let (eca, _) = execution_claim_address(exad.as_ref());
        let account = self
            .rpc_client
            .get_account_with_commitment(&eca, CommitmentConfig::confirmed())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account: {:?}", e))?
            .value
            .ok_or(anyhow::anyhow!("Invalid claim account"))?;
        Ok(ClaimStateHolder::new(account.data))
    }

    pub async fn download_program(&self, image_id: &str) -> Result<Bytes> {
        let deployment = self.get_deployment_v1(image_id).await?;
        let url = deployment
            .url
            .ok_or(anyhow::anyhow!("Invalid deployment"))?;
        let resp = reqwest::get(url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download program: {:?}", e))?;
        Ok(resp
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download program: {:?}", e))?)
    }

    pub async fn get_deployment(&self, image_id: &str) -> Result<Option<Account>> {
        let (deployment_account, _) = deployment_address(image_id);
        let account = self
            .rpc_client
            .get_account_with_commitment(&deployment_account, CommitmentConfig::confirmed())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account: {:?}", e))?;
        Ok(account.value)
    }

    pub async fn get_fees(&self, signer: &Pubkey) -> Result<u64> {
        let fee_accounts = vec![signer.to_owned(), bonsol_interface::ID];
        let compute_fees = self
            .rpc_client
            .get_recent_prioritization_fees(&fee_accounts)
            .await?;
        Ok(if compute_fees.len() == 0 {
            5
        } else {
            compute_fees[0].prioritization_fee
        })
    }

    pub async fn deploy_v1(
        &self,
        signer: &Pubkey,
        image_id: &str,
        image_size: u64,
        program_name: &str,
        url: &str,
        inputs: Vec<ProgramInputType>,
    ) -> Result<Vec<Instruction>> {
        let compute_price_val = self.get_fees(signer).await?;
        let instruction =
            instructions::deploy_v1(signer, image_id, image_size, program_name, url, inputs)?;
        let compute = ComputeBudgetInstruction::set_compute_unit_limit(20_000);
        let compute_price = ComputeBudgetInstruction::set_compute_unit_price(compute_price_val);
        Ok(vec![compute, compute_price, instruction])
    }

    pub async fn execute_v1(
        &self,
        signer: &Pubkey,
        image_id: &str,
        execution_id: &str,
        inputs: Vec<InputT>,
        tip: u64,
        expiration: u64,
        config: ExecutionConfig,
        callback: Option<CallbackConfig>,
    ) -> Result<Vec<Instruction>> {
        let compute_price_val = self.get_fees(signer).await?;
        let instruction = instructions::execute_v1(
            signer,
            image_id,
            execution_id,
            inputs,
            tip,
            expiration,
            config,
            callback,
        )?;
        let compute = ComputeBudgetInstruction::set_compute_unit_limit(20_000);
        let compute_price = ComputeBudgetInstruction::set_compute_unit_price(compute_price_val);
        Ok(vec![compute, compute_price, instruction])
    }

    pub async fn send_txn_standard(
        &self,
        signer: impl Signer,
        instructions: Vec<Instruction>,
    ) -> Result<()> {
        self.send_txn(signer, instructions, false, 1, 5).await
    }

    pub async fn send_txn(
        &self,
        signer: impl Signer,
        instructions: Vec<Instruction>,
        skip_preflight: bool,
        retry_timeout: u64,
        retry_count: usize,
    ) -> Result<()> {
        let mut rt = retry_count;
        loop {
            let blockhash = self.rpc_client.get_latest_blockhash().await?;
            let message =
                v0::Message::try_compile(&signer.pubkey(), &instructions, &[], blockhash)?;
            let tx = VersionedTransaction::try_new(VersionedMessage::V0(message), &[&signer])?;
            let sig = self
                .rpc_client
                .send_transaction_with_config(
                    &tx,
                    RpcSendTransactionConfig {
                        skip_preflight,
                        preflight_commitment: Some(CommitmentLevel::Confirmed),
                        max_retries: Some(0),
                        ..Default::default()
                    },
                )
                .await?;

            let now = Instant::now();
            let confirm_transaction_initial_timeout = Duration::from_secs(retry_timeout);
            let (_, status) = loop {
                let status = self
                    .rpc_client
                    .get_signature_status_with_commitment(&sig, CommitmentConfig::processed())
                    .await?;
                if status.is_none() {
                    let blockhash_not_found = !self
                        .rpc_client
                        .is_blockhash_valid(&blockhash, CommitmentConfig::processed())
                        .await?;
                    if blockhash_not_found && now.elapsed() >= confirm_transaction_initial_timeout {
                        break (sig, status);
                    }
                } else {
                    break (sig, status);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            };

            match status {
                Some(Ok(())) => {
                    return Ok(());
                }
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!("Transaction Falure Cannot Recover {:?}", e));
                }
                None => {
                    rt -= 1;
                    if rt == 0 {
                        return Err(anyhow::anyhow!("Timeout: Failed to confirm transaction"));
                    }
                }
            }
        }
    }
}
