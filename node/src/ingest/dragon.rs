use std::{collections::HashMap, time::Duration};

use {
    anyhow::anyhow,
    solana_sdk::{message::AccountKeys, transaction::VersionedTransaction},
    solana_transaction_status::TransactionStatusMeta,
    yellowstone_grpc_proto::geyser::SubscribeUpdate,
};

use crate::types::BonsolInstruction;

use {
    super::{Ingester, TxChannel},
    anyhow::Result,
    futures::stream::StreamExt,
    solana_sdk::{message::VersionedMessage, pubkey::Pubkey},
    tokio::sync::mpsc::UnboundedSender,
    yellowstone_grpc_client::{GeyserGrpcBuilder, GeyserGrpcClient},
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
            ingest(program, txchan, stream_client).await
        }));
        Ok(rx)
    }

    fn stop(&mut self) -> Result<()> {
        self.op_handle.as_mut().map(|t| t.abort());
        Ok(())
    }
}

async fn ingest(
    program: Pubkey,
    txchan: UnboundedSender<Vec<BonsolInstruction>>,
    stream_client: GeyserGrpcBuilder,
) -> Result<()> {
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
                if let Err(e) = handle_msg(msg, program, &txchan) {
                    eprintln!("Error in stream: {e:?}")
                }
            }
            Err(e) => eprintln!("Error in stream: {e:?}"),
        }
    }
    Ok(())
}

fn handle_msg(
    msg: SubscribeUpdate,
    program: Pubkey,
    txchan: &UnboundedSender<Vec<BonsolInstruction>>,
) -> Result<()> {
    if let Some(UpdateOneof::Transaction(txw)) = msg.update_oneof {
        txw.transaction.map(|tx| -> Result<()> {
            create_tx_with_meta(tx)
                .map(|soltxn| {
                    try_send_instructions(
                        program,
                        txw.slot,
                        soltxn.account_keys(),
                        soltxn.get_transaction(),
                        soltxn.get_status_meta(),
                        &txchan,
                    )
                })
                .map_err(|e| anyhow!("error while sending instructions: {e}"))?
        });
    }
    Ok(())
}

fn try_send_instructions(
    program: Pubkey,
    last_known_block: u64,
    acc: AccountKeys,
    txndata: VersionedTransaction,
    meta: Option<TransactionStatusMeta>,
    txchan: &UnboundedSender<Vec<BonsolInstruction>>,
) -> Result<()> {
    if let VersionedMessage::V0(msg) = txndata.message {
        let bonsolixs = msg
            .instructions
            .into_iter()
            .filter_map(|ix| {
                if acc
                    .get(ix.program_id_index as usize)
                    .is_some_and(|p| p == &program)
                {
                    return Some(BonsolInstruction::new(
                        false,
                        ix.accounts
                            .into_iter()
                            .filter_map(|idx| acc.get(idx as usize).copied())
                            .collect(),
                        ix.data,
                        last_known_block,
                    ));
                }
                None
            })
            .collect::<Vec<BonsolInstruction>>();
        if !bonsolixs.is_empty() {
            txchan.send(bonsolixs).map_err(|e| {
                anyhow!(
                    "failed to send instructions to txn ingest channel: {:?}",
                    e.0
                )
            })?
        }
        if let Some(metadata) = meta {
            if let Some(inner_ix) = metadata.inner_instructions {
                let ixs = inner_ix
                    .into_iter()
                    .flat_map(|ix| {
                        ix.instructions.into_iter().filter_map(|ix| {
                            if acc
                                .get(ix.instruction.program_id_index as usize)
                                .is_some_and(|p| p == &program)
                            {
                                return Some(BonsolInstruction::new(
                                    true,
                                    ix.instruction
                                        .accounts
                                        .into_iter()
                                        .filter_map(|a| acc.get(a as usize).copied())
                                        .collect(),
                                    ix.instruction.data,
                                    last_known_block,
                                ));
                            }
                            None
                        })
                    })
                    .collect::<Vec<BonsolInstruction>>();
                if !ixs.is_empty() {
                    txchan.send(ixs).map_err(|e| {
                        anyhow!(
                            "failed to send instructions to txn ingest channel: {:?}",
                            e.0
                        )
                    })?
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod dragon_ingester_tests {
    use solana_sdk::pubkey::Pubkey;

    use super::{GrpcIngester, Ingester};

    #[tokio::test]
    async fn ingester_starts_and_stops() {
        let mut ingester = GrpcIngester::new(
            "http://localhost:8899".into(),
            "test".into(),
            Some(5),
            Some(5),
        );
        assert!(ingester.start(Pubkey::new_unique()).is_ok());
        assert!(ingester.stop().is_ok());
    }
}
