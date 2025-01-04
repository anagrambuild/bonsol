use std::io::Error as IoError;

use byte_unit::ByteError;
use cargo_toml::Error as CargoManifestError;
use object_store::Error as S3Error;
use serde_json::Error as SerdeJsonError;
use shadow_drive_sdk::error::Error as ShdwDriveError;
use shadow_drive_sdk::Pubkey;
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
    InsufficientFunds(String),

    #[error(transparent)]
    ZkManifestError(#[from] ZkManifestError),

    #[error("Build failed: the following errors were captured from stderr:\n\n{0}")]
    BuildFailure(String),

    #[error("Failed to compute an image ID from binary at path '{binary_path}': {err:?}")]
    FailedToComputeImageId {
        binary_path: String,
        err: anyhow::Error,
    },

    #[error("{0}")]
    MissingInputs(String),

    #[error("Attempt to augment existing input set without specifying its id")]
    InputSetExists,

    #[error(transparent)]
    S3ClientError(#[from] S3ClientError),

    #[error(transparent)]
    ShadowDriveClientError(#[from] ShadowDriveClientError),

    #[error("The binary uploaded does not match the local binary at path '{binary_path}', is the URL correct?\nupload_url: {url}")]
    OriginBinaryMismatch { url: String, binary_path: String },

    #[error("The following build dependencies are missing: {}", missing_deps.join(", "))]
    MissingBuildDependencies { missing_deps: Vec<String> },
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

    #[error(
        "Failed to produce zkprogram image binary path: Image binary path contains non-UTF8 encoded characters"
    )]
    InvalidBinaryPath,

    #[error("Failed to load binary from manifest at '{binary_path}': {err:?}")]
    FailedToLoadBinary { binary_path: String, err: IoError },

    #[error("Program path {0} does not contain a Cargo.toml")]
    MissingManifest(String),

    #[error("Failed to load manifest at '{manifest_path}': {err:?}")]
    FailedToLoadManifest {
        manifest_path: String,
        err: CargoManifestError,
    },

    #[error("Expected '{name}' to be a table at '{manifest_path}'")]
    ExpectedTable { manifest_path: String, name: String },

    #[error("Expected '{name}' to be an array at '{manifest_path}'")]
    ExpectedArray { manifest_path: String, name: String },

    #[error("Manifest at '{0}' does not contain a package name")]
    MissingPackageName(String),

    #[error("Manifest at '{0}' does not contain a package metadata field")]
    MissingPackageMetadata(String),

    #[error("Manifest at '{manifest_path}' has a metadata table that is missing a zkprogram metadata key: meta: {meta:?}")]
    MissingProgramMetadata {
        manifest_path: String,
        meta: cargo_toml::Value,
    },

    #[error("Manifest at '{manifest_path}' has a zkprogram metadata table that is missing a input_order key: zkprogram: {zkprogram:?}")]
    MissingInputOrder {
        manifest_path: String,
        zkprogram: cargo_toml::Value,
    },

    #[error("Failed to parse input: Input contains non-UTF8 encoded characters: {0}")]
    InvalidInput(cargo_toml::Value),

    #[error("Failed to parse the following inputs at '{manifest_path}': {}", errs.join("\n"))]
    InvalidInputs {
        manifest_path: String,
        errs: Vec<String>,
    },
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

#[derive(Debug, DeriveError)]
pub enum ShadowDriveClientError {
    #[error(
        "Failed to produce a valid byte representation for the given size ({size}_f64): {err:?}"
    )]
    ByteError { size: f64, err: ByteError },

    #[error("Shadow Drive storage account creation failed for account with {size}MB under the name '{name}' with signer pubkey {signer}: {err:?}")]
    StorageAccountCreationFailed {
        name: String,
        signer: Pubkey,
        size: u64,
        err: ShdwDriveError,
    },

    #[error("A Shadow Drive storage account was created without a valid bucket:\n\nsize: {size}MB\nname: {name}\nsigner_pubkey: {signer}")]
    InvalidStorageAccount {
        name: String,
        signer: Pubkey,
        size: u64,
    },

    #[error("Failed to upload binary at '{binary_path}' to Shadow Drive account '{storage_account}' under the name '{name}': {err:?}")]
    UploadFailed {
        storage_account: String,
        name: String,
        binary_path: String,
        err: ShdwDriveError,
    },
}
