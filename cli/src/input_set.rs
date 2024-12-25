use anyhow::Result;
use bonsol_sdk::BonsolClient;
use solana_sdk::signer::Signer;

use crate::command::CliInputSetOp;

pub async fn input_set(
    sdk: &BonsolClient,
    keypair: impl Signer,
    action: CliInputSetOp,
) -> Result<()> {
    // let ixs = sdk.input_set_v1(&keypair, , , )
    // sdk.send_txn_standard(&keypair, ixs).await?;
    Ok(())
}
