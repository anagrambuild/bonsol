pub mod util;
use serde::{Deserialize, Serialize, Serializer};    
use {
    anyhow::Result,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_rpc_client_api::config::RpcSendTransactionConfig,
    solana_sdk::{
        account::Account,
        commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction,
        instruction::{AccountMeta, Instruction},
        message::{v0, VersionedMessage},
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        transaction::VersionedTransaction,
    },
};

pub use anagram_bonsol_channel_utils::*;
pub use anagram_bonsol_schema::*;
use flatbuffers::FlatBufferBuilder;
pub mod input_resolver;
pub struct BonsolClient {
    rpc_client: RpcClient,
}

impl BonsolClient {
    pub fn new(rpc_url: String) -> Self {
        BonsolClient {
            rpc_client: RpcClient::new(rpc_url),
        }
    }

    pub fn with_rpc_client(rpc_client: RpcClient) -> Self {
        BonsolClient { rpc_client }
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

    pub async fn deploy_v1(
        &self,
        signer: &Pubkey,
        image_id: &str,
        image_size: u64,
        program_name: &str,
        url: &str,
        inputs: Vec<ProgramInputType>,
    ) -> Result<Vec<Instruction>> {
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
        let fee_accounts = vec![signer.to_owned()];
        let compute_fees = self
            .rpc_client
            .get_recent_prioritization_fees(&fee_accounts)
            .await?;
        let compute_price_val = if compute_fees.len() == 0 {
            5
        } else {
            compute_fees[0].prioritization_fee
        };
        let instruction =
            Instruction::new_with_bytes(anagram_bonsol_channel_utils::ID, ix_data, accounts);
        let compute = ComputeBudgetInstruction::set_compute_unit_limit(20_000);
        let compute_price = ComputeBudgetInstruction::set_compute_unit_price(compute_price_val);

        Ok(vec![compute, compute_price, instruction])
    }

    pub async fn send_txn(
        &self,
        signer: impl Signer,
        instructions: Vec<Instruction>,
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
                    skip_preflight: true,
                    max_retries: Some(0),
                    ..Default::default()
                },
            )
            .await?;
            let conf = self.rpc_client
                .confirm_transaction(&sig)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to confirm transaction: {:?}", e));
            match conf {
                Ok(true) => {
                    return Ok(());
                }
                Ok(false) => {
                    rt -= 1;
                    if rt == 0 {
                        return Err(anyhow::anyhow!("Failed to confirm transaction: max retries exceeded"));
                    }
                }
                Err(e) => {
                    rt -= 1;
                    if rt == 0 {
                        return Err(anyhow::anyhow!("Failed to confirm transaction: {:?}", e));
                    }
                }
            }
        }
    }
}

pub struct SdkInputType(InputType);

impl Serialize for SdkInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            InputType::PublicData => serializer.serialize_str("Public"),
            InputType::PublicAccountData => serializer.serialize_str("PublicAccountData"),
            InputType::PublicUrl => serializer.serialize_str("PublicUrl"),
            InputType::Private => serializer.serialize_str("Private"),
            InputType::InputSet => serializer.serialize_str("InputSet"),
            InputType::PublicProof => serializer.serialize_str("PublicProof"),
            _ => Err(serde::ser::Error::custom("Invalid input type")),
        }
    }
}

impl<'de> Deserialize<'de> for SdkInputType {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
      "Public" => Ok(SdkInputType(InputType::PublicData)),
      "PublicAccountData" => Ok(SdkInputType(InputType::PublicAccountData)),
      "PublicUrl" => Ok(SdkInputType(InputType::PublicUrl)),
      "Private" => Ok(SdkInputType(InputType::Private)),
      "InputSet" => Ok(SdkInputType(InputType::InputSet)),
      "PublicProof" => Ok(SdkInputType(InputType::PublicProof)),
      _ => Err(serde::de::Error::custom(format!("Invalid input type: {}", s))),
    }
  }
}