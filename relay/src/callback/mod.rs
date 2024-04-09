use {
    anagram_bonsol_channel::{execution_address, execution_claim_address},
    anagram_bonsol_schema::{
        ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, ClaimV1, ClaimV1Args,
        StatusTypes, StatusV1, StatusV1Args,
    },
    flatbuffers::FlatBufferBuilder,
    solana_rpc_client_api::config::RpcSendTransactionConfig,
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signature, system_program},
};

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
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
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
        execution_digest: &[u8],
        input_digest: &[u8],
        output_digest: &[u8],
        committed_outputs: Option<&[u8]>,
        exit_code_system: u32,
        exit_code_user: u32,
    ) -> Result<Signature> {
        let (execution_request_data_account, _) =
            execution_address(&requester_account, &execution_id.as_bytes());
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
        let execution_digest = fbb.create_vector(execution_digest);
        let input_digest = fbb.create_vector(input_digest);
        let output_digest = fbb.create_vector(output_digest);
        let eid = fbb.create_string(execution_id);
        let out = match committed_outputs {
            None => None,
            Some(o) => Some(fbb.create_vector(o)),
        };
        let stat = StatusV1::create(
            &mut fbb,
            &StatusV1Args {
                execution_id: Some(eid),                  //0-?? bytes lets say 16
                status: StatusTypes::Completed,           //1 byte
                proof: Some(proof_vec),                   //256 bytes
                execution_digest: Some(execution_digest), //32 bytes
                input_digest: Some(input_digest),         //32 bytes
                output_digest: Some(output_digest),       //32 bytes
                committed_outputs: out,                   //0-?? bytes lets say 32
                exit_code_system,                         //4 byte
                exit_code_user,                           //4 byte
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
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.signer.pubkey()),
            &[&self.signer],
            blockhash,
        );

        self.rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &tx,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: true,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {:?}", e))
    }
}
