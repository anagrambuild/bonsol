use std::str::from_utf8;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use arrayref::array_ref;
use async_trait::async_trait;
use bonsol_schema::{InputT, InputType, ProgramInputType};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use tokio::task::{JoinHandle, JoinSet};

use crate::util::get_body_max_size;

#[derive(Debug, Clone, PartialEq)]
pub enum ProgramInput {
    Empty,
    Resolved(ResolvedInput),
    Unresolved(UnresolvedInput),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnresolvedInput {
    pub index: u8,
    pub url: Url,
    pub input_type: ProgramInputType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedInput {
    pub index: u8,
    pub data: Vec<u8>,
    pub input_type: ProgramInputType,
}

impl ProgramInput {
    pub fn index(&self) -> u8 {
        match self {
            ProgramInput::Resolved(ri) => ri.index,
            ProgramInput::Unresolved(ui) => ui.index,
            _ => 0,
        }
    }
}

/// Input resolvers are responsible for downloading and resolving inputs
/// Private inputs must be resoloved post claim and therefore are seperated from public inputs
/// Public inputs are resolved in parallel and are resolved as soon as possible, Private inputs are currently always remote.
/// The output of resolve_public_inputs is a vec of ProgramInputs and that must be passed to the private input resolver if any private inputs are present in the excecution request
#[async_trait]
pub trait InputResolver: Send + Sync {
    /// Returns true if the input resolver supports the input type
    fn supports(&self, input_type: InputType) -> bool;
    /// Resolves public inputs by parsing them or if remote downloading them
    async fn resolve_public_inputs(
        &self,
        inputs: Vec<InputT>,
    ) -> Result<Vec<ProgramInput>, anyhow::Error>;

    /// Resolves private inputs by sigining the request and attempting to download the inputs
    async fn resolve_private_inputs(
        &self,
        execution_id: &str,
        inputs: &mut Vec<ProgramInput>,
        signer: Arc<&(dyn Signer + Send + Sync)>,
    ) -> Result<(), anyhow::Error>;
}

// naive resolver that downloads inputs just in time
pub struct DefaultInputResolver {
    http_client: Arc<reqwest::Client>,
    solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
    max_input_size_mb: u32,
    timeout: Duration,
}

impl DefaultInputResolver {
    pub fn new(
        http_client: Arc<reqwest::Client>,
        solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
    ) -> Self {
        DefaultInputResolver {
            http_client,
            solana_rpc_client,
            max_input_size_mb: 10,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn new_with_opts(
        http_client: Arc<reqwest::Client>,
        solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
        max_input_size_mb: Option<u32>,
        timeout: Option<Duration>,
    ) -> Self {
        DefaultInputResolver {
            http_client,
            solana_rpc_client,
            max_input_size_mb: max_input_size_mb.unwrap_or(10),
            timeout: timeout.unwrap_or(Duration::from_secs(30)),
        }
    }

    fn par_resolve_input(
        &self,
        client: Arc<reqwest::Client>,
        index: u8,
        input: InputT,
        task_set: &mut JoinSet<Result<ResolvedInput>>,
    ) -> Result<ProgramInput> {
        match input.input_type {
            InputType::PublicUrl => {
                let url = input.data.ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                task_set.spawn(download_public_input(
                    client,
                    index,
                    url.clone(),
                    self.max_input_size_mb as usize,
                    ProgramInputType::Public,
                    self.timeout,
                ));
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index,
                    url,
                    input_type: ProgramInputType::Public,
                }))
            }
            InputType::Private => {
                let url = input.data.ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index,
                    url,
                    input_type: ProgramInputType::Private,
                }))
            }
            InputType::PublicData => {
                let data = input.data.ok_or(anyhow::anyhow!("Invalid data"))?;
                let data = data.to_vec();
                Ok(ProgramInput::Resolved(ResolvedInput {
                    index,
                    data,
                    input_type: ProgramInputType::Public,
                }))
            }
            InputType::PublicProof => {
                let url = input.data.ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                task_set.spawn(download_public_input(
                    client,
                    index,
                    url.clone(),
                    self.max_input_size_mb as usize,
                    ProgramInputType::PublicProof,
                    self.timeout,
                ));
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index,
                    url,
                    input_type: ProgramInputType::PublicProof,
                }))
            }
            InputType::PublicAccountData => {
                let pubkey = input.data.ok_or(anyhow::anyhow!("Invalid data"))?;
                if pubkey.len() != 32 {
                    return Err(anyhow::anyhow!("Invalid pubkey"));
                }
                let pubkey = Pubkey::new_from_array(*array_ref!(pubkey, 0, 32));
                let rpc_client_clone = self.solana_rpc_client.clone();
                task_set.spawn(download_public_account(
                    rpc_client_clone,
                    index,
                    pubkey,
                    self.max_input_size_mb as usize,
                ));
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index,
                    url: format!("solana://{}", pubkey).parse()?,
                    input_type: ProgramInputType::Public,
                }))
            }
            _ => {
                // not implemented yet / or unknown
                Err(anyhow::anyhow!("Invalid input type"))
            }
        }
    }
}

#[async_trait]
impl InputResolver for DefaultInputResolver {
    fn supports(&self, input_type: InputType) -> bool {
        match input_type {
            InputType::PublicUrl => true,
            InputType::PublicData => true,
            InputType::PublicAccountData => true,
            InputType::Private => true,
            InputType::PublicProof => true,
            _ => false,
        }
    }

    async fn resolve_public_inputs(
        &self,
        inputs: Vec<InputT>,
    ) -> Result<Vec<ProgramInput>, anyhow::Error> {
        let mut url_set = JoinSet::new();
        let mut res = vec![ProgramInput::Empty; inputs.len()];
        for (index, input) in inputs.into_iter().enumerate() {
            let client = self.http_client.clone();
            res[index] =
                self.par_resolve_input(client, index as u8, input, &mut url_set)?;
        }
        while let Some(url) = url_set.join_next().await {
            match url {
                Ok(Ok(ri)) => {
                    let index = ri.index as usize;
                    res[index] = ProgramInput::Resolved(ri);
                }
                e => {
                    return Err(anyhow::anyhow!("Error downloading input: {:?}", e));
                }
            }
        }
        Ok(res)
    }

    async fn resolve_private_inputs(
        &self,
        execution_id: &str,
        inputs: &mut Vec<ProgramInput>,
        signer: Arc<&(dyn Signer + Send + Sync)>,
    ) -> Result<(), anyhow::Error> {
        let mut url_set = JoinSet::new();
        for (index, input) in inputs.iter().enumerate() {
            let client = self.http_client.clone();
            if let ProgramInput::Unresolved(ui) = input {
                let pir = PrivateInputRequest {
                    identity: signer.pubkey(),
                    claim_id: execution_id.to_string(),
                    input_index: ui.index,
                    now_utc: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                };
                let pir_str = serde_json::to_string(&pir)?;
                let claim_authorization = signer.sign_message(pir_str.as_bytes());
                url_set.spawn(download_private_input(
                    client,
                    index as u8,
                    ui.url.clone(),
                    self.max_input_size_mb as usize,
                    pir_str,
                    claim_authorization.to_string(), // base58 encoded string
                    self.timeout,
                ));
            }
        }
        while let Some(url) = url_set.join_next().await {
            match url {
                Ok(Ok(ri)) => {
                    let index = ri.index as usize;
                    inputs[index] = ProgramInput::Resolved(ri);
                }
                e => {
                    return Err(anyhow::anyhow!("Error downloading input: {:?}", e));
                }
            }
        }
        Ok(())
    }
}

pub fn resolve_public_data(index: usize, data: &[u8]) -> Result<ProgramInput> {
    let data = data.to_vec();
    Ok(ProgramInput::Resolved(ResolvedInput {
        index: index as u8,
        data,
        input_type: ProgramInputType::Public,
    }))
}

pub fn resolve_remote_public_data(
    client: Arc<reqwest::Client>,
    max_input_size_mb: u64,
    index: usize,
    data: &[u8],
    timeout: Duration,
) -> Result<JoinHandle<Result<ResolvedInput>>> {
    let url = from_utf8(data)?;
    let url = Url::parse(url)?;
    Ok(tokio::task::spawn(download_public_input(
        client,
        index as u8,
        url,
        max_input_size_mb as usize,
        ProgramInputType::Public,
        timeout,
    )))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInputRequest {
    identity: Pubkey,
    claim_id: String,
    input_index: u8,
    now_utc: u64,
}

async fn download_public_input(
    client: Arc<reqwest::Client>,
    index: u8,
    url: Url,
    max_size_mb: usize,
    input_type: ProgramInputType,
    timeout: Duration,
) -> Result<ResolvedInput> {
    let resp = client
        .get(url)
        .timeout(timeout)
        .send()
        .await?
        .error_for_status()?;
    let byte = get_body_max_size(resp.bytes_stream(), max_size_mb * 1024 * 1024).await?;
    Ok(ResolvedInput {
        index,
        data: byte.to_vec(),
        input_type,
    })
}

async fn download_public_account(
    solana_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
    index: u8,
    pubkey: Pubkey,
    max_size_mb: usize,
) -> Result<ResolvedInput> {
    let resp = solana_client.get_account_data(&pubkey).await?;
    if resp.len() > max_size_mb * 1024 * 1024 {
        return Err(anyhow::anyhow!("Max size exceeded"));
    }
    Ok(ResolvedInput {
        index,
        data: resp,
        input_type: ProgramInputType::Public,
    })
}

async fn download_private_input(
    client: Arc<reqwest::Client>,
    index: u8,
    url: Url,
    max_size_mb: usize,
    body: String,
    claim_authorization: String,
    timeout: Duration,
) -> Result<ResolvedInput> {
    let resp = client
        .post(url)
        .body(body)
        .timeout(timeout)
        // Signature of the json payload
        .header("Authorization", format!("Bearer {}", claim_authorization))
        .header("Content-Type", "application/json")
        .send()
        .await?
        .error_for_status()?;
    let byte = get_body_max_size(resp.bytes_stream(), max_size_mb * 1024 * 1024).await?;
    Ok(ResolvedInput {
        index,
        data: byte.to_vec(),
        input_type: ProgramInputType::Private,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::Mock;
    use reqwest::{Client, Url};

    use std::sync::Arc;

    // Modified to return the server along with the mock and URL
    pub async fn get_server(url_path: &str, response: &[u8]) -> (Mock, Url, mockito::ServerGuard) {
        let mut server = mockito::Server::new_async().await;
        let url = Url::parse(&format!("{}{}", server.url(), url_path)).unwrap();

        let mock = server
            .mock("GET", url_path) // Changed to POST to match your function
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(response)
            .create_async()
            .await;

        (mock, url, server)
    }

    #[tokio::test]
    async fn test_download_public_input_success() {
        // 1 MB max size
        let max_size_mb = 1;
        // 10 KB response
        let input_data = vec![1u8; 1024 * 10];

        let (mock, url, _server) = get_server("/download", &input_data).await;
        let client = Arc::new(Client::new());

        let valid_result = download_public_input(
            client.clone(),
            1u8,
            url,
            max_size_mb,
            ProgramInputType::Public,
            Duration::from_secs(30),
        )
        .await;

        assert!(valid_result.is_ok());
        let resolved_input = valid_result.unwrap();
        assert_eq!(resolved_input.index, 1);
        assert_eq!(resolved_input.data, input_data);
        assert!(matches!(
            resolved_input.input_type,
            ProgramInputType::Public
        ));

        mock.assert();
    }

    #[tokio::test]
    async fn test_download_public_input_oversized() {
        // 1 MB max size
        let max_size_mb = 1;
        // 2 MB response
        let input_data = vec![1u8; 1024 * 1024 * 2];

        let (mock, url, _server) = get_server("/download", &input_data).await;
        let client = Arc::new(Client::new());

        let valid_result = download_public_input(
            client.clone(),
            1u8,
            url,
            max_size_mb,
            ProgramInputType::Public,
            Duration::from_secs(30),
        )
        .await;

        assert!(valid_result.is_err());
        assert_eq!(valid_result.unwrap_err().to_string(), "Max size exceeded");

        mock.assert();
    }
}
