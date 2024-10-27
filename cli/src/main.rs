use std::io::{self, Read};
use std::path::Path;

use atty::Stream;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;

use crate::command::{BonsolCli, Command};
use crate::common::{sol_check, try_load_from_config};
use crate::error::BonsolCliError;

mod build;
mod deploy;
mod execute;
mod init;
mod prove;

pub mod command;
pub mod common;
pub(crate) mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let BonsolCli {
        config,
        keypair,
        rpc_url,
        command,
    } = BonsolCli::parse();

    let (rpc, kpp) = rpc_url
        .zip(keypair)
        .unwrap_or(try_load_from_config(config)?);
    let keypair =
        read_keypair_file(Path::new(&kpp)).map_err(|err| BonsolCliError::FailedToReadKeypair {
            file: kpp,
            err: format!("{err:?}"),
        })?;
    let stdin = atty::isnt(Stream::Stdin)
        .then(|| {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer).ok()?;
            (!buffer.trim().is_empty()).then_some(buffer)
        })
        .flatten();
    let sdk = BonsolClient::new(rpc.clone());

    match command {
        Command::Build { zk_program_path } => {
            build::build(&keypair, zk_program_path).and_then(|_| Ok(println!("Build complete")))
        }
        Command::Deploy {
            manifest_path,
            auto_confirm,
            deploy_type,
        } => {
            if !sol_check(rpc.clone(), keypair.pubkey()).await {
                return Err(BonsolCliError::InsufficientFundsForTransactions(
                    keypair.pubkey().to_string(),
                )
                .into());
            }
            deploy::deploy(rpc, keypair, manifest_path, auto_confirm, deploy_type).await
        }
        Command::Execute {
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
                return Err(BonsolCliError::InsufficientFundsForTransactions(
                    keypair.pubkey().to_string(),
                )
                .into());
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
            .await
        }
        Command::Prove {
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
            .await
        }
        Command::Init { project_name, dir } => init::init_project(&project_name, dir),
    }
}
