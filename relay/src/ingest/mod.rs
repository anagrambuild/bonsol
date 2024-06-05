use tokio::sync::mpsc::UnboundedSender;
use std::convert::TryInto;

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
        EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiTransactionEncoding
    },
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
    },
    tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle},
};

// use solana_sdk::{pubkey::Pubkey, transaction::VersionedTransaction};

use {
    std::{collections::HashMap, time::Duration},
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::prelude::{
        subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
        SubscribeRequestFilterBlocks
    },
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

pub struct GrpcIngester {
    grpc_url: String,
    op_handle: Option<JoinHandle<()>>,
}

impl GrpcIngester {
    pub fn new(grpc_url: String) -> GrpcIngester {
        GrpcIngester {
            op_handle: None,
            grpc_url,
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
                                            cpi: false,
                                            accounts: instruction
                                                .accounts
                                                .iter()
                                                .map(|a| scc[*a as usize])
                                                .collect(),
                                            data,
                                            last_known_block,
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
    eprintln!("Subscribed to {}", rpc_url);
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
                    eprintln!("Error in ingester: {:?} retrying ", e);
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

fn to_fixed<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

// todo find a way to consume without clone
fn filter_txs2(
    program: &Pubkey,
    last_known_block: u64,
    tx: yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo,
) -> Vec<BonsolInstruction> {
    let program_bytes = program.to_bytes();
    let mut res = vec![];
    if let Some(dtx) = tx.transaction {
        let msg = dtx.message.unwrap();
        let scc = msg.account_keys;
        if let Some(meta) = tx.meta {
            if meta.err.is_some() {
                return res;
            }
            for ix in msg.instructions.iter() {

                if scc[ix.program_id_index as usize] == program_bytes {
                    res.push(BonsolInstruction {
                        cpi: false,
                        accounts: ix.accounts.iter().map(|a| Pubkey::from(to_fixed::<u8, 32>(scc[*a as usize].clone()))).collect(),
                        data: ix.data.clone(),
                        last_known_block,
                    });
                }
            }
            let inner_ix_groups: Vec<yellowstone_grpc_proto::prelude::InnerInstructions> = meta.inner_instructions;
            for group in inner_ix_groups {
                for ix in group.instructions {
                    if scc[ix.program_id_index as usize] == program_bytes {
                        let data = bs58::decode(&ix.data).into_vec();
                        if let Ok(data) = data {
                            res.push(BonsolInstruction {
                                cpi: false,
                                accounts: ix
                                    .accounts
                                    .iter()
                                    .map(|a| Pubkey::from(to_fixed::<u8, 32>(scc[*a as usize].clone())))
                                    .collect(),
                                data,
                                last_known_block,
                            });
                        } else {
                            println!(
                                "Failed to decode bs58 data for bonsol instruction"
                            );
                        }
                    }
                }
            }
        }
    }
    res
}

impl Ingester for GrpcIngester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel> {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();
        let grpc_url = self.grpc_url.clone();
        self.op_handle = Some(tokio::spawn(async move {
            loop {
                let mut client = GeyserGrpcClient::build_from_shared(grpc_url.clone())
                .ok()
                .unwrap()
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(10))
                .connect()
                .await
                .ok()
                .unwrap();

                let subscription = client.subscribe_with_request(Some(SubscribeRequest {
                    blocks: HashMap::from_iter(vec![ ("blocks".to_string(), SubscribeRequestFilterBlocks{
                        account_include: vec![ program.to_string() ],
                        include_transactions: Some(true),
                        include_accounts: Some(false),
                        include_entries: Some(false)
                    }) ]),
                    commitment: Some(CommitmentLevel::Confirmed.into()),
                    ..Default::default()
                })).await;
                let (grpc_tx, mut grpc_rx) = subscription.unwrap();
                eprintln!("Subscribed to {}", grpc_url);

                while let Some(message) = grpc_rx.next().await {
                    match message {
                        Ok(msg) => match msg.update_oneof {
                            Some(UpdateOneof::Block(b)) => {
                                let height = b.block_height.unwrap().block_height;
                                let ix = b.transactions.iter().map(|tx| {
                                    filter_txs2(&program, height, tx.clone())
                                }).flatten().collect::<Vec<BonsolInstruction>>();
                                txchan.send(ix).unwrap();
                            },
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    }
                }
            }
        }));
        Ok(rx)
    }

    fn stop(&mut self) -> Result<()> {
        todo!()
    }
}
