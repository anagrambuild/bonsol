use clap::{Args, Parser, Subcommand, ValueEnum};

use anagram_bonsol_sdk::{BonsolClient, ProgramInputType};
use indicatif::ProgressBar;
use object_store::{aws::AmazonS3Builder, ObjectStore};
use risc0_zkvm::compute_image_id;
use serde::{Deserialize, Serialize};
use solana_cli_config::{Config, CONFIG_FILE};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;
use std::env;
use std::fs::{self, File};
use std::path::Path;
use std::process::Command as OsCommand;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(version)]
struct BonsolCli {
    #[arg(short = 'c', long)]
    config: Option<String>,
    #[arg(short = 'k', long)]
    keypair: Option<String>,
    #[arg(short = 'u', long)]
    rpc_url: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Args)]
struct S3UploadDestination {
    #[arg(long)]
    bucket: Option<String>,
    #[arg(long)]
    access_key: Option<String>,
    #[arg(long)]
    secret_key: Option<String>,
    #[arg(long)]
    region: Option<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum DeployType {
    S3,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Deploy {
        #[arg(short = 'm', long)]
        manifest_path: String,
        #[arg(short = 't', long)]
        deploy_type: Option<DeployType>,
        #[clap(flatten)]
        s3_upload: S3UploadDestination,
        #[arg(short = 'y', long)]
        auto_confirm: bool,
    },
    Build {
        #[arg(short = 'z', long)]
        zk_program_path: String,
    },
}

fn cargo_has_plugin(plugin_name: &str) -> bool {
    OsCommand::new("cargo")
        .args(&["--list"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.trim().starts_with(plugin_name))
        })
        .unwrap_or(false)
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
    if let None = meta {
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
        .join(&package)
        .join(&package);
    let output = OsCommand::new("cargo")
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

fn has_executable(executable: &str) -> bool {
    OsCommand::new("which")
        .arg(executable)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZkProgramManifest {
    name: String,
    binary_path: String,
    image_id: String,
    input_order: Vec<String>,
    signature: String,
    size: u64,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = BonsolCli::parse();
    let keypair = cli.keypair;
    let config = cli.config;
    let rpc_url = cli.rpc_url;
    let (rpc, kpp) = match (rpc_url, keypair, config) {
        (Some(rpc_url), Some(keypair), None) => (rpc_url, keypair),
        (None, None, config) => {
            let config = Config::load(&config.unwrap_or(CONFIG_FILE.clone().unwrap()));
            match config {
                Ok(config) => (config.json_rpc_url, config.keypair_path),
                Err(e) => {
                    anyhow::bail!("Error loading config: {:?}", e);
                }
            }
        }
        _ => {
            anyhow::bail!("Please provide a keypair and rpc or a solana config file");
        }
    };
    let keypair = read_keypair_file(Path::new(&kpp));
    if keypair.is_err() {
        anyhow::bail!("Invalid keypair");
    }
    let keypair = keypair.unwrap();
    let command = cli.command;
    match command {
        Commands::Build { zk_program_path } => {
            let bar = ProgressBar::new_spinner();
            bar.enable_steady_tick(Duration::from_millis(100));
            let image_path = Path::new(&zk_program_path);
            // ensure cargo risc0 is installed and has the plugin
            if !cargo_has_plugin("risczero")
                || !cargo_has_plugin("binstall")
                || !has_executable("docker")
            {
                bar.finish_with_message(
                    "Please install cargo-risczero and cargo-binstall and docker",
                );
                anyhow::bail!("Please install cargo-risczero and cargo-binstall and docker");
            }

            let build_result = build_maifest(&image_path, &keypair);
            let manifest_path = image_path.join("manifest.json");
            match build_result {
                Err(e) => {
                    bar.finish_with_message(format!("Error building image: {:?}", e));
                    return Ok(());
                }
                Ok(manifest) => {
                    serde_json::to_writer_pretty(File::create(&manifest_path).unwrap(), &manifest)
                        .unwrap();
                }
            }
            bar.finish_with_message(format!(
                "Build complete manifest written at {}",
                manifest_path.display()
            ));
        }
        Commands::Deploy {
            manifest_path,
            s3_upload,
            auto_confirm,
            deploy_type,
        } => {
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
                        anyhow::bail!("Invalid AWS credentials");
                    }
                    let region = region.unwrap();
                    let access_key = access_key.unwrap();
                    let secret_key = secret_key.unwrap();
                    if bucket == "" {
                        bar.finish_and_clear();
                        anyhow::bail!("Please provide a bucket name");
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
                    anyhow::bail!("Please provide an upload config")
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
    };
    Ok(())
}
