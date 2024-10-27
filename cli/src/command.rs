use clap::{command, ArgGroup, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
#[command(group(
    // Ensures mutual exclusivity of config, or keypair and rpc_url
    ArgGroup::new("config_group")
        .required(false)
        .args(&["config"])
        .conflicts_with("rpc_url")
        .conflicts_with("keypair")
        .multiple(false)
))]
pub struct BonsolCli {
    #[arg(short = 'c', long)]
    pub config: Option<String>,
    #[arg(short = 'k', long, requires = "rpc_url")]
    pub keypair: Option<String>,
    #[arg(short = 'u', long, requires = "keypair")]
    pub rpc_url: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Args)]
pub struct S3UploadDestination {
    #[arg(
        help = "Specify the S3 bucket name",
        long,
        required = true,
        value_parser = |s: &str| {
            if s.trim().is_empty() {
                anyhow::bail!("expected a non-empty string")
            }
            Ok(s.to_string())
        }
    )]
    pub bucket: String,
    #[arg(
        help = "Specify the AWS access key ID",
        long,
        required = true,
        env = "AWS_ACCESS_KEY_ID"
    )]
    pub access_key: String,
    #[arg(
        help = "Specify the AWS secret access key",
        long,
        required = true,
        env = "AWS_SECRET_ACCESS_KEY"
    )]
    pub secret_key: String,
    #[arg(
        help = "Specify the AWS region",
        long,
        required = true,
        env = "AWS_REGION"
    )]
    pub region: String,
}

#[derive(Debug, Clone, Args)]
pub struct ShadowDriveUploadDestination {
    #[arg(long)]
    pub storage_account: Option<String>,
    #[arg(long)]
    pub storage_account_size_mb: Option<u64>,
    #[arg(long)]
    pub storage_account_name: Option<String>,
    #[arg(long)]
    pub alternate_keypair: Option<String>, // for testing on devnet but deploying to shadow drive
}

#[derive(Debug, Clone, Args)]
pub struct UrlUploadDestination {
    #[arg(value_name = "URL", index = 1, required = true)]
    pub url: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DeployType {
    S3(S3UploadDestination),
    ShadowDrive(ShadowDriveUploadDestination),
    Url(UrlUploadDestination),
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Deploy {
        #[clap(subcommand)]
        deploy_type: DeployType,
        #[arg(short = 'm', long)]
        manifest_path: String,
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
        #[arg(short = 'e', long)]
        execution_id: String,
        #[arg(short = 'o')]
        output_location: Option<String>,
    },
    Init {
        #[arg(short = 'd', long)]
        dir: Option<String>,
        #[arg(short = 'n', long)]
        project_name: String,
    },
}
