use {
    anyhow::Result,
    futures_util::StreamExt,
    solana_pubsub_client::nonblocking::pubsub_client::PubsubClient,
    solana_rpc_client_api::config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
    solana_sdk::{bs58, commitment_config::CommitmentConfig},
};

use {
    solana_sdk::pubkey::Pubkey,
    solana_transaction_status::{
        EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiTransactionEncoding,
    },
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
    },
    tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle},
};

use crate::types::BonsolInstruction;
pub type TxChannel = UnboundedReceiver<Vec<BonsolInstruction>>;
#[derive(Debug)]
pub enum IngestErrorType {
    RpcError,
    IoError,
}
#[derive(Debug)]
pub struct IngestError {
    pub code: IngestErrorType,
    pub message: String,
}
impl Display for IngestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "IngestError: {:?} - {:?}", self.code, self.message)
    }
}
impl Error for IngestError {}

pub type IngesterResult = Result<(), IngestError>;
pub trait Ingester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel>;

    fn stop(&mut self) -> Result<()>;
}

pub struct RpcIngester {
    rpc_url: String,
    op_handle: Option<JoinHandle<IngesterResult>>,
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
fn filter_txs(program: &Pubkey, tx: EncodedTransactionWithStatusMeta) -> Vec<BonsolInstruction> {
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
                                            cpi: false,
                                            accounts: instruction
                                                .accounts
                                                .iter()
                                                .map(|a| scc[*a as usize])
                                                .collect(),
                                            data,
                                        });
                                    } else {
                                        println!(
                                            "Failed to decode bs58 data for bonsol instruction"
                                        );
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

impl Ingester for RpcIngester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel> {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();
        let rpc_url = self.rpc_url.clone();
        self.op_handle = Some(tokio::spawn(async move {
            let c = PubsubClient::new(&rpc_url)
                .await
                .map_err(|e| IngestError {
                    code: IngestErrorType::RpcError,
                    message: e.to_string(),
                })
                .unwrap();

            let (mut stream, _unsub) = c
                .block_subscribe(
                    RpcBlockSubscribeFilter::MentionsAccountOrProgram(program.to_string()),
                    Some(RpcBlockSubscribeConfig {
                        encoding: Some(UiTransactionEncoding::Base64),
                        max_supported_transaction_version: Some(0),
                        show_rewards: Some(false),
                        commitment: Some(CommitmentConfig::confirmed()),
                        transaction_details: Some(
                            solana_transaction_status::TransactionDetails::Full,
                        ),
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
                            .map::<Vec<BonsolInstruction>, _>(|tx| filter_txs(&program, tx))
                            .flatten()
                            .collect::<Vec<BonsolInstruction>>();
                        txchan.send(ix).unwrap();
                    }
                }
            }
            Ok(())
        }));
        Ok(rx)
    }

    fn stop(&mut self) -> Result<()> {
        self.op_handle.as_mut().map(|t| t.abort());
        Ok(())
    }
}
