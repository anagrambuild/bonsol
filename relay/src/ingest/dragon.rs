use std::{collections::HashMap, time::Duration};

use crate::types::BonsolInstruction;

use {
    super::{Ingester, TxChannel},
    anyhow::Result,
    futures::stream::StreamExt,
    solana_sdk::{message::VersionedMessage, pubkey::Pubkey},
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::{
        convert_from::create_tx_with_meta,
        prelude::{
            subscribe_update::UpdateOneof, SubscribeRequest, SubscribeRequestFilterTransactions,
        },
    },
};
pub struct GrpcIngester {
    url: String,
    token: String,
    connection_timeout_secs: Option<u32>,
    timeout_secs: Option<u32>,
    op_handle: Option<tokio::task::JoinHandle<Result<()>>>,
}

impl GrpcIngester {
    pub fn new(
        url: String,
        token: String,
        connection_timeout_secs: Option<u32>,
        timeout_secs: Option<u32>,
    ) -> Self {
        GrpcIngester {
            url,
            token,
            connection_timeout_secs,
            timeout_secs,
            op_handle: None,
        }
    }
}

impl Ingester for GrpcIngester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel> {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();
        let stream_client = GeyserGrpcClient::build_from_shared(self.url.clone())?
            .x_token(Some(self.token.clone()))?
            .connect_timeout(Duration::from_secs(
                self.connection_timeout_secs.unwrap_or(10) as u64,
            ))
            .timeout(Duration::from_secs(self.timeout_secs.unwrap_or(10) as u64));
        self.op_handle = Some(tokio::spawn(async move {
            let mut client = stream_client.connect().await?;
            let mut txmap = HashMap::new();
            txmap.insert(
                program.to_string(),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    failed: Some(false),
                    account_required: vec![program.to_string()],
                    ..Default::default()
                },
            );
            let (_, mut stream) = client
                .subscribe_with_request(Some(SubscribeRequest {
                    transactions: txmap,
                    ..Default::default()
                }))
                .await?;
            while let Some(message) = stream.next().await {
                match message {
                    Ok(msg) => {
                        if let Some(UpdateOneof::Transaction(txw)) = msg.update_oneof {
                            if let Some(tx) = txw.transaction {
                                if let Ok(soltxn) = create_tx_with_meta(tx) {
                                    let acc = soltxn.account_keys();
                                    let txndata = soltxn.get_transaction();
                                    let meta = soltxn.get_status_meta();
                                    //unwrap so we can consume
                                    if let VersionedMessage::V0(msg) = txndata.message {
                                        let bonsolixs = msg
                                            .instructions
                                            .into_iter()
                                            .filter(|ix| {
                                                acc.get(ix.program_id_index as usize)
                                                    == Some(&program)
                                            })
                                            .map(|ix| BonsolInstruction {
                                                cpi: false,
                                                accounts: ix
                                                    .accounts
                                                    .into_iter()
                                                    .map(|idx| {
                                                        acc.get(idx as usize)
                                                            .map(|a| *a)
                                                            .unwrap_or_default()
                                                    })
                                                    .collect(),
                                                data: ix.data,
                                                last_known_block: txw.slot,
                                            })
                                            .collect::<Vec<BonsolInstruction>>();
                                        if bonsolixs.len() > 0 {
                                            match txchan.send(bonsolixs) {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    println!(
                                                        "Error sending to txn ingest channel: {:?}",
                                                        e
                                                    );
                                                }
                                            }
                                        }

                                        if let Some(metadata) = meta {
                                            if let Some(inner_ix) = metadata.inner_instructions {
                                                let ixs = inner_ix
                                                    .into_iter()
                                                    .flat_map(|ix| {
                                                        ix.instructions
                                                            .into_iter()
                                                            .filter(|ix| {
                                                                acc.get(
                                                                    ix.instruction.program_id_index
                                                                        as usize,
                                                                ) == Some(&program)
                                                            })
                                                            .map(|ix| BonsolInstruction {
                                                                cpi: true,
                                                                accounts: ix
                                                                    .instruction
                                                                    .accounts
                                                                    .into_iter()
                                                                    .map(|a| {
                                                                        acc.get(a as usize)
                                                                            .map(|a| *a)
                                                                            .unwrap_or_default()
                                                                    })
                                                                    .collect(),
                                                                data: ix.instruction.data,
                                                                last_known_block: txw.slot,
                                                            })
                                                            .collect::<Vec<BonsolInstruction>>()
                                                    })
                                                    .collect::<Vec<BonsolInstruction>>();
                                                if ixs.len() > 0 {
                                                    match txchan.send(ixs) {
                                                        Ok(_) => {}
                                                        Err(e) => {
                                                            println!(
                                                                "Error sending to txn ingest channel: {:?}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        println!("Error in stream");
                    }
                }
            }
            return Ok(());
        }));
        Ok(rx)
    }

    fn stop(&mut self) -> Result<()> {
        self.op_handle.as_mut().map(|t| t.abort());
        Ok(())
    }
}
