use crate::common::{get_inputs, transform_inputs, ZkProgramManifest};
use anyhow::Result;
use bonsol_sdk::{image::Image, input_resolver::{DefaultInputResolver, InputResolver}, prover::new_risc0_exec_env, BonsolClient};
use bytes::Bytes;
use risc0_binfmt::MemoryImage;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use std::{
    fs::{read, File},
    io::Read,
    path::Path, sync::Arc,
};

pub async fn prove(
    sdk: &BonsolClient,
    rpc_url: String,
    manifest_path: Option<String>,
    program_id: Option<String>,
    input_file: Option<String>,
    stdin: Option<String>,
) -> Result<()> {
    let image_bytes = match (&program_id, manifest_path) {
        (Some(i), None) => {
            let bytes: Bytes = sdk.download_program(&i).await?;
            Ok(bytes)
        }
        (None, Some(m)) => {
            let manifest_file = File::open(Path::new(&mp))?;
            let manifest: ZkProgramManifest = serde_json::from_reader(manifest_file)?;
            let binary_path = Path::new(&manifest.binary_path);
            let bytes = read(binary_path)?;
            Ok(Bytes::from(bytes))
        }
        _ => Err(anyhow::anyhow!(
            "Please provide a program id or a manifest path"
        )),
    }?;
    let image = Image::from_bytes(image_bytes)?;
    let memory_image = image.get_memory_image()?;
    let cli_inputs = get_inputs(input_file, None)?;
    let inputs = transform_inputs(cli_inputs)?;
    let input_resolver = DefaultInputResolver::new(
        Arc::new(reqwest::Client::new()),
        Arc::new(RpcClient::new(rpc_url)),
    );
    let mut program_inputs = input_resolver.resolve_public_inputs(inputs).await?;
    input_resolver.resolve_private_inputs(execution_id, program_inputs, signer);
    let mut exec = new_risc0_exec_env(memory_image, program_inputs)?;

    Ok(())
}
