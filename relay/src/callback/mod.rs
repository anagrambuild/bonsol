use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use tokio::{sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Semaphore,
}, task::JoinHandle};

use crate::types::CallbackInstruction;
const RPC_PERMITS: usize = 200;
pub struct RpcCallback {
    pub rpc_url: String,
    pub bonsol_program: String,
    pub signer: Arc<Keypair>,
    pub worker_handle: Option<JoinHandle<Result<()>>>,
}

impl RpcCallback {
    pub fn new(rpc_url: String, bonsol_program: String, signer: Keypair) -> Self {
        Self {
            rpc_url: rpc_url,
            signer: Arc::new(signer),
            bonsol_program,
            worker_handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<UnboundedSender<CallbackInstruction>> {
        let (tx, mut rx) = unbounded_channel::<CallbackInstruction>();
        let sem = Arc::new(Semaphore::new(RPC_PERMITS));
        let prog_id = Arc::new(Pubkey::from_str(&self.bonsol_program)?);
        let client = Arc::new(RpcClient::new(self.rpc_url.clone()));
        let sig = self.signer.clone();
        self.worker_handle = Some(tokio::spawn(async move {
            while let Some(cix) = rx.recv().await {
                let client = client.clone();
                let prog = prog_id.clone();
                let signer = sig.clone();
                let sem = sem.clone();
                if let Some(ix_data) = cix.ix_data {
                    tokio::spawn(async move {
                        let _permit = sem.acquire().await.unwrap();
                        let (id, additional_accounts) = match cix.program_exec {
                            None => (prog.as_ref().to_owned(), vec![]),
                            Some(pe) => {
                                let prog = pe.program_id;
                                //todo: call simulation on program to get other accounts
                                (prog, vec![])
                            }
                        };
                        
                        let mut accounts = vec![
                            AccountMeta::new_readonly(cix.requester_account, false),
                            AccountMeta::new(cix.execution_request_data_account, false),
                            AccountMeta::new_readonly(id, false),
                            AccountMeta::new(signer.pubkey(), true),
                        ];
                        accounts.extend(additional_accounts);
                        let instruction =
                            Instruction::new_with_bytes(prog.as_ref().clone(), &ix_data, accounts);
                        let blockhash_req = client.get_latest_blockhash().await;
                        let blockhash = match blockhash_req {
                            Ok(blockhash) => blockhash,
                            Err(e) => {
                                println!("Failed to get blockhash: {:?}", e);
                                return;
                            }
                        };
                        let tx = Transaction::new_signed_with_payer(
                            &[instruction],
                            Some(&signer.pubkey()),
                            &[&signer],
                            blockhash,
                        );

                        match client.send_and_confirm_transaction(&tx).await {
                            Ok(_) => {
                                println!("Transaction sent successfully");
                            }
                            Err(e) => {
                                println!("Failed to send transaction: {:?}", e);
                            }
                        }
                    });
                } else {
                    println!("Empty instruction data, skipping");
                }
            }
            Ok(())
        }));

        Ok(tx)
    }
}
