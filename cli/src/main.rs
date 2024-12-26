use std::fs;
use std::io::{self, Read};
use std::path::Path;

use atty::Stream;
use bonsol_sdk::BonsolClient;
use clap::Parser;
use risc0_circuit_rv32im::prove::emu::exec::DEFAULT_SEGMENT_LIMIT_PO2;
use risc0_circuit_rv32im::prove::emu::testutil::DEFAULT_SESSION_LIMIT;
use risc0_zkvm::ExecutorEnv;
use solana_sdk::signer::Signer;

use crate::command::{BonsolCli, Command};
use crate::common::{execute_get_inputs, load_solana_config, sol_check, ZkProgramManifest};
use crate::error::{BonsolCliError, ZkManifestError};

mod build;
mod deploy;
mod estimate;
mod execute;
mod init;
mod input_set;
mod prove;

#[cfg(all(test, feature = "integration"))]
mod tests;

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

    match command {
        Command::Build { zk_program_path } => build::build(
            &load_solana_config(config, rpc_url, keypair)?.1,
            zk_program_path,
        ),
        Command::Deploy { deploy_args } => {
            let (rpc_url, keypair) = load_solana_config(config, rpc_url, keypair)?;
            if !sol_check(rpc_url.clone(), keypair.pubkey()).await {
                return Err(BonsolCliError::InsufficientFunds(keypair.pubkey().to_string()).into());
            }

            deploy::deploy(rpc_url, keypair, deploy_args).await
        }
        Command::Estimate {
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
            let (rpc_url, keypair) = load_solana_config(config, rpc_url, keypair)?;
            if !sol_check(rpc_url.clone(), keypair.pubkey()).await {
                return Err(BonsolCliError::InsufficientFunds(keypair.pubkey().to_string()).into());
            }
            let stdin = atty::isnt(Stream::Stdin)
                .then(|| {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer).ok()?;
                    (!buffer.trim().is_empty()).then_some(buffer)
                })
                .flatten();
            let sdk = BonsolClient::new(rpc_url.clone());

            execute::execute(
                &sdk,
                rpc_url,
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
            let rpc_url = load_solana_config(config, rpc_url, keypair)?.0;
            let stdin = atty::isnt(Stream::Stdin)
                .then(|| {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer).ok()?;
                    (!buffer.trim().is_empty()).then_some(buffer)
                })
                .flatten();
            let sdk = BonsolClient::new(rpc_url.clone());

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
        Command::InputSet { input_set } => {
            let (rpc_url, keypair) = load_solana_config(config, rpc_url, keypair)?;
            let sdk = BonsolClient::new(rpc_url.clone());
            input_set::input_set(&sdk, &keypair, input_set).await
        }
        Command::Init { project_name, dir } => init::init_project(&project_name, dir),
    }
}
