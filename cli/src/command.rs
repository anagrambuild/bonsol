use bonsol_interface::bonsol_schema::input_set_op_v1_generated::InputSetOp;
use clap::{command, ArgGroup, Args, Parser, Subcommand, ValueEnum};

use crate::common::CliInput;

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

pub struct ParsedBonsolCli {
    pub config: Option<String>,

    pub keypair: Option<String>,

    pub rpc_url: Option<String>,

    pub command: ParsedCommand,
}

impl TryFrom<BonsolCli> for ParsedBonsolCli {
    type Error = anyhow::Error;

    fn try_from(value: BonsolCli) -> Result<Self, Self::Error> {
        Ok(Self {
            config: value.config,
            keypair: value.keypair,
            rpc_url: value.rpc_url,
            command: value.command.try_into()?,
        })
    }
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
}

#[derive(Debug, Clone, Args)]
pub struct UrlUploadArgs {
    #[arg(help = "Specify a URL endpoint to deploy to", long, required = true)]
    pub url: String,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DeployType {
    S3,
    ShadowDrive,
    Url,
}

#[derive(Debug, Clone)]
pub enum DeployDestination {
    S3(S3UploadArgs),
    ShadowDrive(ShadowDriveUploadArgs),
    Url(UrlUploadArgs),
}
impl DeployDestination {
    pub fn try_parse(
        deploy_type: DeployType,
        s3: Option<S3UploadArgs>,
        sd: Option<ShadowDriveUploadArgs>,
        url: Option<UrlUploadArgs>,
    ) -> anyhow::Result<Self> {
        match deploy_type {
            // Because we are not supporting a direct mapping (eg, subcommand),
            // it's possible for a user to specify a deployment type and provide the wrong
            // arguments. If we support subcommands in the future this will be
            // much clearer, otherwise we would need to do more validation here
            // to provide better error messages when the wrong args are present.
            DeployType::S3 if s3.is_some() => Ok(Self::S3(s3.unwrap())),
            DeployType::ShadowDrive if sd.is_some() => Ok(Self::ShadowDrive(sd.unwrap())),
            DeployType::Url if url.is_some() => Ok(Self::Url(url.unwrap())),
            _ => anyhow::bail!("The deployment type and its corresponding args do not match, expected args for deployment type '{:?}'", deploy_type),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeployArgs {
    pub dest: DeployDestination,
    pub manifest_path: String,
    pub auto_confirm: bool,
}
impl DeployArgs {
    pub fn parse(dest: DeployDestination, manifest_path: String, auto_confirm: bool) -> Self {
        Self {
            dest,
            manifest_path,
            auto_confirm,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum CliInputSetOp {
    #[command(about = "Create a new set of onchain inputs")]
    Create {
        #[arg(help = "Specify inputs to include in the newly created input set", long = "input", short = 'i', name = "input", value_parser = parse_cli_input)]
        inputs: Vec<CliInput>,
    },

    #[command(about = "Update an onchain input set with new inputs")]
    Update {
        #[arg(help = "Specify inputs to include in the input set", long = "input", short = 'i', name = "input",  value_parser = parse_cli_input)]
        inputs: Vec<CliInput>,
    },

    #[command(about = "Delete inputs from an onchain input set")]
    Delete {
        #[arg(help = "Specify inputs to remove from the input set", long = "input", short = 'i', name = "input", value_parser = parse_cli_input)]
        inputs: Vec<CliInput>,
    },
}

fn parse_cli_input(s: &str) -> Result<CliInput, serde_json::Error> {
    serde_json::from_str(s)
}

impl<'a> From<&'a CliInputSetOp> for InputSetOp {
    fn from(value: &'a CliInputSetOp) -> Self {
        match value {
            CliInputSetOp::Create { .. } => InputSetOp::Create,
            CliInputSetOp::Update { .. } => InputSetOp::Update,
            CliInputSetOp::Delete { .. } => InputSetOp::Delete,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "Deploy a program with various storage options, such as S3, ShadowDrive, or manually with a URL"
    )]
    Deploy {
        #[arg(
            help = "Specify the deployment type",
            short = 't',
            long,
            value_enum,
            required = true
        )]
        deploy_type: DeployType,

        #[command(flatten)]
        s3: Option<S3UploadArgs>,

        #[command(flatten)]
        shadow_drive: Option<ShadowDriveUploadArgs>,

        #[command(flatten)]
        url: Option<UrlUploadArgs>,

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
        #[arg(
            help = "The path to a ZK program folder containing a Cargo.toml",
            short = 'z',
            long
        )]
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
    #[command(about = "Manage JSON input sets for dedicated input set files")]
    InputSet {
        #[command(subcommand)]
        input_set: CliInputSetOp,
    },
}

#[derive(Debug)]
pub enum ParsedCommand {
    Deploy {
        deploy_args: DeployArgs,
    },
    Build {
        zk_program_path: String,
    },
    Execute {
        execution_request_file: Option<String>,

        program_id: Option<String>,

        execution_id: Option<String>,

        expiry: Option<u64>,

        tip: Option<u64>,

        input_file: Option<String>,

        wait: bool,

        timeout: Option<u64>,
    },
    Prove {
        manifest_path: Option<String>,

        program_id: Option<String>,

        input_file: Option<String>,

        execution_id: String,

        output_location: Option<String>,
    },
    Init {
        dir: Option<String>,

        project_name: String,
    },
    InputSet {
        input_set: CliInputSetOp,
    },
}

impl TryFrom<Command> for ParsedCommand {
    type Error = anyhow::Error;

    fn try_from(value: Command) -> Result<Self, Self::Error> {
        match value {
            Command::Deploy {
                deploy_type,
                s3,
                shadow_drive,
                url,
                manifest_path,
                auto_confirm,
            } => Ok(ParsedCommand::Deploy {
                deploy_args: DeployArgs::parse(
                    DeployDestination::try_parse(deploy_type, s3, shadow_drive, url)?,
                    manifest_path,
                    auto_confirm,
                ),
            }),
            Command::Build { zk_program_path } => Ok(ParsedCommand::Build { zk_program_path }),
            Command::Execute {
                execution_request_file,
                program_id,
                execution_id,
                expiry,
                tip,
                input_file,
                wait,
                timeout,
            } => Ok(ParsedCommand::Execute {
                execution_request_file,
                program_id,
                execution_id,
                expiry,
                tip,
                input_file,
                wait,
                timeout,
            }),
            Command::Prove {
                manifest_path,
                program_id,
                input_file,
                execution_id,
                output_location,
            } => Ok(ParsedCommand::Prove {
                manifest_path,
                program_id,
                input_file,
                execution_id,
                output_location,
            }),
            Command::Init { dir, project_name } => Ok(ParsedCommand::Init { dir, project_name }),
            Command::InputSet { input_set } => Ok(ParsedCommand::InputSet { input_set }),
        }
    }
}
