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
pub struct S3UploadArgs {
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

    #[command(flatten)]
    pub shared_args: SharedDeployArgs,
}

#[derive(Debug, Clone, Args)]
#[command(alias = "sd", group(
    // If creating a new account, there's no reason to pass an already existing pubkey
    ArgGroup::new("create_group")
        .required(true) // Ensures that either `create` or `storage_account` is specified
        .args(&["create", "storage_account"])
        .multiple(false)
))]
pub struct ShadowDriveUploadArgs {
    #[arg(help = "Specify a Shadow Drive storage account public key", long)]
    pub storage_account: Option<String>,

    #[arg(
        help = "Specify the size of the Shadow Drive storage account in MB",
        long
    )]
    pub storage_account_size_mb: Option<u64>,

    #[arg(help = "Specify the name of the Shadow Drive storage account", long)]
    pub storage_account_name: Option<String>,

    #[arg(
        help = "Specify an alternate keypair for testing on devnet, but deploying to Shadow Drive",
        long
    )]
    pub alternate_keypair: Option<String>,

    #[arg(help = "Create a new Shadow Drive storage account", long)]
    pub create: bool,

    #[command(flatten)]
    pub shared_args: SharedDeployArgs,
}

#[derive(Debug, Clone, Args)]
pub struct UrlUploadArgs {
    #[arg(help = "Specify a URL endpoint to deploy to", long, required = true)]
    pub url: String,

    #[command(flatten)]
    pub shared_args: SharedDeployArgs,
}

#[derive(Debug, Clone, Subcommand)]
pub enum DeployArgs {
    #[command(about = "Deploy a program using an AWS S3 bucket")]
    S3(S3UploadArgs),

    #[command(about = "Deploy a program using ShadowDrive")]
    ShadowDrive(ShadowDriveUploadArgs),

    #[command(about = "Deploy a program manually with a URL")]
    Url(UrlUploadArgs),
}

impl DeployArgs {
    pub fn shared_args(&self) -> SharedDeployArgs {
        match self {
            Self::S3(s3) => s3.shared_args.clone(),
            Self::ShadowDrive(sd) => sd.shared_args.clone(),
            Self::Url(url) => url.shared_args.clone(),
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct SharedDeployArgs {
    #[arg(
        help = "The path to the program's manifest file (manifest.json)",
        short = 'm',
        long
    )]
    pub manifest_path: String,

    #[arg(
        help = "Whether to automatically confirm deployment",
        short = 'y',
        long
    )]
    pub auto_confirm: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "Deploy a program with various storage options, such as S3, ShadowDrive, or manually with a URL"
    )]
    Deploy {
        #[clap(subcommand)]
        deploy_args: DeployArgs,
    },

    #[command(about = "Build a ZK program")]
    Build {
        #[arg(
            help = "The path to a ZK program folder containing a Cargo.toml",
            short = 'z',
            long
        )]
        zk_program_path: String,
    },

    #[command(about = "Estimate the execution cost of a ZK RISC0 program")]
    Estimate {
        #[arg(
            help = "The path to the program's manifest file (manifest.json)",
            short = 'm',
            long
        )]
        manifest_path: String,

        #[arg(help = "The path to the program input file", short = 'i', long)]
        input_file: Option<String>,

        #[arg(
            help = "Set the maximum number of cycles [default: 16777216u64]",
            short = 'c',
            long
        )]
        max_cycles: Option<u64>,
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

        #[arg(short = 'i', long, help = "override inputs in execution request file")]
        input_file: Option<String>,

        /// wait for execution to be proven
        #[arg(short = 'w', long, help = "wait for execution to be proven")]
        wait: bool,

        /// timeout in seconds
        #[arg(short = 't', long, help = "timeout in seconds")]
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

    #[command(about = "Initialize a new project")]
    Init {
        #[arg(short = 'd', long)]
        dir: Option<String>,

        #[arg(short = 'n', long)]
        project_name: String,
    },
}
