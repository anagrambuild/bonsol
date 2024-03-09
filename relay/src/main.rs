mod ingest;
mod risc0;
mod callback;
pub mod types;
use std::{str::FromStr};
use anyhow::Result;
use clap::Parser;
use clap_derive::Subcommand;
use ingest::Ingester;
use risc0::Risc0Runner;
use solana_sdk::{pubkey::Pubkey, signature::{read_keypair_file, Keypair}};
use thiserror::Error;
use tokio::{select, signal};
#[derive(Subcommand, Debug)]
pub enum SubCommand {
    StartWithRpc {
        #[arg(short, long)]
        wss_rpc_url: String,
        #[arg(short, long)]
        rpc_url: String,
        #[arg(short, long)]
        bonsol_program: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long="keypair", short='k')]
    node_keypair_path: String,
    #[arg(long)]
    risc0_image_folder: String,
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
    wss_rpc_url: String,
    bonsol_program: String,
    image_folder: String,
    signer: Keypair
) -> Result<()> {
    let mut ingester = ingest::RpcIngester::new(wss_rpc_url);
    let program = Pubkey::from_str(&bonsol_program)?;
    let mut bix_channel = ingester.start(program)?;
    let mut callback = callback::RpcCallback::new(rpc_url, bonsol_program, signer);
    let s = callback.start().await?;
    let mut runner = Risc0Runner::new(image_folder, s)?;
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
            wss_rpc_url,
            rpc_url,
            bonsol_program,
        } => tokio::spawn(async move {
            let signer = read_keypair_file(args.node_keypair_path)
                .map_err(|_| CliError::InvalidRpcUrl)
                ?;
            start_rpc_ingester_risc0(rpc_url, wss_rpc_url, bonsol_program, args.risc0_image_folder, signer).await
        }),
    };

    select! {
        _ = signal::ctrl_c() => {},
    }
    taskhandle.abort();

    Ok(())
}
