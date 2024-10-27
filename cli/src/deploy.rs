use std::fs::{self, File};
use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use bonsol_sdk::{BonsolClient, ProgramInputType};
use byte_unit::{Byte, ByteUnit};
use indicatif::ProgressBar;
use object_store::aws::AmazonS3Builder;
use object_store::ObjectStore;
use shadow_drive_sdk::models::ShadowFile;
use shadow_drive_sdk::ShadowDriveClient;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;

use crate::command::{DeployType, S3UploadDestination};
use crate::common::ZkProgramManifest;
use crate::error::{BonsolCliError, S3ClientError, ZkManifestError};

pub async fn deploy(
    rpc_url: String,
    signer: &impl shadow_drive_sdk::Signer,
    manifest_path: String,
    auto_confirm: bool,
    deploy_type: DeployType,
) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    let manifest_file = File::open(Path::new(&manifest_path)).map_err(|err| {
        BonsolCliError::ZkManifestError(ZkManifestError::FailedToOpen {
            manifest_path: manifest_path.clone(),
            err,
        })
    })?;

    let manifest: ZkProgramManifest = serde_json::from_reader(manifest_file).map_err(|err| {
        BonsolCliError::ZkManifestError(ZkManifestError::FailedDeserialization {
            manifest_path,
            err,
        })
    })?;
    let loaded_binary = fs::read(&manifest.binary_path).map_err(|err| {
        BonsolCliError::ZkManifestError(ZkManifestError::FailedToLoad {
            binary_path: manifest.binary_path,
            err,
        })
    })?;
    let url: String = match deploy_type {
        DeployType::S3(s3_upload) => {
            let S3UploadDestination {
                bucket,
                access_key,
                secret_key,
                region,
            } = s3_upload;

            let s3_client = AmazonS3Builder::new()
                .with_bucket_name(&bucket)
                .with_region(&region)
                .with_access_key_id(&access_key)
                .with_secret_access_key(&secret_key)
                .build()
                .map_err(|err| {
                    BonsolCliError::S3ClientError(S3ClientError::FailedToBuildClient {
                        args: vec![
                            format!("bucket: {bucket}"),
                            format!("access_key: {access_key}"),
                            format!(
                                "secret_key: {}..{}",
                                &secret_key[..4],
                                &secret_key[secret_key.len() - 4..]
                            ),
                            format!("region: {region}"),
                        ],
                        err,
                    })
                })?;

            let dest =
                object_store::path::Path::from(format!("{}-{}", manifest.name, manifest.image_id));
            let url = format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, dest);
            // get the file to see if it exists
            if s3_client.head(&dest).await.is_ok() {
                bar.set_message("File already exists, skipping upload");
            } else {
                s3_client
                    .put(&dest, loaded_binary.into())
                    .await
                    .map_err(|err| {
                        BonsolCliError::S3ClientError(S3ClientError::UploadFailed { dest, err })
                    })?;
            }

            bar.finish_and_clear();
            Ok(url)
        }
        DeployType::ShadowDrive(shadow_drive_upload) => {
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
        DeployType::Url(url_upload) => {
            let req = reqwest::get(&url_upload.url).await?;
            let bytes = req.bytes().await?;
            if bytes != loaded_binary {
                return Err(anyhow::anyhow!("The binary uploaded does not match the local binary, check that the url is correct"));
            }
            bar.finish_and_clear();
            Ok(url_upload.url)
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
            Ok(())
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
            Ok(())
        }
        Err(e) => {
            bar.finish_with_message(format!("Error getting deployment: {:?}", e));
            Ok(())
        }
    }
}
