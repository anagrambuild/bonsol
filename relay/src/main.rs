
mod ingest;
mod bonsai;
use std::str::FromStr;

use anagram_bonsol_schema::parse_ix_data;
use bonsai::BonsaiRunner;
use clap::{Parser};
use clap_derive::Subcommand;
use ingest::Ingester;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::{UiInnerInstructions, UiInstruction};
use thiserror::Error;
use anyhow::Result;
use tokio::sync::Semaphore;
use tokio::{select, task::JoinHandle};
use tokio::signal;
use bonsai_sdk::alpha::Client;
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    StartWithRpc {
        #[arg(short, long)]
        rpc_url: String,
        #[arg(short, long)]
        bonsol_program: String,
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short,long)]
    risc0_image_folder: String,
    #[arg(short, long)]
    bonsai_api_key: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid RPC URL")]
    InvalidRpcUrl,
    #[error("Invalid Bonsol program")]
    InvalidBonsolProgram,
}


async fn start_rpc_ingester(
    rpc_url: String, 
    bonsol_program: String,
    image_folder: String,
    bonai_api_key: String,
    sem: Semaphore
) -> Result<()> {
    let mut ingester = ingest::RpcIngester::new(rpc_url);
    let program = Pubkey::from_str(&bonsol_program)?;
    let mut bix_channel = ingester.start(program)?;
    
    let bonsai_client: Client = Client::from_parts(
        "https://api.bonsai.xyz".to_string(),
       bonai_api_key,
        risc0_zkvm::VERSION
    )?;
    let mut runner = BonsaiRunner::new(bonsai_client,image_folder, "./data".to_string())?;
    let runner_chan = runner.start()?;
    
    while let Some(bix) = bix_channel.recv().await {
        for ix in bix {
            runner_chan.send(ix).unwrap();
        }
    }
    
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    let sem = Semaphore::new(15);
    let args: Args = Args::parse();
    
    let taskhandle = match args.subcmd {
        SubCommand::StartWithRpc { rpc_url, bonsol_program } => {
            tokio::spawn(async move {
                match start_rpc_ingester(rpc_url, 
                    bonsol_program,
                    args.risc0_image_folder,
                    args.bonsai_api_key,
                    sem
                ).await {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                
                }
            })
        }
    };

    select! {
        _ = signal::ctrl_c() => {},
    }
    taskhandle.abort();
    
    Ok(())
}
