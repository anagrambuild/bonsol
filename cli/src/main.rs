use std::fs;
use std::io::{self, Read};
use std::path::Path;

use atty::Stream;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use common::{execute_get_inputs, ZkProgramManifest};
use risc0_circuit_rv32im::prove::emu::exec::DEFAULT_SEGMENT_LIMIT_PO2;
use risc0_circuit_rv32im::prove::emu::testutil::DEFAULT_SESSION_LIMIT;
use risc0_zkvm::ExecutorEnv;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;

use crate::command::{BonsolCli, ParsedBonsolCli, ParsedCommand};
use crate::common::{sol_check, try_load_from_config};
use crate::error::{BonsolCliError, ZkManifestError};

mod build;
mod deploy;
mod estimate;
mod execute;
mod init;
mod prove;

#[cfg(all(test, feature = "integration-tests"))]
mod tests;

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
            manifest_path,
            input_file,
            max_cycles,
        } => {
            let manifest_file = fs::File::open(Path::new(&manifest_path)).map_err(|err| {
                BonsolCliError::ZkManifestError(ZkManifestError::FailedToOpen {
                    manifest_path: manifest_path.clone(),
                    err,
                })
            })?;
            let manifest: ZkProgramManifest =
                serde_json::from_reader(manifest_file).map_err(|err| {
                    BonsolCliError::ZkManifestError(ZkManifestError::FailedDeserialization {
                        manifest_path,
                        err,
                    })
                })?;
            let elf = fs::read(&manifest.binary_path).map_err(|err| {
                BonsolCliError::ZkManifestError(ZkManifestError::FailedToLoadBinary {
                    binary_path: manifest.binary_path.clone(),
                    err,
                })
            })?;
            let mut env = &mut ExecutorEnv::builder();
            env = env
                .segment_limit_po2(DEFAULT_SEGMENT_LIMIT_PO2 as u32)
                .session_limit(max_cycles.or(DEFAULT_SESSION_LIMIT));

            if input_file.is_some() {
                let inputs = execute_get_inputs(input_file, None)?;
                let inputs: Vec<&str> = inputs.iter().map(|i| i.data.as_str()).collect();
                env = env.write(&inputs.as_slice())?;
            }
            estimate::estimate(elf.as_slice(), env.build()?)
        }
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
