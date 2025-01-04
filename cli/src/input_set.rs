use std::{fs, str::FromStr};

use anyhow::Result;
use bonsol_sdk::BonsolClient;
use solana_sdk::signer::Signer;

use crate::{
    command::CliInputSetOp,
    common::{execute_get_inputs, rand_id, CliInput, CliInputType, ExecutionRequestFile},
    error::BonsolCliError,
};

pub(crate) fn resolve_inputs(
    execution_request_file: Option<String>,
    input_file: Option<String>,
    stdin: Option<String>,
) -> Result<Vec<CliInput>> {
    if input_file.as_ref().or(stdin.as_ref()).is_some() {
        execute_get_inputs(input_file, stdin)
    } else {
        let req_file = fs::File::open(execution_request_file.ok_or(
            BonsolCliError::MissingInputs(
                "Input set management function called without supplying inputs. Please supply inputs via stdin, input file, or execution request file.".into()
            )
        )?)?;
        let req_file: ExecutionRequestFile = serde_json::from_reader(req_file)?;
        req_file.inputs.ok_or(
            BonsolCliError::MissingInputs(
                "The provided execution request file does not contain any inputs. Please update the inputs field and try again.".into()
            ).into()
        )
    }
}

pub async fn input_set(
    sdk: &BonsolClient,
    keypair: impl Signer,
    id: Option<String>,
    op: CliInputSetOp,
    inputs: Vec<CliInput>,
) -> Result<()> {
    let ixs = sdk
        .input_set_v1(
            &keypair.pubkey(),
            id.or(Some(rand_id(8))).unwrap().as_str(),
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
fn test_input_set_v1() {
    let signer = solana_sdk::pubkey::Pubkey::new_unique();
    let op = CliInputSetOp::Create;
    let inputs = vec![
        crate::common::CliInput {
            input_type: String::from("PublicData"),
            data: String::from("{\"attestation\":\"test\"}"),
        },
        crate::common::CliInput {
            input_type: String::from("Private"),
            data: String::from("https://echoserver.dev/server?response=N4IgFgpghgJhBOBnEAuA2mkBjA9gOwBcJCBaAgTwAcIQAaEIgDwIHpKAbKASzxAF0+9AEY4Y5VKArVUDCMzogYUAlBlFEBEAF96G5QFdkKAEwAGU1qA"),
        }
    ];

    assert!(
        bonsol_sdk::instructions::input_set_v1(
            &signer,
            rand_id(8).as_str(),
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
