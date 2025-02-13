use {
    crate::CliError,
    figment::{
        providers::{Format, Toml},
        Figment,
    },
    serde::{Deserialize, Serialize},
    std::path::Path,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IngesterConfig {
    RpcBlockSubscription {
        wss_rpc_url: String,
    },
    GrpcSubscription {
        grpc_url: String,
        connection_timeout_secs: u32,
        timeout_secs: u32,
        token: String,
    },
    WebsocketSub, //not implemented
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TransactionSenderConfig {
    Rpc { rpc_url: String },
    //--- below not implemented yet
    Tpu,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SignerConfig {
    KeypairFile { path: String }, //--- below not implemented yet maybe hsm, signer server or some weird sig agg shiz
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub enum MissingImageStrategy {
    #[default]
    DownloadAndClaim,
    DownloadAndMiss,
    Fail,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProverNodeConfig {
    pub env: Option<String>,
    #[serde(default = "default_bonsol_program")]
    pub bonsol_program: String,
    #[serde(default = "default_risc0_image_folder")]
    pub risc0_image_folder: String,
    #[serde(default = "default_risc0_image_folder_limit")]
    pub risc0_image_folder_limit: u32,
    #[serde(default = "default_max_image_size_mb")]
    pub max_image_size_mb: u32,
    #[serde(default = "default_image_compression_ttl_hours")]
    pub image_compression_ttl_hours: u32,
    #[serde(default = "default_max_input_size_mb")]
    pub max_input_size_mb: u32,
    #[serde(default = "default_image_download_timeout_secs")]
    pub image_download_timeout_secs: u32,
    #[serde(default = "default_input_download_timeout_secs")]
    pub input_download_timeout_secs: u32,
    #[serde(default = "default_maximum_concurrent_proofs")]
    pub maximum_concurrent_proofs: u32,
    #[serde(default = "default_ingester_config")]
    pub ingester_config: IngesterConfig,
    #[serde(default = "default_transaction_sender_config")]
    pub transaction_sender_config: TransactionSenderConfig,
    #[serde(default = "default_signer_config")]
    pub signer_config: SignerConfig,
    #[serde(default = "default_stark_compression_tools_path")]
    pub stark_compression_tools_path: String,
    #[serde(default = "default_metrics_config")]
    pub metrics_config: MetricsConfig,
    #[serde(default)]
    pub missing_image_strategy: MissingImageStrategy,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MetricsConfig {
    Prometheus {},
    None,
}

// ... keeping all the default functions unchanged ...

const fn default_metrics_config() -> MetricsConfig {
    MetricsConfig::None
}

fn default_stark_compression_tools_path() -> String {
    std::env::current_dir()
        .unwrap_or(Path::new("./").into())
        .join("stark")
        .to_string_lossy()
        .to_string()
}

fn default_bonsol_program() -> String {
    "BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew".to_string()
}

fn default_risc0_image_folder() -> String {
    "./elf".to_string()
}

const fn default_risc0_image_folder_limit() -> u32 {
    300
}

const fn default_max_image_size_mb() -> u32 {
    10
}

const fn default_image_compression_ttl_hours() -> u32 {
    5
}

const fn default_max_input_size_mb() -> u32 {
    1
}

const fn default_image_download_timeout_secs() -> u32 {
    120
}

const fn default_input_download_timeout_secs() -> u32 {
    30
}

const fn default_maximum_concurrent_proofs() -> u32 {
    100
}

fn default_ingester_config() -> IngesterConfig {
    IngesterConfig::RpcBlockSubscription {
        wss_rpc_url: "ws://localhost:8900".to_string(),
    }
}

fn default_transaction_sender_config() -> TransactionSenderConfig {
    TransactionSenderConfig::Rpc {
        rpc_url: "http://localhost:8899".to_string(),
    }
}

fn default_signer_config() -> SignerConfig {
    SignerConfig::KeypairFile {
        path: "./node-keypair.json".to_string(),
    }
}

impl Default for ProverNodeConfig {
    fn default() -> Self {
        ProverNodeConfig {
            env: Some("dev".to_string()),
            bonsol_program: default_bonsol_program(),
            risc0_image_folder: default_risc0_image_folder(),
            risc0_image_folder_limit: default_risc0_image_folder_limit(),
            max_image_size_mb: default_max_image_size_mb(),
            image_compression_ttl_hours: default_image_compression_ttl_hours(),
            max_input_size_mb: default_max_input_size_mb(),
            image_download_timeout_secs: default_image_download_timeout_secs(),
            input_download_timeout_secs: default_input_download_timeout_secs(),
            maximum_concurrent_proofs: default_maximum_concurrent_proofs(),
            ingester_config: default_ingester_config(),
            transaction_sender_config: default_transaction_sender_config(),
            signer_config: default_signer_config(),
            stark_compression_tools_path: default_stark_compression_tools_path(),
            metrics_config: default_metrics_config(),
            missing_image_strategy: MissingImageStrategy::default(),
        }
    }
}

pub fn load_config(config_path: &str) -> Result<ProverNodeConfig, CliError> {
    let figment = Figment::new().merge(Toml::file(config_path));
    let cfg: ProverNodeConfig = figment
        .extract()
        .map_err(|e| return CliError::InvalidConfig(e.to_string()))?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() -> anyhow::Result<()> {
        let config_content = r#"
 risc0_image_folder = "/elf"
max_input_size_mb = 10
image_download_timeout_secs = 60
input_download_timeout_secs = 60
maximum_concurrent_proofs = 10
max_image_size_mb = 4
image_compression_ttl_hours = 24
stark_compression_tools_path = "./stark/"
env = "dev"

[transaction_sender_config]
Rpc = { rpc_url = "http://localhost:8899" }

[signer_config]
KeypairFile = { path = "node_keypair.json" }"#;
        let config: ProverNodeConfig = toml::from_str(config_content).unwrap();
        assert_eq!(config.risc0_image_folder, "/elf");
        assert_eq!(config.max_input_size_mb, 10);
        assert_eq!(config.image_download_timeout_secs, 60);
        assert_eq!(config.input_download_timeout_secs, 60);
        assert_eq!(config.maximum_concurrent_proofs, 10);
        assert_eq!(config.max_image_size_mb, 4);
        assert_eq!(config.image_compression_ttl_hours, 24);
        assert_eq!(config.stark_compression_tools_path, "./stark/");
        assert_eq!(config.env, Some("dev".to_string()));
        let serialized = toml::to_string(&config)?;
        let deserialized: ProverNodeConfig = toml::from_str(&serialized)?;
        assert_eq!(deserialized.risc0_image_folder, config.risc0_image_folder);
        assert_eq!(deserialized.max_input_size_mb, config.max_input_size_mb);
        assert_eq!(
            deserialized.image_download_timeout_secs,
            config.image_download_timeout_secs
        );
        assert_eq!(
            deserialized.input_download_timeout_secs,
            config.input_download_timeout_secs
        );
        assert_eq!(
            deserialized.maximum_concurrent_proofs,
            config.maximum_concurrent_proofs
        );
        assert_eq!(deserialized.max_image_size_mb, config.max_image_size_mb);
        assert_eq!(
            deserialized.image_compression_ttl_hours,
            config.image_compression_ttl_hours
        );
        assert_eq!(
            deserialized.stark_compression_tools_path,
            config.stark_compression_tools_path
        );
        assert_eq!(deserialized.env, config.env);
        match &config.transaction_sender_config {
            TransactionSenderConfig::Rpc { rpc_url } => {
                assert_eq!(rpc_url, "http://localhost:8899");
            }
            _ => panic!("Expected Rpc transaction sender config"),
        }
        match &config.signer_config {
            SignerConfig::KeypairFile { path } => {
                assert_eq!(path, "node_keypair.json");
            }
        }
        assert_eq!(config.risc0_image_folder, "/elf");
        assert_eq!(config.max_input_size_mb, 10);
        assert_eq!(config.image_download_timeout_secs, 60);
        assert_eq!(config.input_download_timeout_secs, 60);
        assert_eq!(config.maximum_concurrent_proofs, 10);
        assert_eq!(config.max_image_size_mb, 4);
        assert_eq!(config.image_compression_ttl_hours, 24);
        assert_eq!(config.stark_compression_tools_path, "./stark/");
        assert_eq!(config.env, Some("dev".to_string()));
        match config.transaction_sender_config {
            TransactionSenderConfig::Rpc { rpc_url } => {
                assert_eq!(rpc_url, "http://localhost:8899");
            }
            _ => panic!("Expected Rpc transaction sender config"),
        }
        match config.signer_config {
            SignerConfig::KeypairFile { path } => {
                assert_eq!(path, "node_keypair.json");
            }
        }
        Ok(())
    }
}
