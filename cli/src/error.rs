use object_store::Error as S3Error;
use serde_json::Error as SerdeJsonError;
use std::io::Error as IoError;
use thiserror::Error as DeriveError;

pub(crate) const DEFAULT_SOLANA_CONFIG_PATH: &str = ".config/solana/cli/config.yml";
pub(crate) const SOLANA_CONFIG_DOCS_URL: &str =
    "https://solana.com/docs/intro/installation#solana-config";

#[derive(Debug, DeriveError)]
pub enum BonsolCliError {
    #[error(transparent)]
    ParseConfigError(#[from] ParseConfigError),

    #[error("Failed to read keypair from file '{file}': {err}")]
    FailedToReadKeypair { file: String, err: String },

    #[error("Account '{0}' does not have any SOL to pay for the transaction(s)")]
    InsufficientFundsForTransactions(String),

    #[error(transparent)]
    ZkManifestError(#[from] ZkManifestError),

    #[error(transparent)]
    S3ClientError(#[from] S3ClientError),
}

#[derive(Debug, DeriveError, Clone)]
pub enum ParseConfigError {
    #[error("")]
    Uninitialized,

    #[error("The provided solana cli config path '{path:?}' does not exist")]
    ConfigNotFound { path: String },

    #[error("The default solana cli config path '/home/{whoami}/{DEFAULT_SOLANA_CONFIG_PATH}' does not exist.")]
    DefaultConfigNotFound { whoami: String },

    #[error("Failed to load solana cli config at '{path}': {err}")]
    FailedToLoad { path: String, err: String },
}
impl ParseConfigError {
    pub(crate) fn context(&self, whoami: Option<String>) -> String {
        match self {
            Self::ConfigNotFound { .. } => format!("The solana cli config path was invalid, please double check that the path is correct and try again.\nTip: Try using an absolute path."),
            Self::DefaultConfigNotFound { .. } => format!(
"The default solana cli config path is used when no other options for deriving the RPC URL and keypair file path are provided, ie. '--rpc_url' and '--keypair', or a path to a config that isn't at the default location, ie '--config'.
Tip: Try running 'solana config get'. If you have a custom config path set, double check that the default path also exists. A custom config path can be passed to bonsol with the '--config' option, eg. 'bonsol --config /path/to/config.yml'.

For more information on the solana cli config see: {}",
                SOLANA_CONFIG_DOCS_URL
            ),
            Self::FailedToLoad { path, .. } => {
                if let Some(whoami) = whoami {
                    let default_path = format!("/home/{}/{}", whoami, DEFAULT_SOLANA_CONFIG_PATH);
                    if path == &default_path {
                        return format!(
"The default solana cli config path is used when no other options for deriving the RPC URL and keypair file path are provided, ie. '--rpc_url' and '--keypair', or a path to a config that isn't at the default location, ie '--config'.
Tip: Try running 'solana config get'. This will give you information about your current config. If for whatever reason the keypair or RPC URL are missing, please follow the instructions below and try again.

- To generate a new keypair at the default path: 'solana-keygen new'
- To set the RPC URL, select a cluster. For instance, 'mainnet-beta': 'solana config set --url mainnet-beta'

For more information on the solana cli config see: {}",
                            SOLANA_CONFIG_DOCS_URL
                        );
                    }
                }
                format!(
"The config at '{}' exists, but there was a problem parsing it into what bonsol needs, ie. a keypair file and RPC URL.
Tip: Try running 'solana config get'. This will give you information about your current config. If for whatever reason the keypair or RPC URL are missing, please follow the instructions below and try again.

- To generate a new keypair at the default path: 'solana-keygen new'
- To set the RPC URL, select a cluster. For instance, 'mainnet-beta': 'solana config set --url mainnet-beta'

For more information on the solana cli config see: {}",
                    path,
                    SOLANA_CONFIG_DOCS_URL
                )
            },
            Self::Uninitialized => unreachable!(),
        }
    }
}

#[derive(Debug, DeriveError)]
pub enum ZkManifestError {
    #[error("Failed to open manifest at '{manifest_path}': {err:?}")]
    FailedToOpen { manifest_path: String, err: IoError },

    #[error("Failed to deserialize json manifest at '{manifest_path}': {err:?}")]
    FailedDeserialization {
        manifest_path: String,
        err: SerdeJsonError,
    },

    #[error("Failed to load binary from manifest at '{binary_path}': {err:?}")]
    FailedToLoad { binary_path: String, err: IoError },
}

#[derive(Debug, DeriveError)]
pub enum S3ClientError {
    #[error("Failed to build S3 client with the following args:\n{}\n\n{err:?}", args.join(",\n"))]
    FailedToBuildClient { args: Vec<String>, err: S3Error },

    #[error("Failed to upload to '{dest}': {err:?}")]
    UploadFailed {
        dest: object_store::path::Path,
        err: S3Error,
    },
}
