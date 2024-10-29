mod build;
mod deploy;
mod execute;
mod init;
mod prove;

// mod execute;
pub mod command;
pub mod common;
use anyhow::anyhow;
use atty::Stream;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use command::{BonsolCli, Commands};
use common::sol_check;
use solana_cli_config::{Config, CONFIG_FILE};
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;
use std::io::{self, Read};
use std::path::Path;

const SOL_CHECK_MESSAGE: &str = "Your account needs to have some SOL to pay for the transactions";
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = BonsolCli::parse();
    let keypair = cli.keypair;
    let config = cli.config;
    let rpc_url = cli.rpc_url;
    let (rpc, kpp) = match (rpc_url, keypair, config) {
        (Some(rpc_url), Some(keypair), None) => (rpc_url, keypair),
        (None, None, config) => {
            let config_location = CONFIG_FILE
                .clone()
                .ok_or(anyhow!("Please provide a config file"))?;
            let config = Config::load(&config.unwrap_or(config_location.clone()));
            match config {
                Ok(config) => (config.json_rpc_url, config.keypair_path),
                Err(e) => {
                    anyhow::bail!("Error loading config [{}]: {:?}", config_location, e);
                }
            }
        }
        _ => {
            anyhow::bail!("Please provide a keypair and rpc or a solana config file");
        }
    };

    let keypair = read_keypair_file(Path::new(&kpp));
    if keypair.is_err() {
        anyhow::bail!("Invalid keypair");
    }
    let command = cli.command;
    let keypair = keypair.unwrap();
    let stdin = if atty::isnt(Stream::Stdin) {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            None
        } else {
            Some(buffer)
        }
    } else {
        None
    };
    let sdk = BonsolClient::new(rpc.clone());
    match command {
        Commands::Build { zk_program_path } => match build::build(&keypair, zk_program_path) {
            Err(e) => {
                anyhow::bail!(e);
            }
            Ok(_) => {
                println!("Build complete");
            }
        },
        Commands::Deploy {
            manifest_path,
            s3_upload,
            compute_units,
            auto_confirm,
        } => {
            if !sol_check(rpc.clone(), keypair.pubkey()).await {
                anyhow::bail!(SOL_CHECK_MESSAGE);
            }
            deploy::deploy(rpc, manifest_path, s3_upload, compute_units, auto_confirm).await?;
        }
        Commands::Execute {
            execution_request_file,
            program_id,
            execution_id,
            expiry,
            input_file,
            wait,
            tip,
            timeout,
        } => {
            if !sol_check(rpc.clone(), keypair.pubkey()).await {
                anyhow::bail!(SOL_CHECK_MESSAGE);
            }
            execute::execute(
                &sdk,
                rpc,
                &keypair,
                execution_request_file,
                program_id,
                execution_id,
                timeout,
                input_file,
                tip,
                expiry,
                stdin,
                wait,
            )
            .await?;
        }
        Commands::Prove {
            manifest_path,
            program_id,
            input_file,
            execution_id,
            output_location,
        } => {
            prove::prove(
                &sdk,
                execution_id,
                manifest_path,
                program_id,
                input_file,
                output_location,
                stdin,
            )
            .await?;
        }
        Commands::Init { project_name, dir } => {
            init::init_project(&project_name, dir)?;
        }
    };
    Ok(())
}
