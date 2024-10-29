use std::fs::{self, File};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::common::*;
use anyhow::Result;
use indicatif::ProgressBar;
use risc0_zkvm::compute_image_id;
use solana_sdk::signer::Signer;

pub fn build(keypair: &impl Signer, zk_program_path: String) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    let image_path = Path::new(&zk_program_path);
    // ensure cargo risc0 is installed and has the plugin
    if !cargo_has_plugin("risczero") || !cargo_has_plugin("binstall") || !has_executable("docker") {
        bar.finish_and_clear();
        return Err(anyhow::anyhow!(
            "Please install cargo-risczero and cargo-binstall and docker"
        ));
    }

    let build_result = build_maifest(image_path, &keypair);
    let manifest_path = image_path.join("manifest.json");
    match build_result {
        Err(e) => {
            bar.finish_with_message(format!("Error building image: {:?}", e));
            Ok(())
        }
        Ok(manifest) => {
            serde_json::to_writer_pretty(File::create(&manifest_path).unwrap(), &manifest).unwrap();
            bar.finish_and_clear();
            Ok(())
        }
    }
}

fn build_maifest(
    image_path: &Path,
    keypair: &impl Signer,
) -> Result<ZkProgramManifest, std::io::Error> {
    let manifest_path = image_path.join("Cargo.toml");
    let manifest = cargo_toml::Manifest::from_path(&manifest_path).unwrap();
    let package = manifest
        .package
        .as_ref()
        .map(|p| &p.name)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Cargo.toml",
        ))?;
    let meta = manifest.package.as_ref().and_then(|p| p.metadata.as_ref());
    if meta.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Cargo.toml, missing package metadata",
        ));
    }

    let inputs = meta
        .unwrap()
        .as_table()
        .and_then(|m| m.get("zkprogram"))
        .and_then(|m| m.as_table())
        .and_then(|m| m.get("input_order"))
        .and_then(|m| m.as_array())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Cargo.toml, missing zkprogram metadata",
        ))?;

    let binary_path = image_path
        .join("target/riscv-guest/riscv32im-risc0-zkvm-elf/docker")
        .join(package)
        .join(package);
    let output = Command::new("cargo")
        .current_dir(image_path)
        .arg("risczero")
        .arg("build")
        .arg("--manifest-path")
        .arg("Cargo.toml")
        .env("CARGO_TARGET_DIR", image_path.join("target"))
        .output()?;

    if output.status.success() {
        let elf_contents = fs::read(&binary_path)?;
        let image_id = compute_image_id(&elf_contents)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Invalid image"))?;
        let signature = keypair.sign_message(elf_contents.as_slice());
        let manifest = ZkProgramManifest {
            name: package.to_string(),
            binary_path: binary_path.to_str().unwrap().to_string(),
            input_order: inputs
                .iter()
                .map(|i| i.as_str().unwrap().to_string())
                .collect(),
            image_id: image_id.to_string(),
            size: elf_contents.len() as u64,
            signature: signature.to_string(),
        };
        Ok(manifest)
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("Build failed: {}", error);
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Build failed",
        ))
    }
}
