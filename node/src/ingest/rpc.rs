use {
    super::{Ingester, IngesterResult, TxChannel},
    crate::{
        ingest::{IngestError, IngestErrorType},
        types::BonsolInstruction,
    },
    anyhow::Result,
    solana_pubsub_client::nonblocking::pubsub_client::PubsubClient,
    solana_rpc_client_api::config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
    solana_sdk::{bs58, commitment_config::CommitmentConfig, pubkey::Pubkey},
    solana_transaction_status::{
        EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiTransactionEncoding,
    },
    tokio::{sync::mpsc::UnboundedSender, task::JoinHandle}, tracing::error,
};

use futures_util::StreamExt;

pub struct RpcIngester {
    rpc_url: String,
    op_handle: Option<JoinHandle<()>>,
}

impl RpcIngester {
    pub fn new(rpc_url: String) -> RpcIngester {
        RpcIngester {
            op_handle: None,
            rpc_url,
        }
    }
}

// todo find a way to consume without clone
fn filter_txs(
    program: &Pubkey,
    last_known_block: u64,
    tx: EncodedTransactionWithStatusMeta,
) -> Vec<BonsolInstruction> {
    let mut res = vec![];

    if let Some(dtx) = tx.transaction.decode() {
        let scc = dtx.message.static_account_keys();
        if let Some(meta) = tx.meta {
            if meta.err.is_some() {
                return res;
            }
            for ix in dtx.message.instructions().iter() {
                if ix.program_id(scc) == program {
                    res.push(BonsolInstruction {
                        cpi: false,
                        accounts: ix.accounts.iter().map(|a| scc[*a as usize]).collect(),
                        data: ix.data.clone(),
                        last_known_block,
                    });
                }
            }
            let o_ix_groups: Option<Vec<UiInnerInstructions>> = meta.inner_instructions.into();
            if let Some(inner_ix_groups) = o_ix_groups {
                for group in inner_ix_groups {
                    for ix in group.instructions {
                        match ix {
                            UiInstruction::Compiled(instruction) => {
                                if &scc[instruction.program_id_index as usize] == program {
                                    let data = bs58::decode(&instruction.data).into_vec();
                                    if let Ok(data) = data {
                                        res.push(BonsolInstruction {
                                            cpi: true,
                                            accounts: instruction
                                                .accounts
                                                .iter()
                                                .map(|a| scc[*a as usize])
                                                .collect(),
                                            data,
                                            last_known_block,
                                        });
                                    } else {
                                        error!("Failed to decode bs58 data for bonsol instruction");
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    res
}

async fn ingest(
    rpc_url: String,
    program: Pubkey,
    txchan: UnboundedSender<Vec<BonsolInstruction>>,
) -> IngesterResult {
    let c = PubsubClient::new(&rpc_url).await.map_err(|e| IngestError {
        code: IngestErrorType::RpcError,
        message: e.to_string(),
    })?;

    let (mut stream, _unsub) = c
        .block_subscribe(
            RpcBlockSubscribeFilter::MentionsAccountOrProgram(program.to_string()),
            Some(RpcBlockSubscribeConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                max_supported_transaction_version: Some(0),
                show_rewards: Some(false),
                commitment: Some(CommitmentConfig::confirmed()),
                transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
            }),
        )
        .await
        .map_err(|e| IngestError {
            code: IngestErrorType::RpcError,
            message: e.to_string(),
        })?;
    while let Some(msg) = stream.next().await {
        if let Some(blk) = msg.value.block {
            if let Some(txs) = blk.transactions {
                let ix = txs
                    .into_iter()
                    .map::<Vec<BonsolInstruction>, _>(|tx| {
                        filter_txs(&program, blk.block_height.unwrap_or(blk.parent_slot), tx)
                    })
                    .flatten()
                    .collect::<Vec<BonsolInstruction>>();
                txchan.send(ix).unwrap();
            }
        }
    }
    Ok(())
}

impl Ingester for RpcIngester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel> {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();
        let rpc_url = self.rpc_url.clone();
        self.op_handle = Some(tokio::spawn(async move {
            let mut retry = 10;
            loop {
                let res = ingest(rpc_url.clone(), program, txchan.clone()).await;
                if let Err(e) = res {
                    error!("Error in ingester: {:?} retrying ", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    if retry == 0 {
                        break;
                    }
                    retry -= 1;
                }
            }
        }));
        Ok(rx)
    }

    fn stop(&mut self) -> Result<()> {
        self.op_handle.as_mut().map(|t| t.abort());
        Ok(())
    }
}
