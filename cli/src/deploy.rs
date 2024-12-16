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
use shadow_drive_sdk::{Keypair, ShadowDriveClient, Signer};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;

use crate::command::{DeployArgs, DeployDestination, S3UploadArgs, ShadowDriveUploadArgs};
use crate::common::ZkProgramManifest;
use crate::error::{BonsolCliError, S3ClientError, ShadowDriveClientError, ZkManifestError};

pub async fn deploy(rpc_url: String, signer: Keypair, deploy_args: DeployArgs) -> Result<()> {
    let bar = ProgressBar::new_spinner();
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
    let DeployArgs {
        dest,
        manifest_path,
        auto_confirm,
    } = deploy_args;

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
        BonsolCliError::ZkManifestError(ZkManifestError::FailedToLoadBinary {
            binary_path: manifest.binary_path.clone(),
            err,
        })
    })?;
    let url: String = match dest {
        DeployDestination::S3(s3_upload) => {
            let S3UploadArgs {
                bucket,
                access_key,
                secret_key,
                region,
                endpoint,
                ..
            } = s3_upload;

            let dest =
            object_store::path::Path::from(format!("{}-{}", manifest.name, manifest.image_id));

            let url = if let Some(endpoint) = endpoint {
                format!("{}", endpoint)
            } else {
                format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, dest)
            };

            let s3_client = AmazonS3Builder::new()
                .with_bucket_name(&bucket)
                .with_region(&region)
                .with_access_key_id(&access_key)
                .with_secret_access_key(&secret_key)
                .with_url(&url)
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
            println!("Uploaded to S3 url {}", url);
            url
        }
        DeployDestination::ShadowDrive(shadow_drive_upload) => {
            let ShadowDriveUploadArgs {
                storage_account,
                storage_account_size_mb,
                storage_account_name,
                alternate_keypair,
                create,
                ..
            } = shadow_drive_upload;

            let alternate_keypair = alternate_keypair
                .map(|alt_keypair| -> anyhow::Result<Keypair> {
                    read_keypair_file(Path::new(&alt_keypair)).map_err(|err| {
                        BonsolCliError::FailedToReadKeypair {
                            file: alt_keypair,
                            err: format!("{err:?}"),
                        }
                        .into()
                    })
                })
                .transpose()?;
            let wallet = alternate_keypair.as_ref().unwrap_or(&signer);
            let wallet_pubkey = wallet.pubkey();

            let client = ShadowDriveClient::new(wallet, &rpc_url);

            let storage_account = if create {
                let name = storage_account_name.unwrap_or(manifest.name.clone());
                let min = std::cmp::max(((loaded_binary.len() as u64) / 1024 / 1024) * 2, 1);
                let size = storage_account_size_mb.unwrap_or(min);

                println!(
                    "Creating storage account with {}MB under the name '{}' with signer pubkey {}",
                    size, &name, wallet_pubkey
                );
                let storage_account = client
                    .create_storage_account(
                        &name,
                        Byte::from_unit(size as f64, ByteUnit::MB).map_err(|err| {
                            BonsolCliError::ShadowDriveClientError(
                                ShadowDriveClientError::ByteError {
                                    size: size as f64,
                                    err,
                                },
                            )
                        })?,
                        shadow_drive_sdk::StorageAccountVersion::V2,
                    )
                    .await
                    .map_err(|err| {
                        BonsolCliError::ShadowDriveClientError(
                            ShadowDriveClientError::StorageAccountCreationFailed {
                                name: name.clone(),
                                signer: wallet_pubkey,
                                size,
                                err,
                            },
                        )
                    })?
                    .shdw_bucket
                    .ok_or(BonsolCliError::ShadowDriveClientError(
                        ShadowDriveClientError::InvalidStorageAccount {
                            name,
                            signer: wallet_pubkey,
                            size,
                        },
                    ))?;

                println!("Created new storage account with public key: {storage_account}");
                storage_account
            } else {
                // cli parsing prevents both `create` and `storage_account` to be passed simultaneously
                // and require at least one or the other is passed, making this unwrap safe.
                storage_account.unwrap().to_string()
            };

            let name = format!("{}-{}", manifest.name, manifest.image_id);
            let resp = client
                .store_files(
                    &Pubkey::from_str(&storage_account)?,
                    vec![ShadowFile::bytes(name.clone(), loaded_binary)],
                )
                .await
                .map_err(|err| {
                    BonsolCliError::ShadowDriveClientError(ShadowDriveClientError::UploadFailed {
                        storage_account,
                        name: manifest.name.clone(),
                        binary_path: manifest.binary_path,
                        err,
                    })
                })?;

            bar.finish_and_clear();
            println!("Uploaded to shadow drive");
            resp.message
        }
        DeployDestination::Url(url_upload) => {
            let req = reqwest::get(&url_upload.url).await?;
            let bytes = req.bytes().await?;
            if bytes != loaded_binary {
                return Err(BonsolCliError::OriginBinaryMismatch {
                    url: url_upload.url,
                    binary_path: manifest.binary_path,
                }
                .into());
            }

            bar.finish_and_clear();
            url_upload.url
        }
    };

    if !auto_confirm {
        bar.finish_and_clear();
        println!("Deploying to Solana, which will cost real money. Are you sure you want to continue? (y/n)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let response = input.trim();
        if response != "y" {
            bar.finish_and_clear();
            println!("Response: {response}\nAborting...");
            return Ok(());
        }
    }
    let bonsol_client = BonsolClient::with_rpc_client(rpc_client);
    let image_id = manifest.image_id;
    let deploy = bonsol_client.get_deployment(&image_id).await;
    match deploy {
        Ok(Some(account)) => {
            bar.finish_and_clear();
            println!(
                "Deployment for account '{}' already exists, deployments are immutable",
                account.owner
            );
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
            if let Err(err) = bonsol_client.send_txn_standard(signer, deploy_txn).await {
                bar.finish_and_clear();
                anyhow::bail!(err)
            }

            bar.finish_and_clear();
            println!("{} deployed", image_id);
            Ok(())
        }
        Err(e) => {
            bar.finish_with_message(format!("Error getting deployment: {:?}", e));
            Ok(())
        }
    }
}
