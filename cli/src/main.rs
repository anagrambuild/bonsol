mod build;
mod deploy;
mod execute;
mod init;
mod prove;

// mod execute;
pub mod command;
pub mod common;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use command::{BonsolCli, Commands};
use solana_cli_config::{Config, CONFIG_FILE};
use solana_sdk::signature::read_keypair_file;
use std::{io::{self, Read}, path::Path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = BonsolCli::parse();
    let keypair = cli.keypair;
    let config = cli.config;
    let rpc_url = cli.rpc_url;
    let (rpc, kpp) = match (rpc_url, keypair, config) {
        (Some(rpc_url), Some(keypair), None) => (rpc_url, keypair),
        (None, None, config) => {
            let config = Config::load(&config.unwrap_or(CONFIG_FILE.clone().unwrap()));
            match config {
                Ok(config) => (config.json_rpc_url, config.keypair_path),
                Err(e) => {
                    anyhow::bail!("Error loading config: {:?}", e);
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
    let keypair = keypair.unwrap();
    let command = cli.command;
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
            shadow_drive_upload,
            auto_confirm,
            deploy_type,
        } => {
            deploy::deploy(
                rpc,
                &keypair,
                manifest_path,
                s3_upload,
                shadow_drive_upload,
                auto_confirm,
                deploy_type,
            )
            .await?;
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
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let stdin = if buffer.trim().is_empty() {
                None 
            } else {
                Some(buffer)
            };
            execute::execute(
                &sdk,
                rpc,
                &keypair,
                execution_request_file,
                program_id,
                execution_id,
                expiry,
                input_file,
                tip,
                timeout,
                stdin,
                wait
            )
            .await?;
        }
        Commands::Init { project_name, dir } => {
            init::init_project(&project_name, dir)?;
        }
        _ => {
            println!("Invalid command");
            return Ok(());
        }
    };
    Ok(())
}
