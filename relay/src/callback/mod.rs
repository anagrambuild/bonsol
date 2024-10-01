use std::sync::Arc;

use {
    async_trait::async_trait,
    bonsol_channel_utils::{deployment_address, execution_address, execution_claim_address},
    bonsol_schema::{
        ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, ClaimV1, ClaimV1Args,
        StatusTypes, StatusV1, StatusV1Args,
    },
    dashmap::DashMap,
    flatbuffers::FlatBufferBuilder,
    itertools::Itertools,
    solana_rpc_client_api::config::RpcSendTransactionConfig,
    solana_sdk::{
        account::Account,
        commitment_config::CommitmentConfig,
        message::{v0, VersionedMessage},
        signature::Signature,
        signer::SignerError,
        system_program,
        transaction::VersionedTransaction,
    },
    solana_transaction_status::TransactionStatus,
    tokio::task::JoinHandle,
};

use {
    crate::{observe::MetricEvents, types::ProgramExec},
    anyhow::Result,
    metrics::gauge,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    },
    tracing::info,
};

#[async_trait]
pub trait TransactionSender {
    fn start(&mut self);
    fn stop(&mut self);
    fn sign_calldata(&self, data: &str) -> Result<String>;
    async fn claim(
        &self,
        execution_id: &str,
        requester: Pubkey,
        execution_account: Pubkey,
        block_commitment: u64,
    ) -> Result<Signature>;
    async fn submit_proof(
        &self,
        execution_id: &str,
        requester_account: Pubkey,
        callback_exec: Option<ProgramExec>,
        proof: &[u8],
        execution_digest: &[u8],
        input_digest: &[u8],
        assumption_digest: &[u8],
        committed_outputs: &[u8],
        additional_accounts: Vec<AccountMeta>,
        exit_code_system: u32,
        exit_code_user: u32,
    ) -> Result<Signature>;
    async fn get_current_block(&self) -> Result<u64>;
    fn get_signature_status(&self, sig: &Signature) -> Option<TransactionStatus>;
    fn clear_signature_status(&self, sig: &Signature);
    async fn get_deployment_account(&self, image_id: &str) -> Result<Account>;
}

pub struct RpcTransactionSender {
    pub rpc_client: Arc<RpcClient>,
    pub bonsol_program: Pubkey,
    pub signer: Keypair,
    pub txn_status_handle: Option<JoinHandle<()>>,
    pub sigs: Arc<DashMap<Signature, Option<TransactionStatus>>>,
}

impl Signer for RpcTransactionSender {
    fn pubkey(&self) -> Pubkey {
        self.signer.pubkey()
    }

    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        Ok(self.signer.pubkey())
    }

    fn sign_message(&self, message: &[u8]) -> Signature {
        self.signer.sign_message(message)
    }

    fn try_sign_message(
        &self,
        message: &[u8],
    ) -> std::result::Result<Signature, solana_sdk::signer::SignerError> {
        self.signer.try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        false
    }
}

impl RpcTransactionSender {
    pub fn new(rpc_url: String, bonsol_program: Pubkey, signer: Keypair) -> Self {
        Self {
            rpc_client: Arc::new(RpcClient::new(rpc_url)),
            signer: signer,
            bonsol_program,
            txn_status_handle: None,
            sigs: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl TransactionSender for RpcTransactionSender {
    fn sign_calldata(&self, data: &str) -> Result<String> {
        let sig = self.signer.sign_message(data.as_bytes());
        Ok(sig.to_string())
    }

    fn get_signature_status(&self, sig: &Signature) -> Option<TransactionStatus> {
        if let Some(status) = self.sigs.get(sig) {
            status.clone()
        } else {
            None
        }
    }

    fn clear_signature_status(&self, sig: &Signature) {
        self.sigs.remove(sig);
    }

    async fn claim(
        &self,
        execution_id: &str,
        requester: Pubkey,
        execution_account: Pubkey,
        block_commitment: u64,
    ) -> Result<Signature> {
        let (execution_claim_account, _) = execution_claim_address(&execution_account.as_ref());
        eprintln!("{:?}", execution_account);
        let accounts = vec![
            AccountMeta::new(execution_account, false),
            AccountMeta::new_readonly(requester, false),
            AccountMeta::new(execution_claim_account, false),
            AccountMeta::new(self.signer.pubkey(), true),
            AccountMeta::new(self.signer.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ];
        let mut fbb = FlatBufferBuilder::new();
        let eid = fbb.create_string(execution_id);
        let stat = ClaimV1::create(
            &mut fbb,
            &ClaimV1Args {
                block_commitment,
                execution_id: Some(eid),
            },
        );
        fbb.finish(stat, None);
        let statbytes = fbb.finished_data();
        let mut fbb2 = FlatBufferBuilder::new();
        let off = fbb2.create_vector(statbytes);
        let root = ChannelInstruction::create(
            &mut fbb2,
            &ChannelInstructionArgs {
                ix_type: ChannelInstructionIxType::ClaimV1,
                claim_v1: Some(off),
                ..Default::default()
            },
        );
        fbb2.finish(root, None);
        let ix_data = fbb2.finished_data();
        let instruction = Instruction::new_with_bytes(self.bonsol_program, &ix_data, accounts);
        let blockhash_req = self.rpc_client.get_latest_blockhash().await;
        let blockhash = match blockhash_req {
            Ok(blockhash) => blockhash,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get blockhash: {:?}", e));
            }
        };
        let msg = v0::Message::try_compile(&self.signer.pubkey(), &[instruction], &[], blockhash)?;
        let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&self.signer])?;
        let sig = self
            .rpc_client
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        self.sigs.insert(sig, None);
        Ok(sig)
    }

    async fn submit_proof(
        &self,
        execution_id: &str,
        requester_account: Pubkey,
        callback_exec: Option<ProgramExec>,
        proof: &[u8],
        execution_digest: &[u8],
        input_digest: &[u8],
        assumption_digest: &[u8],
        committed_outputs: &[u8],
        additional_accounts: Vec<AccountMeta>,
        exit_code_system: u32,
        exit_code_user: u32,
    ) -> Result<Signature> {
        let (execution_request_data_account, _) =
            execution_address(&requester_account, &execution_id.as_bytes());
        let (id, additional_accounts) = match callback_exec {
            None => (self.bonsol_program, vec![]),
            Some(pe) => {
                let prog = pe.program_id;
                //todo: add read interface simulation on program to get other accounts
                (prog, additional_accounts)
            }
        };

        let mut accounts = vec![
            AccountMeta::new(requester_account, false),
            AccountMeta::new(execution_request_data_account, false),
            AccountMeta::new_readonly(id, false),
            AccountMeta::new(self.signer.pubkey(), true),
        ];
        accounts.extend(additional_accounts);
        let mut fbb = FlatBufferBuilder::new();
        let proof_vec = fbb.create_vector(proof);
        let execution_digest = fbb.create_vector(execution_digest);
        let input_digest = fbb.create_vector(input_digest);
        let assumption_digest = fbb.create_vector(assumption_digest);
        let eid = fbb.create_string(execution_id);
        let out = fbb.create_vector(committed_outputs);
        let stat = StatusV1::create(
            &mut fbb,
            &StatusV1Args {
                execution_id: Some(eid),                    //0-?? bytes lets say 16
                status: StatusTypes::Completed,             //1 byte
                proof: Some(proof_vec),                     //256 bytes
                execution_digest: Some(execution_digest),   //32 bytes
                input_digest: Some(input_digest),           //32 bytes
                assumption_digest: Some(assumption_digest), //32 bytes
                committed_outputs: Some(out),               //0-?? bytes lets say 32
                exit_code_system,                           //4 byte
                exit_code_user,                             //4 byte
            }, //total ~408 bytes plenty of room for more stuff
        );
        fbb.finish(stat, None);
        let statbytes = fbb.finished_data();
        let mut fbb2 = FlatBufferBuilder::new();
        let off = fbb2.create_vector(statbytes);
        let root = ChannelInstruction::create(
            &mut fbb2,
            &ChannelInstructionArgs {
                ix_type: ChannelInstructionIxType::StatusV1,
                status_v1: Some(off),
                ..Default::default()
            },
        );
        fbb2.finish(root, None);
        let ix_data = fbb2.finished_data();
        let instruction = Instruction::new_with_bytes(self.bonsol_program, &ix_data, accounts);
        let blockhash_req = self.rpc_client.get_latest_blockhash().await;
        let blockhash = match blockhash_req {
            Ok(blockhash) => blockhash,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to get blockhash: {:?}", e));
            }
        };
        let msg = v0::Message::try_compile(&self.signer.pubkey(), &[instruction], &[], blockhash)?;
        let tx = VersionedTransaction::try_new(VersionedMessage::V0(msg), &[&self.signer])?;

        let sig = self
            .rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &tx,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))?;
        self.sigs.insert(sig, None);
        Ok(sig)
    }

    fn start(&mut self) {
        let sigs = self.sigs.clone();
        let rpc_client = self.rpc_client.clone();
        self.txn_status_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            let sigs = sigs.clone();
            let mut sigs_num = sigs.len();
            loop {
                interval.tick().await;
                let sigs = sigs.clone();
                if sigs.len() != sigs_num {
                    emit_gauge!(MetricEvents::SignaturesInFlight, sigs.len() as f64, sent => "s");
                }
                let all_sigs = sigs.iter().map(|x| x.key().clone()).collect_vec();
                let statuses = rpc_client.get_signature_statuses(&all_sigs).await;
                if let Ok(statuses) = statuses {
                    for sig in all_sigs.into_iter().zip(statuses.value.into_iter()) {
                        sigs.insert(sig.0, sig.1);
                    }
                }
                sigs_num = sigs.len();
            }
        }));
    }

    fn stop(&mut self) {
        if let Some(handle) = self.txn_status_handle.take() {
            handle.abort();
        }
    }

    async fn get_current_block(&self) -> Result<u64> {
        self.rpc_client
            .get_block_height()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))
    }

    async fn get_deployment_account(&self, image_id: &str) -> Result<Account> {
        let (deployment_account, _) = deployment_address(image_id);
        self.rpc_client
            .get_account(&deployment_account)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get account: {:?}", e))
    }
}
