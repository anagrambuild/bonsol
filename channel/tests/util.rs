use anagram_bonsol_channel::{execution_address, ID};
use anagram_bonsol_schema::{
    ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType, ExecutionInputType,
    ExecutionRequestV1, ExecutionRequestV1Args,
};
use anyhow::Result;
use flatbuffers::FlatBufferBuilder;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{system_instruction, system_program};

pub fn program_test() -> ProgramTest {
    let program_test = ProgramTest::new("anagram_bonsol_channel", ID, None);

    program_test
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}

pub async fn airdrop(
    context: &mut ProgramTestContext,
    receiver: &Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            receiver,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();
    Ok(())
}

pub async fn execute(params: ExecuteParams) -> Result<Instruction> {
    let ExecuteParams {
        execution_id,
        image_id,
        input_type,
        input,
        requester,
        callback_program_id,
        callback_instruction_prefix,
        tip,
    } = params;

    let mut builder = FlatBufferBuilder::new();

    let exec_id = builder.create_string(&execution_id);
    let img_id = builder.create_string(&image_id);

    let cb = callback_program_id.map(|id| builder.create_vector(id.as_ref()));
    let prf = callback_instruction_prefix.map(|prefix| builder.create_vector(&prefix));
    let ind = builder.create_vector(&input);

    let execution_request = ExecutionRequestV1::create(
        &mut builder,
        &ExecutionRequestV1Args {
            execution_id: Some(exec_id),
            image_id: Some(img_id),
            callback_program_id: cb,
            callback_instruction_prefix: prf,
            input_type: input_type,
            input_data: Some(ind),
            tip: tip.unwrap_or(0),
            input_digest: None,
        },
    );

    {
        builder.finish(execution_request, None);
    }
    let erbuf = builder.finished_data();
    let mut builder2 = &mut FlatBufferBuilder::new();
    let erv = builder2.create_vector(erbuf);

    let channel_instruction = ChannelInstruction::create(
        &mut builder2,
        &ChannelInstructionArgs {
            execute_v1: Some(erv),
            status_v1: None,
            ix_type: ChannelInstructionIxType::ExecuteV1,
        },
    );
    {
        builder2.finish(channel_instruction, None);
    }
    let ci = builder2.finished_data();

    let (execution_address, _) = execution_address(&requester, execution_id.as_bytes());
    let callback_program_address = callback_program_id.unwrap_or(ID);

    let accounts = vec![
        AccountMeta::new(requester, true),
        AccountMeta::new(execution_address, false),
        AccountMeta::new_readonly(callback_program_address, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    Ok(Instruction {
        program_id: ID,
        accounts,
        data: ci.to_vec(),
    })
}

pub struct ExecuteParams {
    pub execution_id: String,
    pub image_id: String,
    pub input_type: ExecutionInputType,
    pub input: Vec<u8>,
    pub requester: Pubkey,
    pub callback_program_id: Option<Pubkey>,
    pub callback_instruction_prefix: Option<Vec<u8>>,
    pub tip: Option<u64>,
}
