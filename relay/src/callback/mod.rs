use std::{f64::consts::E, str::FromStr, sync::Arc};

use anagram_bonsol_channel::{execution_address, execution_claim_address};
use anagram_bonsol_schema::{
    ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, ClaimV1, ClaimV1Args,
    StatusTypes, StatusV1, StatusV1Args,
};
use flatbuffers::FlatBufferBuilder;
use solana_rpc_client_api::{config::RpcSendTransactionConfig, request};
use solana_sdk::{address_lookup_table::instruction, commitment_config::CommitmentConfig, signature::Signature, system_instruction::transfer, system_program};

use {
    anyhow::Result,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    },
    tokio::{
        sync::{
            mpsc::{unbounded_channel, UnboundedSender},
            Semaphore,
        },
        task::JoinHandle,
    },
};

use crate::types::ProgramExec;
const RPC_PERMITS: usize = 200;
pub struct TransactionSender {
    pub rpc_client: RpcClient,
    pub bonsol_program: Pubkey,
    pub signer: Keypair,
}

impl TransactionSender {
    pub fn new(rpc_url: String, bonsol_program: Pubkey, signer: Keypair) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
            signer: signer,
            bonsol_program,
        }
    }

    pub fn sign_calldata(&self, data: &str) -> Result<String> {
        let sig = self.signer.sign_message(data.as_bytes());
        Ok(sig.to_string())
    }

    pub async fn claim(
        &self,
        execution_id: &str,
        execution_account: Pubkey,
        block_commitment: u64,
    ) -> Result<Signature> {
        let (execution_claim_account, _) = execution_claim_address(&execution_id.as_bytes());
        eprintln!("{:?}", execution_account);
        let accounts = vec![
            AccountMeta::new(execution_account, false),
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
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.signer.pubkey()),
            &[&self.signer],
            blockhash,
        );

        self.rpc_client
            .send_transaction_with_config(&tx,
                RpcSendTransactionConfig{
                    skip_preflight: true,
                    ..Default::default()
                }
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))
    }

    pub async fn submit_proof(
        &self,
        execution_id: &str,
        requester_account: Pubkey,
        callback_exec: Option<ProgramExec>,
        proof: &[u8],
        inputs: &[u8],
        input_digest: &str,
    ) -> Result<Signature> {
        let (execution_request_data_account, _) = execution_address(&requester_account, &execution_id.as_bytes());
        let (id, additional_accounts) = match callback_exec {
            None => (self.bonsol_program, vec![]),
            Some(pe) => {
                let prog = pe.program_id;
                //todo: call simulation on program to get other accounts
                (prog, vec![])
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
        let inputs_vec = fbb.create_vector(inputs);
        let digest = fbb.create_vector(input_digest.as_bytes());
        let eid = fbb.create_string(execution_id);
        let stat = StatusV1::create(
            &mut fbb,
            &StatusV1Args {
                execution_id: Some(eid),
                status: StatusTypes::Completed,
                proof: Some(proof_vec),
                inputs: Some(inputs_vec),
                input_digest: Some(digest),
            },
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
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.signer.pubkey()),
            &[&self.signer],
            blockhash,
        );

        self.rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(&tx,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig{
                    skip_preflight: true,
                    ..Default::default()
                }
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))
    }
}
