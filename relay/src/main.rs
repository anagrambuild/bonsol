pub mod types;
pub mod util;
#[macro_use]
pub mod observe;
mod ingest;

mod callback;
pub mod config;
mod prover;
use {
    anyhow::{Ok, Result},
    callback::{RpcTransactionSender, TransactionSender},
    config::*,
    ingest::{GrpcIngester, Ingester, RpcIngester},
    metrics::counter,
    metrics_exporter_prometheus::PrometheusBuilder,
    observe::MetricEvents,
    prover::Risc0Runner,
    solana_sdk::{pubkey::Pubkey, signature::read_keypair_file, signer::Signer},
    std::{str::FromStr, sync::Arc},
    thiserror::Error,
    tokio::{select, signal},
    tracing::{error, info},
    tracing_subscriber,
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
    tracing_subscriber::fmt()
        .json()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "-f" {
        error!("Usage: relay -f <config_file>");
        return Ok(());
    }
    let config_file = &args[2];
    let config = config::load_config(config_file);
    let program = Pubkey::from_str(&config.bonsol_program)?;
    if let MetricsConfig::Prometheus {} = config.metrics_config {
        let builder = PrometheusBuilder::new();
        builder
            .install()
            .expect("failed to install prometheus exporter");
        info!("Prometheus exporter installed");
    }
    emit_event!(MetricEvents::BonsolStartup, up => true);
    //todo use traits for signer
    let signer = match config.signer_config.clone() {
        SignerConfig::KeypairFile { path } => {
            info!("Using Keypair File");
            read_keypair_file(&path).map_err(|_| CliError::InvalidSigner)?
        }
        _ => return Err(CliError::InvalidSigner.into()),
    };
    let signer_identity = signer.pubkey();

    //Todo traitify ingester
    let mut ingester: Box<dyn Ingester> = match config.ingester_config.clone() {
        IngesterConfig::RpcBlockSubscription { wss_rpc_url } => {
            info!("Using RPC Block Subscription");
            Box::new(RpcIngester::new(wss_rpc_url))
        }
        IngesterConfig::GrpcSubscription {
            grpc_url,
            token,
            connection_timeout_secs,
            timeout_secs,
        } => {
            info!("Using GRPC Subscription");
            Box::new(GrpcIngester::new(
                grpc_url,
                token,
                Some(connection_timeout_secs),
                Some(timeout_secs),
            ))
        }
        _ => return Err(CliError::InvalidIngester.into()),
    };

    let mut transaction_sender = match config.transaction_sender_config.clone() {
        TransactionSenderConfig::Rpc { rpc_url } => {
            RpcTransactionSender::new(rpc_url, program, signer)
        }
        _ => return Err(CliError::InvalidRpcUrl.into()),
    };
    transaction_sender.start();
    //may take time to load images, depending on the number of images TODO put limit
    let mut runner = Risc0Runner::new(
        config.clone(),
        signer_identity,
        config.risc0_image_folder,
        Arc::new(transaction_sender),
    )
    .await?;
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
            info!("Runner exited");
        },
        _ = signal::ctrl_c() => {

        },
    }
    info!("Exited");
    Ok(())
}
