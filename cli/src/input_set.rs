use anyhow::Result;
use bonsol_sdk::BonsolClient;
use solana_sdk::signer::Signer;

use crate::{command::CliInputSetOp, common::CliInput};

pub async fn input_set(
    sdk: &BonsolClient,
    keypair: impl Signer,
    program_id: Option<String>,
    op: CliInputSetOp,
    inputs: Vec<CliInput>,
) -> Result<()> {
    let ixs = sdk
        .input_set_v1(
            &keypair.pubkey(),
            program_id.unwrap().as_str(),
            op.into(),
            inputs.len(),
            inputs.iter().map(|i| i.into()),
        )
        .await?;
    sdk.send_txn_standard(&keypair, ixs).await?;
    Ok(())
}
