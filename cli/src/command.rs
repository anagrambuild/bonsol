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
    #[arg(
        help = "The path to a Solana CLI config [Default: '~/.config/solana/cli/config.yml']",
        short = 'c',
        long
    )]
    pub config: Option<String>,
    #[arg(
        help = "The path to a Solana keypair file [Default: '~/.config/solana/id.json']",
        short = 'k',
        long,
        requires = "rpc_url"
    )]
    pub keypair: Option<String>,
    #[arg(
        help = "The Solana cluster the Solana CLI will make requests to",
        short = 'u',
        long,
        requires = "keypair"
    )]
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
                anyhow::bail!("expected a non-empty string representation of an S3 bucket name")
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
#[command(group(
    // If creating a new account, there's no reason to pass an already existing pubkey
    ArgGroup::new("create_group")
        .required(true) // Ensures that either `create` or `storage_account` is specified
        .args(&["create", "storage_account"])
))]
pub struct ShadowDriveUploadDestination {
    #[arg(help = "Specify a storage account public key", long)]
    pub storage_account: String,
    #[arg(help = "Specify the size of the storage account in MB", long)]
    pub storage_account_size_mb: Option<u64>,
    #[arg(help = "Specify the name of the storage account", long)]
    pub storage_account_name: Option<String>,
    #[arg(
        help = "Specify an alternate keypair for testing on devnet, but deploying to shadow drive",
        long
    )]
    pub alternate_keypair: Option<String>,
    #[arg(help = "Create a new storage account", long)]
    pub create: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UrlUploadDestination {
    #[arg(
        help = "Specify a URL endpoint to deploy to",
        value_name = "URL",
        index = 1,
        required = true
    )]
    pub url: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DeployType {
    #[command(about = "Deploy a program using an AWS S3 bucket")]
    S3(S3UploadDestination),
    #[command(about = "Deploy a program using Shadow Drive")]
    ShadowDrive(ShadowDriveUploadDestination),
    #[command(about = "Deploy a program with a URL")]
    Url(UrlUploadDestination),
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "Deploy a program with various storage options, such as S3, ShadowDrive, or a URL"
    )]
    Deploy {
        #[clap(subcommand)]
        deploy_type: DeployType,
        #[arg(
            help = "The path to the program's manifest file (manifest.json)",
            short = 'm',
            long
        )]
        manifest_path: String,
        #[arg(
            help = "Whether to automatically confirm deployment",
            short = 'y',
            long
        )]
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
