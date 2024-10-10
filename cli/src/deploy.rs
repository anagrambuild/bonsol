use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::str::FromStr;

use crate::command::{DeployType, S3UploadDestination, ShadowDriveUpload, UrlUploadDestination};
use crate::common::ZkProgramManifest;
use anyhow::Result;
use bonsol_sdk::{BonsolClient, ProgramInputType};
use byte_unit::{Byte, ByteUnit};
use indicatif::ProgressBar;
use object_store::aws::AmazonS3Builder;
use object_store::ObjectStore;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::ShadowDriveClient;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::commitment_config::CommitmentConfig;

pub async fn deploy(
    rpc: String,
    signer: &impl shadow_drive_sdk::Signer,
    manifest_path: String,
    s3_upload: S3UploadDestination,
    shadow_drive_upload: ShadowDriveUpload,
    url_upload: UrlUploadDestination,
    auto_confirm: bool,
    deploy_type: Option<DeployType>,
) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    let rpc_url = rpc.clone();
    let rpc_client = RpcClient::new_with_commitment(rpc, CommitmentConfig::confirmed());
    let manifest_path = Path::new(&manifest_path);
    let manifest_file = File::open(manifest_path)
        .map_err(|e| anyhow::anyhow!("Error opening manifest file: {:?}", e))?;

    let manifest: ZkProgramManifest = serde_json::from_reader(manifest_file)
        .map_err(|e| anyhow::anyhow!("Error parsing manifest file: {:?}", e))?;
    let loaded_binary = fs::read(&manifest.binary_path)
        .map_err(|e| anyhow::anyhow!("Error loading binary: {:?}", e))?;
    let url: String = match deploy_type {
        Some(DeployType::S3) => {
            let bucket = s3_upload
                .bucket
                .ok_or(anyhow::anyhow!("Please provide a bucket name"))?;
            let region = s3_upload.region.or_else(|| env::var("AWS_REGION").ok());
            let access_key = s3_upload
                .access_key
                .or_else(|| env::var("AWS_ACCESS_KEY_ID").ok());
            let secret_key = s3_upload
                .secret_key
                .or_else(|| env::var("AWS_SECRET_ACCESS_KEY").ok());
            if region.is_none() || access_key.is_none() || secret_key.is_none() {
                bar.finish_and_clear();
                return Err(anyhow::anyhow!("Invalid AWS credentials"));
            }
            let region = region.unwrap();
            let access_key = access_key.unwrap();
            let secret_key = secret_key.unwrap();
            if bucket == "" {
                bar.finish_and_clear();
                return Err(anyhow::anyhow!("Please provide a bucket name"));
            }
            let s3_client = AmazonS3Builder::new()
                .with_bucket_name(bucket.clone())
                .with_region(&region)
                .with_access_key_id(&access_key)
                .with_secret_access_key(&secret_key)
                .build()
                .map_err(|e| anyhow::anyhow!("Error creating S3 client: {:?}", e))?;
            let dest =
                object_store::path::Path::from(format!("{}-{}", manifest.name, manifest.image_id));
            let destc = dest.clone();
            //get the file to see if it exists
            let exists = s3_client.head(&destc).await.is_ok();
            let url = format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                bucket,
                region,
                dest.to_string()
            );
            if exists {
                bar.set_message("File already exists, skipping upload");
                Ok::<_, anyhow::Error>(url)
            } else {
                let upload = s3_client.put(&destc, loaded_binary.into()).await;
                match upload {
                    Ok(_) => {
                        bar.finish_and_clear();
                        Ok::<_, anyhow::Error>(url)
                    }
                    Err(e) => {
                        bar.finish_and_clear();
                        anyhow::bail!("Error uploading to {} {:?}", dest.to_string(), e)
                    }
                }
            }
        }
        Some(DeployType::ShadowDrive) => {
            let storage_account = shadow_drive_upload
                .storage_account
                .ok_or(anyhow::anyhow!("Please provide a storage account"))?;
            let shadow_drive = ShadowDriveClient::new(signer, &rpc_url);
            let alt_client = if let Some(alt_keypair) = shadow_drive_upload.alternate_keypair {
                Some(ShadowDriveClient::new(
                    read_keypair_file(Path::new(&alt_keypair))
                        .map_err(|e| anyhow::anyhow!("Invalid keypair file: {:?}", e))?,
                    &rpc_url,
                ))
            } else {
                None
            };
            let sa = if storage_account == "create" {
                let name = shadow_drive_upload
                    .storage_account_name
                    .unwrap_or(manifest.name.clone());
                let min = std::cmp::max(((loaded_binary.len() as u64) / 1024 / 1024) * 2, 1);
                let size = shadow_drive_upload.storage_account_size_mb.unwrap_or(min);
                let res = if let Some(alt_client) = &alt_client {
                    alt_client
                        .create_storage_account(
                            &name,
                            Byte::from_unit(size as f64, ByteUnit::MB)
                                .map_err(|e| anyhow::anyhow!("Invalid size: {:?}", e))?,
                            shadow_drive_sdk::StorageAccountVersion::V2,
                        )
                        .await
                        .map_err(|e| anyhow::anyhow!("Error creating storage account: {:?}", e))?
                } else {
                    println!(
                        "Creating storage account with {}MB under the name {} with {}",
                        size,
                        &name,
                        signer.pubkey()
                    );

                    shadow_drive
                        .create_storage_account(
                            &name,
                            Byte::from_unit(size as f64, ByteUnit::MB)
                                .map_err(|e| anyhow::anyhow!("Invalid size: {:?}", e))?,
                            shadow_drive_sdk::StorageAccountVersion::V2,
                        )
                        .await
                        .map_err(|e| anyhow::anyhow!("Error creating storage account: {:?}", e))?
                };
                res.shdw_bucket
                    .ok_or(anyhow::anyhow!("Invalid storage account"))?
            } else {
                storage_account.to_string()
            };

            let name = format!("{}-{}", manifest.name, manifest.image_id);
            let resp = if let Some(alt_client) = alt_client {
                alt_client
                    .store_files(
                        &Pubkey::from_str(&sa)?,
                        vec![ShadowFile::bytes(name, loaded_binary)],
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Error uploading to shadow drive: {:?}", e))?
            } else {
                shadow_drive
                    .store_files(
                        &Pubkey::from_str(&sa)?,
                        vec![ShadowFile::bytes(name, loaded_binary)],
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Error uploading to shadow drive: {:?}", e))?
            };
            bar.finish_and_clear();
            println!("Uploaded to shadow drive");
            Ok::<_, anyhow::Error>(resp.message)
        }
        Some(DeployType::Url) => Ok(url_upload.url),
        _ => {
            bar.finish_and_clear();
            return Err(anyhow::anyhow!("Please provide an upload config"));
        }
    }?;

    if !auto_confirm {
        bar.finish_and_clear();
        println!("Deploying to Solana, which will cost real money. Are you sure you want to continue? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "y" {
            bar.finish_and_clear();
            println!("Aborting");
            return Ok(());
        }
    }
    let bonsol_client = BonsolClient::with_rpc_client(rpc_client);
    let image_id = manifest.image_id.clone();
    let deploy = bonsol_client.get_deployment(&image_id).await;
    match deploy {
        Ok(Some(_)) => {
            bar.finish_and_clear();
            println!("Deployment already exists, deployments are immutable");
            return Ok(());
        }
        Ok(None) => {
            let deploy_txn = bonsol_client
                .deploy_v1(
                    &signer.pubkey(),
                    &image_id,
                    manifest.size,
                    &manifest.name,
                    &url,
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
            match bonsol_client.send_txn_standard(signer, deploy_txn).await {
                Ok(_) => {
                    bar.finish_and_clear();
                    println!("{} deployed", image_id);
                }
                Err(e) => {
                    bar.finish_and_clear();
                    anyhow::bail!(e);
                }
            };
            return Ok(());
        }
        Err(e) => {
            bar.finish_with_message(format!("Error getting deployment: {:?}", e));
            return Ok(());
        }
    };
}
