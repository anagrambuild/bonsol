use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use bonsol_sdk::{BonsolClient, ProgramInputType};
use byte_unit::{Byte, ByteUnit};
use indicatif::ProgressBar;
use object_store::aws::AmazonS3Builder;
use object_store::ObjectStore;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};

use crate::command::S3UploadDestination;
use crate::common::ZkProgramManifest;

pub async fn deploy(
    rpc: String,
    manifest_path: String,
    s3_upload: S3UploadDestination,
    compute_units: Option<u32>,
    auto_confirm: bool,
) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    let rpc_client = RpcClient::new_with_commitment(rpc.clone(), CommitmentConfig::confirmed());
    let manifest_path = Path::new(&manifest_path);
    let manifest_file = File::open(manifest_path)
        .map_err(|e| anyhow::anyhow!("Error opening manifest file: {:?}", e))?;

    let manifest: ZkProgramManifest = serde_json::from_reader(manifest_file)
        .map_err(|e| anyhow::anyhow!("Error parsing manifest file: {:?}", e))?;
    let loaded_binary = fs::read(&manifest.binary_path)
        .map_err(|e| anyhow::anyhow!("Error loading binary: {:?}", e))?;

    // Handle S3 configuration
    let bucket = s3_upload.bucket;
    let region = s3_upload.region;
    let access_key = s3_upload.access_key;
    let secret_key = s3_upload.secret_key;
    let s3_client = AmazonS3Builder::new()
        .with_bucket_name(bucket.clone())
        .with_region(&region)
        .with_access_key_id(&access_key)
        .with_secret_access_key(&secret_key)
        .build()
        .map_err(|e| anyhow::anyhow!("Error creating S3 client: {:?}", e))?;

    let dest = object_store::path::Path::from(format!("{}-{}", manifest.name, manifest.image_id));
    let destc = dest.clone();
    let exists = s3_client.head(&destc).await.is_ok();
    let url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, dest);

    if exists {
        bar.set_message("File already exists, skipping upload");
    } else {
        match s3_client.put(&destc, loaded_binary.into()).await {
            Ok(_) => {
                bar.finish_and_clear();
            }
            Err(e) => {
                bar.finish_and_clear();
                return Err(anyhow::anyhow!(
                    "Error uploading to {} {:?}",
                    dest.to_string(),
                    e
                ));
            }
        }
    }

    if !auto_confirm {
        bar.finish_and_clear();
        println!("Deploying to Solana, which will cost real money. Are you sure you want to continue? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "y" {
            println!("Aborting");
            return Ok(());
        }
    }

    let bonsol_client = BonsolClient::with_rpc_client(rpc_client);
    let image_id = manifest.image_id.clone();

    match bonsol_client.get_deployment(&image_id).await {
        Ok(Some(_)) => {
            bar.finish_and_clear();
            println!("Deployment already exists, deployments are immutable");
            Ok(())
        }
        Ok(None) => {
            // Get keypair path from environment and handle errors properly
            let keypair_path = env::var("KEYPAIR_PATH")
                .map_err(|e| anyhow::anyhow!("Failed to get KEYPAIR_PATH: {}", e))?;
            let signer = read_keypair_file(&keypair_path)
                .map_err(|e| anyhow::anyhow!("Failed to read keypair file: {}", e))?;

            let deploy_txn = bonsol_client
                .deploy_v1(
                    &signer.pubkey(),
                    &image_id,
                    manifest.size,
                    &manifest.name,
                    &url,
                    compute_units,
                    manifest
                        .input_order
                        .iter()
                        .map(|i| match i.as_str() {
                            "Public" => ProgramInputType::Public,
                            "Private" => ProgramInputType::Private,
                            _ => ProgramInputType::Unknown,
                        })
                        .collect(),
                )
                .await?;

            match bonsol_client.send_txn_standard(&signer, deploy_txn).await {
                Ok(_) => {
                    bar.finish_and_clear();
                    println!("{} deployed", image_id);
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Transaction failed: {}", e)),
            }
        }
        Err(e) => {
            bar.finish_with_message(format!("Error getting deployment: {:?}", e));
            Ok(())
        }
    }
}
