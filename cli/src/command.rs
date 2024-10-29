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
    pub bucket: String,
    #[arg(long)]
    pub access_key: String,
    #[arg(long)]
    pub secret_key: String,
    #[arg(long)]
    pub region: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Deploy {
        #[arg(short = 'm', long)]
        manifest_path: String,
        #[clap(flatten)]
        s3_upload: S3UploadDestination,
        #[arg(short = 'b', long)]
        compute_units: Option<u32>,
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
