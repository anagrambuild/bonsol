use std::str::FromStr;

use anyhow::Result;
use bonsol_sdk::BonsolClient;
use solana_sdk::signer::Signer;

use crate::{
    command::CliInputSetOp,
    common::{CliInput, CliInputType},
};

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
            inputs.into_iter().map(|i| {
                (
                    CliInputType::from_str(&i.input_type)
                        .expect(&format!("found invalid input type: {}", i.input_type))
                        .0,
                    i.data,
                )
            }),
        )
        .await?;
    sdk.send_txn_standard(&keypair, ixs).await?;
    Ok(())
}

#[test]
fn test_input_set_v1_with_simple_image() {
    let signer = solana_sdk::pubkey::Pubkey::new_unique();
    let image_id = "68f4b0c5f9ce034aa60ceb264a18d6c410a3af68fafd931bcfd9ebe7c1e42960";
    let op = CliInputSetOp::Create;
    let inputs = vec![
        CliInput {
            input_type: String::from("PublicData"),
            data: String::from("{\"attestation\":\"test\"}"),
        },
        CliInput {
            input_type: String::from("Private"),
            data: String::from("https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA"),
        }
    ];

    assert!(
        bonsol_sdk::instructions::input_set_v1(
            &signer,
            image_id,
            op.into(),
            inputs.len(),
            inputs.into_iter().map(|i| {
                (
                    CliInputType::from_str(&i.input_type)
                        .expect(&format!("found invalid input type: {}", i.input_type))
                        .0,
                    i.data,
                )
            }),
        )
        .is_ok(),
        "failed to create instruction from test data"
    );
}
