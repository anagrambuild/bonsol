use clap::{command, Args, Parser, Subcommand, ValueEnum};

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

#[derive(Debug, Clone, ValueEnum)]
pub enum ProveMode {
    Local,
    Remote,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DeployType {
    S3,
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
        #[arg(short = 'y', long)]
        auto_confirm: bool,
    },
    Build {
        #[arg(short = 'z', long)]
        zk_program_path: String,
    },
    Prove {
        #[arg(short = 'm', long)]
        manifest_path: String,
        #[arg(short = 'l', long)]
        prove_mode: ProveMode,
        #[arg(short, long)]
        inputs: Option<Vec<String>>,
        #[arg(short, long)]
        input_file: Option<String>,
    },
}
