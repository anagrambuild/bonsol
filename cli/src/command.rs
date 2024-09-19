use clap::{command, Args, Parser, Subcommand, ValueEnum};

use crate::common::CliInput;

#[derive(Parser, Debug)]
#[command(version)]
pub struct BonsolCli {
    #[arg(short = 'c', long)]
    pub config: Option<String>,
    #[arg(short = 'k', long)]
    pub keypair: Option<String>,
    #[arg(short = 'u', long)]
    pub rpc_url: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Args)]
pub struct S3UploadDestination {
    #[arg(long)]
    pub bucket: Option<String>,
    #[arg(long)]
    pub access_key: Option<String>,
    #[arg(long)]
    pub secret_key: Option<String>,
    #[arg(long)]
    pub region: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct ShadowDriveUpload {
    #[arg(long)]
    pub storage_account: Option<String>,
    #[arg(long)]
    pub storage_account_size_mb: Option<u64>,
    #[arg(long)]
    pub storage_account_name: Option<String>,
    #[arg(long)]
    pub alternate_keypair: Option<String>, // for testing on devnet but deploying to shadow drive
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DeployType {
    S3,
    ShadowDrive,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Deploy {
        #[arg(short = 'm', long)]
        manifest_path: String,
        #[arg(short = 't', long)]
        deploy_type: Option<DeployType>,
        #[clap(flatten)]
        s3_upload: S3UploadDestination,
        #[clap(flatten)]
        shadow_drive_upload: ShadowDriveUpload,
        #[arg(short = 'y', long)]
        auto_confirm: bool,
    },
    Build {
        #[arg(short = 'z', long)]
        zk_program_path: String,
    },
    Execute {
        #[arg(short = 'f', long)]
        execution_request_file: Option<String>,
        // overridable settings
        #[arg(short = 'p', long)]
        program_id: Option<String>,
        #[arg(short = 'e', long)]
        execution_id: Option<String>,
        #[arg(short = 'x', long)]
        expiry: Option<u64>,
        #[arg(short = 'm', long)]
        tip: Option<u64>,
        #[arg(short = 'i')]
        input_file: Option<String>, // overrides inputs in execution request file
        /// wait for execution to be proven
        #[arg(short = 'w', long)]
        wait: bool,
        /// timeout in seconds
        #[arg(short = 't', long)]
        timeout: Option<u64>,
    },
    Prove {
        #[arg(short = 'm', long)]
        manifest_path: Option<String>,
        #[arg(short = 'p', long)]
        program_id: Option<String>,
        #[arg(short = 'i')]
        input_file: Option<String>, 
    },
    Init {
        #[arg(short = 'd', long)]
        dir: Option<String>,
        #[arg(short = 'n', long)]
        project_name: String,
    }
}
