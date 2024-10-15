use crate::common::{proof_get_inputs, ZkProgramManifest};
use anyhow::{anyhow, Result};
use bonsol_prover::image::Image;
use bonsol_prover::prover::{get_risc0_prover, new_risc0_exec_env};
use bonsol_sdk::BonsolClient;
use bytes::Bytes;
use risc0_zkvm::VerifierContext;
use std::fs::{read, File};
use std::io::Write;
use std::path::Path;

pub async fn prove(
    sdk: &BonsolClient,
    execution_id: String,
    manifest_path: Option<String>,
    program_id: Option<String>,
    input_file: Option<String>,
    output_location: Option<String>,
    stdin: Option<String>,
) -> Result<()> {
    let pwd = std::env::current_dir()?;
    let image_bytes = match (&program_id, manifest_path) {
        (Some(i), None) => {
            let bytes: Bytes = sdk.download_program(i).await?;
            Ok(bytes)
        }
        (None, Some(m)) => {
            let manifest_path = Path::new(&m);
            let manifest_file = if manifest_path.is_relative() {
                File::open(pwd.join(manifest_path))?
            } else {
                File::open(manifest_path)?
            };
            let manifest: ZkProgramManifest = serde_json::from_reader(manifest_file)?;
            let binary_path = Path::new(&manifest.binary_path);
            let bytes =
                read(binary_path).map_err(|_| anyhow!("Failed to read binary in manifest file"))?;
            Ok(Bytes::from(bytes))
        }
        _ => Err(anyhow!("Please provide a program id or a manifest path")),
    }?;
    let ext = Path::new(&execution_id).with_extension("bin");
    let output_binary_path = output_location
        .map(|o| Path::new(&o).join(&ext))
        .unwrap_or(ext);
    let image = Image::from_bytes(image_bytes)?;
    let memory_image = image.get_memory_image()?;
    let program_inputs = proof_get_inputs(input_file, stdin)?;
    let mut exec = new_risc0_exec_env(memory_image, program_inputs)?;
    let session = exec.run()?;
    let prover = get_risc0_prover()?;
    let ctx = VerifierContext::default();
    println!("Generating proof");
    let info = prover.prove_session(&ctx, &session);
    match info {
        Ok(proveinfo) => {
            let proof = bincode::serialize(&proveinfo.receipt)?;
            let mut file = File::create(&output_binary_path)?;
            file.write_all(&proof)?;
            println!("Proof written to {}", output_binary_path.to_string_lossy());
        }
        Err(e) => {
            println!("Error generating proof: {:?}", e);
        }
    }
    Ok(())
}
