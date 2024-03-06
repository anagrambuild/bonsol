mod ingest;
mod risc0;
use std::str::FromStr;
mod bonsai;
use anyhow::Result;
use bonsai::BonsaiRunner;
use bonsai_sdk::alpha::Client;
use clap::Parser;
use clap_derive::Subcommand;
use ingest::Ingester;
use risc0::Risc0Runner;
use solana_sdk::pubkey::Pubkey;
use thiserror::Error;
use tokio::signal;
use tokio::{select};
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    StartWithRpc {
        #[arg(short, long)]
        rpc_url: String,
        #[arg(short, long)]
        bonsol_program: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    runner: String,
    #[arg(long)]
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

async fn start_rpc_ingester_risc0(
    rpc_url: String,
    bonsol_program: String,
    image_folder: String,
) -> Result<()> {
    let mut ingester = ingest::RpcIngester::new(rpc_url);
    let program = Pubkey::from_str(&bonsol_program)?;
    let mut bix_channel = ingester.start(program)?;

    let mut runner = Risc0Runner::new(image_folder)?;
    let runner_chan = runner.start()?;

    while let Some(bix) = bix_channel.recv().await {
        for ix in bix {
            runner_chan.send(ix).unwrap();
        }
    }
    Ok(())
}

async fn start_rpc_ingester_bonsai(
    rpc_url: String,
    bonsol_program: String,
    image_folder: String,
    bonai_api_key: String,
) -> Result<()> {
    let mut ingester = ingest::RpcIngester::new(rpc_url);
    let program = Pubkey::from_str(&bonsol_program)?;
    let mut bix_channel = ingester.start(program)?;

    let bonsai_client: Client = Client::from_parts(
        "https://api.bonsai.xyz".to_string(),
        bonai_api_key,
        risc0_zkvm::VERSION,
    )?;

    let mut runner = BonsaiRunner::new(bonsai_client, image_folder, "./data".to_string())?;
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
    let args: Args = Args::parse();
    let taskhandle = match args.subcmd {
        SubCommand::StartWithRpc {
            rpc_url,
            bonsol_program,
        } => tokio::spawn(async move {
             match args.runner.as_str() {
                "risc0" => {
                    start_rpc_ingester_risc0(rpc_url, bonsol_program, args.risc0_image_folder).await
                }
                "bonsai" => {
                    start_rpc_ingester_bonsai(
                        rpc_url,
                        bonsol_program,
                        args.risc0_image_folder,
                        args.bonsai_api_key,
                    )
                    .await
                },
                _ => Err(anyhow::anyhow!("Invalid runner")),
            }.unwrap();
        }),
    };

    select! {
        _ = signal::ctrl_c() => {},
    }
    taskhandle.abort();

    Ok(())
}
