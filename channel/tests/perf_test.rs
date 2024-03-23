#![cfg(feature = "test-sbf")]
use std::fs::File;

use anagram_bonsol_channel::{execution_address, ID};
use solana_program_test::*;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

mod util;
use anyhow::Result;
use std::io::Read;
use util::*;

#[tokio::test]
async fn test_proof_verification_performance() -> Result<()> {
    let mut context = program_test().start_with_context().await;
    let requester = Keypair::new();
    let input1 = r#"{ "attestation": "test" }"#;
    let input2 = "attestation";
    let create_req_ix = execute(ExecuteParams {
        requester: requester.pubkey(),
        execution_id: "test-execution-id".to_string(),
        image_id: "ffddd818c78b130ba84aacd1cc69e075e40053c4ca14f1b0272bf196c204d2d0".to_string(),
       input: 
        
        callback_program_id: None,
        callback_instruction_prefix: None,
        tip: None,
    })
    .await?;
    airdrop(&mut context, &requester.pubkey(), 100000000).await?;
    let txn1 = Transaction::new_signed_with_payer(
        &[create_req_ix],
        Some(&requester.pubkey()),
        &[&requester],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(txn1).await?;

    let mut load_saved_status = File::open("tests/fixtures/status.fb")?;
    let mut buffer = Vec::new();
    load_saved_status.read_to_end(&mut buffer)?;
    let (ea, _) = execution_address(&requester.pubkey(), "test-execution-id".as_bytes());

    let verify_ix = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(requester.pubkey(), false),
            AccountMeta::new(ea, false),
            AccountMeta::new(ID, false),
            AccountMeta::new(context.payer.pubkey(), true),
        ],
        data: buffer,
    };
    let txn2 = Transaction::new_signed_with_payer(
        &[verify_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    let f = context.banks_client.simulate_transaction(txn2).await?;
    println!(
        "Units Consumed {}",
        f.simulation_details.unwrap().units_consumed
    );

    Ok(())
}
