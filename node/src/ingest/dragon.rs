use std::{collections::HashMap, time::Duration};

use {
    anyhow::anyhow,
    solana_sdk::{message::AccountKeys, transaction::VersionedTransaction},
    solana_transaction_status::TransactionStatusMeta,
    yellowstone_grpc_proto::geyser::SubscribeUpdate,
};

use crate::types::{filter_bonsol_instructions, BonsolInstruction};

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
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl<'a> TryFrom<&'a mut GrpcIngester> for GeyserGrpcBuilder {
    type Error = anyhow::Error;
    fn try_from(value: &'a mut GrpcIngester) -> Result<Self, Self::Error> {
        Ok(GeyserGrpcClient::build_from_shared(value.url.clone())?
            .x_token(Some(value.token.clone()))?
            .connect_timeout(Duration::from_secs(
                value.connection_timeout_secs.unwrap_or(10) as u64,
            ))
            .timeout(Duration::from_secs(value.timeout_secs.unwrap_or(10) as u64)))
    }
}

impl Ingester for GrpcIngester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel> {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();
        let stream_client = GeyserGrpcBuilder::try_from(&mut *self)?;
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
    let program_filter = |acc: &AccountKeys, program: &Pubkey, index: usize| -> bool {
        acc.get(index).is_some_and(|p| p == program)
    };

    if let VersionedMessage::V0(msg) = txndata.message {
        let mut bonsolixs: Vec<BonsolInstruction> = filter_bonsol_instructions(
            msg.instructions,
            &acc,
            &program,
            last_known_block,
            program_filter,
        )
        .collect();
        if let Some(metadata) = meta {
            if let Some(inner_ix) = metadata.inner_instructions {
                bonsolixs.extend(inner_ix.into_iter().flat_map(|ix| {
                    filter_bonsol_instructions(
                        ix.instructions,
                        &acc,
                        &program,
                        last_known_block,
                        program_filter,
                    )
                }));
            }
        }
        if !bonsolixs.is_empty() {
            txchan.send(bonsolixs).map_err(|e| {
                anyhow!(
                    "failed to send instructions to txn ingest channel: {:?}",
                    e.0
                )
            })?
        }
    }
    Ok(())
}

#[cfg(test)]
mod dragon_ingester_tests {
    use {
        expect_test::{expect, Expect},
        solana_sdk::{
            instruction::CompiledInstruction,
            message::{v0::Message, AccountKeys, Message as LegacyMessage, VersionedMessage},
            pubkey::Pubkey,
            transaction::VersionedTransaction,
        },
        solana_transaction_status::{InnerInstruction, InnerInstructions, TransactionStatusMeta},
    };

    use {super::try_send_instructions, crate::types::BonsolInstruction};

    fn check_instructions(output: &[BonsolInstruction], expect: Expect) {
        expect.assert_eq(&format!("{output:#?}"));
    }

    fn create_test_compiled_tx(
        instructions: Vec<CompiledInstruction>,
        legacy: bool,
    ) -> (VersionedTransaction, Pubkey) {
        let mut t = VersionedTransaction::default();
        let program = Pubkey::new_unique();
        t.message = if legacy {
            let mut msg = LegacyMessage::default();
            msg.instructions = instructions;
            VersionedMessage::Legacy(msg)
        } else {
            let mut msg = Message::default();
            msg.instructions = instructions;
            VersionedMessage::V0(msg)
        };
        (t, program)
    }

    fn create_test_inner_tx(instructions: Vec<InnerInstructions>) -> TransactionStatusMeta {
        let mut t = TransactionStatusMeta::default();
        t.inner_instructions = Some(instructions);
        t
    }

    #[tokio::test]
    async fn v1_txns_pass() {
        let (txchan, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let (txndata, program) = create_test_compiled_tx(
            vec![CompiledInstruction::new_from_raw_parts(
                0,
                vec![0, 0, 0, 0],
                vec![0, 1],
            )],
            false,
        );
        let meta = create_test_inner_tx(vec![InnerInstructions {
            index: 0,
            instructions: vec![InnerInstruction {
                instruction: CompiledInstruction::new_from_raw_parts(
                    0,
                    vec![0, 0, 0, 0],
                    vec![0, 1],
                ),
                stack_height: None,
            }],
        }]);
        let static_keys = vec![program];
        let acc = AccountKeys::new(&static_keys, None);

        try_send_instructions(program, 1, acc, txndata, Some(meta), &txchan)
            .expect("failed to send instructions");

        assert!(!rx.is_empty());
        let bonsol_ixs = rx
            .recv()
            .await
            .expect("expected a non-empty vector of BonsolInstructions");

        check_instructions(
            &bonsol_ixs,
            expect![[r#"
            [
                BonsolInstruction {
                    cpi: false,
                    accounts: [
                        1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM,
                    ],
                    data: [
                        0,
                        0,
                        0,
                        0,
                    ],
                    last_known_block: 1,
                },
                BonsolInstruction {
                    cpi: true,
                    accounts: [
                        1111111QLbz7JHiBTspS962RLKV8GndWFwiEaqKM,
                    ],
                    data: [
                        0,
                        0,
                        0,
                        0,
                    ],
                    last_known_block: 1,
                },
            ]"#]],
        );
    }

    #[tokio::test]
    async fn legacy_txns_fail() {
        let (txchan, rx) = tokio::sync::mpsc::unbounded_channel();

        let (txndata, program) = create_test_compiled_tx(
            vec![CompiledInstruction::new_from_raw_parts(
                0,
                vec![0, 0, 0, 0],
                vec![0, 1],
            )],
            true,
        );
        let meta = create_test_inner_tx(vec![InnerInstructions {
            index: 0,
            instructions: vec![InnerInstruction {
                instruction: CompiledInstruction::new_from_raw_parts(
                    0,
                    vec![0, 0, 0, 0],
                    vec![0, 1],
                ),
                stack_height: None,
            }],
        }]);
        let static_keys = vec![program];
        let acc = AccountKeys::new(&static_keys, None);

        try_send_instructions(program, 1, acc, txndata, Some(meta), &txchan)
            .expect("failed to send instructions");

        assert!(rx.is_empty())
    }
}
