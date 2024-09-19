use std::{
    str::from_utf8,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use bonsol_schema::{root_as_input_set, Input, InputT, InputType, ProgramInputType};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use tokio::task::{JoinHandle, JoinSet};

use crate::util::get_body_max_size;

#[derive(Debug, Clone)]
pub enum ProgramInput {
    Empty,
    Resolved(ResolvedInput),
    Unresolved(UnresolvedInput),
}

#[derive(Debug, Clone)]
pub struct UnresolvedInput {
    pub index: u8,
    pub url: Url,
    pub input_type: ProgramInputType,
}

#[derive(Debug, Clone)]
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

// naive resolver that downloads inputs and resolves inputsets just in time
pub struct DefaultInputResolver {
    http_client: Arc<reqwest::Client>,
    solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
    max_input_size_mb: u64,
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
        }
    }

    pub fn new_with_opts(
        http_client: Arc<reqwest::Client>,
        solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
        max_input_size_mb: Option<u64>,
    ) -> Self {
        DefaultInputResolver {
            http_client,
            solana_rpc_client,
            max_input_size_mb: max_input_size_mb.unwrap_or(10),
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
                let url = input
                    .data
                    .ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                task_set.spawn(dowload_public_input(
                    client,
                    index as u8,
                    url.clone(),
                    self.max_input_size_mb.clone() as usize,
                    ProgramInputType::Public,
                ));
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index: index as u8,
                    url,
                    input_type: ProgramInputType::Public,
                }))
            }
            InputType::Private => {
                let url = input
                    .data
                    .ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index: index as u8,
                    url,
                    input_type: ProgramInputType::Private,
                }))
            }
            InputType::PublicData => {
                let data = input
                    .data
                    .ok_or(anyhow::anyhow!("Invalid data"))?;
                let data = data.to_vec();
                Ok(ProgramInput::Resolved(ResolvedInput {
                    index: index as u8,
                    data,
                    input_type: ProgramInputType::Public,
                }))
            }
            InputType::PublicProof => {
                let url = input
                    .data
                    .ok_or(anyhow::anyhow!("Invalid data"))?;
                let url = from_utf8(&url)?;
                let url = Url::parse(url)?;
                task_set.spawn(dowload_public_input(
                    client,
                    index as u8,
                    url.clone(),
                    self.max_input_size_mb.clone() as usize,
                    ProgramInputType::PublicProof,
                ));
                Ok(ProgramInput::Unresolved(UnresolvedInput {
                    index: index as u8,
                    url,
                    input_type: ProgramInputType::PublicProof,
                }))
            }
            _ => {
                // not implemented yet / or unknown
                Err(anyhow::anyhow!("Invalid input type"))
            }
        }
    }

    async fn par_resolve_input_set<'a>(
        &self,
        input_set_account: Pubkey,
        client: Arc<reqwest::Client>,
        index: u8,
        mut task_set: &mut JoinSet<Result<ResolvedInput>>,
    ) -> Result<Vec<ProgramInput>> {
        let data = self
            .solana_rpc_client
            .get_account_data(&input_set_account)
            .await?;
        let input_set =
            root_as_input_set(&*data).map_err(|_| anyhow::anyhow!("Invalid Input set data"))?;
        if input_set.inputs().is_none() {
            return Err(anyhow::anyhow!("Invalid Input set data"));
        }
        let inputs = input_set.inputs().unwrap();
        let mut res = Vec::with_capacity(inputs.len());
        for input in inputs.into_iter() {
            if input.input_type() == InputType::InputSet {
                return Err(anyhow::anyhow!("Input set nesting not supported"));
            }
            let input = input.unpack();
            res.push(self.par_resolve_input(client.clone(), index, input, &mut task_set)?);
        }
        Ok(res)
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
            InputType::InputSet => true,
            _ => false,
        }
    }

    async fn resolve_public_inputs(
        &self,
        inputs: Vec<InputT>,
    ) -> Result<Vec<ProgramInput>, anyhow::Error> {
        let mut url_set = JoinSet::new();
        let mut res = vec![ProgramInput::Empty; inputs.len()];
        let mut index_offset = 0;
        for (index, input) in inputs.into_iter().enumerate() {
            let index: u8 = index as u8 + index_offset;
            let client = self.http_client.clone();

            match input.input_type {
                InputType::InputSet => {
                    if let Some(input_set_account) =
                        input.data.as_ref().and_then(|i| Pubkey::try_from(i.as_slice()).ok())
                    {
                        let inputs = self
                            .par_resolve_input_set(input_set_account, client, index, &mut url_set)
                            .await?;
                        index_offset += inputs.len() as u8;
                        res.extend(inputs);
                    } else {
                        return Err(anyhow::anyhow!("Invalid Input set data"));
                    }
                }
                _ => {
                    res[index as usize] =
                        self.par_resolve_input(client, index, input, &mut url_set)?;
                }
            }
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
) -> Result<JoinHandle<Result<ResolvedInput>>> {
    let url = from_utf8(data)?;
    let url = Url::parse(url)?;
    Ok(tokio::task::spawn(dowload_public_input(
        client,
        index as u8,
        url,
        max_input_size_mb as usize,
        ProgramInputType::Public,
    )))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInputRequest {
    identity: Pubkey,
    claim_id: String,
    input_index: u8,
    now_utc: u64,
}

async fn dowload_public_input(
    client: Arc<reqwest::Client>,
    index: u8,
    url: Url,
    max_size: usize,
    input_type: ProgramInputType,
) -> Result<ResolvedInput> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let byte = get_body_max_size(resp.bytes_stream(), max_size).await?;
    Ok(ResolvedInput {
        index,
        data: byte.to_vec(),
        input_type,
    })
}

async fn download_private_input(
    client: Arc<reqwest::Client>,
    index: u8,
    url: Url,
    max_size: usize,
    body: String,
    claim_authorization: String,
) -> Result<ResolvedInput> {
    let resp = client
        .post(url)
        .body(body)
        // Signature of the json payload
        .header("Authorization", format!("Bearer {}", claim_authorization))
        .header("Content-Type", "application/json")
        .send()
        .await?
        .error_for_status()?;
    let byte = get_body_max_size(resp.bytes_stream(), max_size).await?;
    Ok(ResolvedInput {
        index,
        data: byte.to_vec(),
        input_type: ProgramInputType::Private,
    })
}
