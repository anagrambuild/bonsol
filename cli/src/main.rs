use std::io::{self, Read};
use std::path::Path;

use atty::Stream;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;

use crate::command::{BonsolCli, ParsedBonsolCli, ParsedCommand};
use crate::common::{sol_check, try_load_from_config};
use crate::error::BonsolCliError;

mod build;
mod deploy;
mod estimate;
mod execute;
mod init;
mod prove;

pub mod command;
pub mod common;
pub(crate) mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ParsedBonsolCli {
        config,
        keypair,
        rpc_url,
        command,
    } = BonsolCli::parse().try_into()?;

    let (rpc, kpp) = match rpc_url.zip(keypair) {
        Some(conf) => conf,
        None => try_load_from_config(config)?,
    };
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
        ParsedCommand::Build { zk_program_path } => build::build(&keypair, zk_program_path),
        ParsedCommand::Deploy { deploy_args } => {
            if !sol_check(rpc.clone(), keypair.pubkey()).await {
                return Err(BonsolCliError::InsufficientFundsForTransactions(
                    keypair.pubkey().to_string(),
                )
                .into());
            }
            deploy::deploy(rpc, keypair, deploy_args).await
        }
        ParsedCommand::Estimate {
            zk_program_path,
            runtime_args,
            build,
        } => estimate::estimate(&keypair, zk_program_path, &runtime_args, build),
        ParsedCommand::Execute {
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
        ParsedCommand::Prove {
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
        ParsedCommand::Init { project_name, dir } => init::init_project(&project_name, dir),
    }
}
