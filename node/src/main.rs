pub mod types;
#[macro_use]
pub mod observe;
mod ingest;

mod callback;
pub mod config;
mod prover;
use {
    anyhow::Result,
    bonsol_prover::input_resolver::DefaultInputResolver,
    callback::{RpcTransactionSender, TransactionSender},
    config::*,
    ingest::{GrpcIngester, Ingester, RpcIngester},
    metrics::counter,
    metrics_exporter_prometheus::PrometheusBuilder,
    observe::MetricEvents,
    prover::Risc0Runner,
    rlimit::Resource,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
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
    init_tracing();
    set_unlimited_stack_size();

    let config = parse_args_and_load_config()?;
    let program = Pubkey::from_str(&config.bonsol_program)?;

    setup_metrics(&config)?;
    emit_event!(MetricEvents::BonsolStartup, up => true);

    let signer = setup_signer(&config)?;
    let signer_identity = signer.pubkey();

    let mut ingester = setup_ingester(&config)?;
    let (mut transaction_sender, solana_rpc_client) =
        setup_transaction_sender(&config, program, signer)?;

    transaction_sender.start();

    let input_resolver = setup_input_resolver(solana_rpc_client);
    let mut runner = create_runner(
        config.clone(),
        signer_identity,
        Arc::new(transaction_sender),
        Arc::new(input_resolver),
    )
    .await?;

    let runner_chan = runner.start()?;
    let mut ingester_chan = ingester.start(program)?;
    let handle = spawn_ingester_runner_task(ingester_chan, runner_chan);

    wait_for_exit(handle).await;

    info!("Exited");
    Ok(())
}

fn set_unlimited_stack_size() {
    if let Err(e) = rlimit::setrlimit(Resource::STACK, u64::MAX, u64::MAX) {
        eprintln!("Error setting rlimit: {}", e);
    }
}

fn parse_args_and_load_config() -> Result<Config> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "-f" {
        error!("Usage: bonsol-node -f <config_file>");
        return Err(CliError::InvalidArguments.into());
    }
    let config_file = &args[2];
    Ok(config::load_config(config_file))
}

fn setup_metrics(config: &Config) -> Result<()> {
    if let MetricsConfig::Prometheus {} = config.metrics_config {
        PrometheusBuilder::new()
            .install()
            .expect("failed to install prometheus exporter");
        info!("Prometheus exporter installed");
    }
    Ok(())
}

fn setup_signer(config: &Config) -> Result<Keypair> {
    match &config.signer_config {
        SignerConfig::KeypairFile { path } => {
            info!("Using Keypair File");
            read_keypair_file(path).map_err(|_| CliError::InvalidSigner.into())
        }
        _ => Err(CliError::InvalidSigner.into()),
    }
}

fn setup_ingester(config: &Config) -> Result<Box<dyn Ingester>> {
    match &config.ingester_config {
        IngesterConfig::RpcBlockSubscription { wss_rpc_url } => {
            info!("Using RPC Block Subscription");
            Ok(Box::new(RpcIngester::new(wss_rpc_url.clone())))
        }
        IngesterConfig::GrpcSubscription {
            grpc_url,
            token,
            connection_timeout_secs,
            timeout_secs,
        } => {
            info!("Using GRPC Subscription");
            Ok(Box::new(GrpcIngester::new(
                grpc_url.clone(),
                token.clone(),
                Some(*connection_timeout_secs),
                Some(*timeout_secs),
            )))
        }
        _ => Err(CliError::InvalidIngester.into()),
    }
}

fn setup_transaction_sender(
    config: &Config,
    program: Pubkey,
    signer: Keypair,
) -> Result<(RpcTransactionSender, RpcClient)> {
    match &config.transaction_sender_config {
        TransactionSenderConfig::Rpc { rpc_url } => {
            let transaction_sender = RpcTransactionSender::new(rpc_url.clone(), program, signer);
            let solana_rpc_client = RpcClient::new(rpc_url.clone());
            Ok((transaction_sender, solana_rpc_client))
        }
        _ => Err(CliError::InvalidRpcUrl.into()),
    }
}

fn setup_input_resolver(solana_rpc_client: RpcClient) -> DefaultInputResolver {
    DefaultInputResolver::new(
        Arc::new(reqwest::Client::new()),
        Arc::new(solana_rpc_client),
    )
}

async fn create_runner(
    config: Config,
    signer_identity: Pubkey,
    transaction_sender: Arc<RpcTransactionSender>,
    input_resolver: Arc<DefaultInputResolver>,
) -> Result<Risc0Runner> {
    Risc0Runner::new(
        config,
        signer_identity,
        config.risc0_image_folder.clone(),
        transaction_sender,
        input_resolver,
    )
    .await
}

fn spawn_ingester_runner_task(
    mut ingester_chan: Receiver<Vec<Instruction>>,
    runner_chan: Sender<Instruction>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(bix) = ingester_chan.recv().await {
            for ix in bix {
                runner_chan.send(ix).unwrap();
            }
        }
    })
}

async fn wait_for_exit(handle: JoinHandle<()>) {
    select! {
        _ = handle => {
            info!("Runner exited");
        },
        _ = signal::ctrl_c() => {},
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();
}
