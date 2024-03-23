mod callback;
pub mod config;
mod ingest;
mod risc0;
pub mod types;
pub mod util;
use {
    crate::{callback::TransactionSender, ingest::Ingester},
    anyhow::Result,
    config::*,
    risc0::Risc0Runner,
    solana_sdk::{
        pubkey::Pubkey, signature::{read_keypair_file, Keypair}, signer::Signer, transaction::Transaction
    },
    std::{str::FromStr, sync::Arc},
    thiserror::Error,
    tokio::{select, signal},
};

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Invalid RPC URL")]
    InvalidRpcUrl,
    #[error("Invalid Bonsol program")]
    InvalidBonsolProgram,
    #[error("Invalid RISC0 image folder")]
    InvalidRisc0ImageFolder,
    #[error("Invalid signer: Missing/Invalid")]
    InvalidSigner,
    #[error("Invalid Ingester")]
    InvalidIngester,
    #[error("Invalid Transaction Sender")]
    InvalidTransactionSender,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "-f" {
        eprintln!("Usage: relay -f <config_file>");
        return Ok(());
    }
    let config_file = &args[2];
    let config = config::load_config(config_file);
    let program = Pubkey::from_str(&config.bonsol_program)?;
    //todo use traits for signer
    let signer = match config.signer_config.clone() {
        SignerConfig::KeypairFile { path } => {
            read_keypair_file(&path).map_err(|_| CliError::InvalidSigner)?
        }
        _ => return Err(CliError::InvalidSigner.into()),
    };
    let signer_identity = signer.pubkey();

    //Todo traitify ingester
    let mut ingester = match config.ingester_config.clone() {
        IngesterConfig::RpcBlockSubscription { wss_rpc_url } => {
            ingest::RpcIngester::new(wss_rpc_url)
        }
        _ => return Err(CliError::InvalidIngester.into()),
    };

    let transaction_sender = match config.transaction_sender_config.clone() {
        TransactionSenderConfig::Rpc { rpc_url } => {
            TransactionSender::new(rpc_url, program, signer)
        }
        _ => return Err(CliError::InvalidRpcUrl.into()),
    };

    //may take time to load images, depending on the number of images TODO put limit
    let mut runner =Risc0Runner::new(
        config.clone(),
        signer_identity,
        config.risc0_image_folder, 
        Arc::new(transaction_sender)
    ).await?;
    let runner_chan = runner.start()?;
    let mut ingester_chan = ingester.start(program)?;
    let handle = tokio::spawn(async move {
        while let Some(bix) = ingester_chan.recv().await {
            for ix in bix {
                runner_chan.send(ix).unwrap();
            }
        }
    });

    select! {
        _ = handle => {
            
            eprintln!("Runner exited");
        },
        _ = signal::ctrl_c() => {
            
        },
    }
    
    eprintln!("Exited");
    Ok(())
}
