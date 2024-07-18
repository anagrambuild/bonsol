use std::{env, fs::{self, File}, path::Path};

use anagram_bonsol_sdk::{BonsolClient, ProgramInputType};
use indicatif::ProgressBar;
use object_store::{aws::AmazonS3Builder, ObjectStore};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signer::Signer;
use anyhow::Result;
use crate::{command::{DeployType, S3UploadDestination}, common::ZkProgramManifest};

pub async fn deploy(
  rpc: String,
  keypair: &impl Signer,
  manifest_path: String,
  s3_upload: S3UploadDestination,
  auto_confirm: bool,
  deploy_type: Option<DeployType>,
) -> Result<()> {
  let bar = ProgressBar::new_spinner();
  let rpc_client = RpcClient::new(rpc);
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
          let dest = object_store::path::Path::from(format!(
              "{}-{}",
              manifest.name, manifest.image_id
          ));
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
      _ => {
          bar.finish_and_clear();
          return Err(anyhow::anyhow!("Please provide an upload config"))
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
                  &keypair.pubkey(),
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
          match bonsol_client.send_txn(keypair, deploy_txn, 5).await {
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
