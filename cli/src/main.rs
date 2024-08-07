mod build;
mod deploy;
pub mod common;
pub mod command;
use clap::Parser;
use command::{BonsolCli, Commands};
use solana_cli_config::{Config, CONFIG_FILE};
use solana_sdk::signature::read_keypair_file;
use std::path::Path;

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
    match command {
        Commands::Build { zk_program_path } => {
           match build::build(&keypair, zk_program_path) {
                Err(e) => {
                    anyhow::bail!(e);
                }
                Ok(_) => {
                    println!("Build complete");
                }
            }
        }
        Commands::Deploy {
            manifest_path,
            s3_upload,
            shadow_drive_upload,
            auto_confirm,
            deploy_type,
        } => {
            deploy::deploy(rpc, &keypair, manifest_path, s3_upload, shadow_drive_upload, auto_confirm, deploy_type).await?;
        },
        Commands::Prove {
            manifest_path,
            prove_mode,
            inputs,
            input_file,
        } => {
            todo!()
        }
    };
    Ok(())
}
