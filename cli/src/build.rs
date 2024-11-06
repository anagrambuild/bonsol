use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use cargo_toml::Manifest;
use indicatif::ProgressBar;
use risc0_zkvm::compute_image_id;
use solana_sdk::signer::Signer;

use crate::common::*;
use crate::error::{BonsolCliError, ZkManifestError};

pub fn build(keypair: &impl Signer, zk_program_path: String) -> Result<()> {
    validate_build_dependencies()?;

    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));

    let image_path = Path::new(&zk_program_path);
    let (cargo_package_name, input_order) = parse_cargo_manifest(image_path)?;
    let build_result =
        build_zkprogram_manifest(image_path, &keypair, cargo_package_name, input_order);
    let manifest_path = image_path.join(MANIFEST_JSON);

    match build_result {
        Err(e) => {
            bar.finish_with_message(format!(
                "Build failed for program '{}': {:?}",
                image_path.to_string_lossy(),
                e
            ));
            Ok(())
        }
        Ok(manifest) => {
            serde_json::to_writer_pretty(File::create(&manifest_path)?, &manifest)?;
            bar.finish_and_clear();
            println!("Build complete");
            Ok(())
        }
    }
}

fn validate_build_dependencies() -> Result<(), BonsolCliError> {
    const CARGO_RISCZERO: &str = "risczero";
    const DOCKER: &str = "docker";

    let mut missing_deps = Vec::with_capacity(2);

    if !cargo_has_plugin(CARGO_RISCZERO) {
        missing_deps.push(format!("cargo-{}", CARGO_RISCZERO));
    }
    if !has_executable(DOCKER) {
        missing_deps.push(DOCKER.into());
    }

    if !missing_deps.is_empty() {
        return Err(BonsolCliError::MissingBuildDependencies { missing_deps });
    }

    Ok(())
}

fn parse_cargo_manifest_inputs(
    manifest: &Manifest,
    manifest_path_str: String,
) -> Result<Vec<String>> {
    const METADATA: &str = "metadata";
    const ZKPROGRAM: &str = "zkprogram";
    const INPUT_ORDER: &str = "input_order";

    let meta = manifest
        .package
        .as_ref()
        .and_then(|p| p.metadata.as_ref())
        .ok_or(ZkManifestError::MissingPackageMetadata(
            manifest_path_str.clone(),
        ))?;
    let meta_table = meta.as_table().ok_or(ZkManifestError::ExpectedTable {
        manifest_path: manifest_path_str.clone(),
        name: METADATA.into(),
    })?;
    let zkprogram = meta_table
        .get(ZKPROGRAM)
        .ok_or(ZkManifestError::MissingProgramMetadata {
            manifest_path: manifest_path_str.clone(),
            meta: meta.to_owned(),
        })?;
    let zkprogram_table = zkprogram.as_table().ok_or(ZkManifestError::ExpectedTable {
        manifest_path: manifest_path_str.clone(),
        name: ZKPROGRAM.into(),
    })?;
    let input_order =
        zkprogram_table
            .get(INPUT_ORDER)
            .ok_or(ZkManifestError::MissingInputOrder {
                manifest_path: manifest_path_str.clone(),
                zkprogram: zkprogram.to_owned(),
            })?;
    let inputs = input_order
        .as_array()
        .ok_or(ZkManifestError::ExpectedArray {
            manifest_path: manifest_path_str.clone(),
            name: INPUT_ORDER.into(),
        })?;

    let (input_order, errs): (
        Vec<Result<String, ZkManifestError>>,
        Vec<Result<String, ZkManifestError>>,
    ) = inputs
        .iter()
        .map(|i| -> Result<String, ZkManifestError> {
            i.as_str()
                .map(|s| s.to_string())
                .ok_or(ZkManifestError::InvalidInput(i.to_owned()))
        })
        .partition(|res| res.is_ok());
    if !errs.is_empty() {
        let errs: Vec<String> = errs
            .into_iter()
            .map(|r| format!("Error: {:?}\n", r.unwrap_err()))
            .collect();
        return Err(ZkManifestError::InvalidInputs {
            manifest_path: manifest_path_str,
            errs,
        }
        .into());
    }

    Ok(input_order.into_iter().map(Result::unwrap).collect())
}

fn parse_cargo_manifest(image_path: &Path) -> Result<(String, Vec<String>)> {
    let cargo_manifest_path = image_path.join(CARGO_TOML);
    let cargo_manifest_path_str = cargo_manifest_path.to_string_lossy().to_string();
    if !cargo_manifest_path.exists() {
        return Err(
            ZkManifestError::MissingManifest(image_path.to_string_lossy().to_string()).into(),
        );
    }
    let cargo_manifest = cargo_toml::Manifest::from_path(&cargo_manifest_path).map_err(|err| {
        ZkManifestError::FailedToLoadManifest {
            manifest_path: cargo_manifest_path_str.clone(),
            err,
        }
    })?;
    let cargo_package_name = cargo_manifest
        .package
        .as_ref()
        .map(|p| p.name.clone())
        .ok_or(ZkManifestError::MissingPackageName(
            cargo_manifest_path_str.clone(),
        ))?;
    let input_order = parse_cargo_manifest_inputs(&cargo_manifest, cargo_manifest_path_str)?;

    Ok((cargo_package_name, input_order))
}

fn build_zkprogram_manifest(
    image_path: &Path,
    keypair: &impl Signer,
    cargo_package_name: String,
    input_order: Vec<String>,
) -> Result<ZkProgramManifest> {
    const RISCV_DOCKER_PATH: &str = "target/riscv-guest/riscv32im-risc0-zkvm-elf/docker";
    const CARGO_RISCZERO_BUILD_ARGS: &[&str; 4] =
        &["risczero", "build", "--manifest-path", "Cargo.toml"];

    let binary_path = image_path
        .join(RISCV_DOCKER_PATH)
        .join(&cargo_package_name)
        .join(&cargo_package_name);
    let output = Command::new(CARGO_COMMAND)
        .current_dir(image_path)
        .args(CARGO_RISCZERO_BUILD_ARGS)
        .env("CARGO_TARGET_DIR", image_path.join(TARGET_DIR))
        .output()?;

    if output.status.success() {
        let elf_contents = fs::read(&binary_path)?;
        let image_id = compute_image_id(&elf_contents).map_err(|err| {
            BonsolCliError::FailedToComputeImageId {
                binary_path: binary_path.to_string_lossy().to_string(),
                err,
            }
        })?;
        let signature = keypair.sign_message(elf_contents.as_slice());
        let zkprogram_manifest = ZkProgramManifest {
            name: cargo_package_name,
            binary_path: binary_path
                .to_str()
                .ok_or(ZkManifestError::InvalidBinaryPath)?
                .to_string(),
            input_order,
            image_id: image_id.to_string(),
            size: elf_contents.len() as u64,
            signature: signature.to_string(),
        };
        return Ok(zkprogram_manifest);
    }

    Err(BonsolCliError::BuildFailure(String::from_utf8_lossy(&output.stderr).to_string()).into())
}
