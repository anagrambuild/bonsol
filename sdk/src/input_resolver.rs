use std::sync::Arc;

use anagram_bonsol_schema::{Input, InputType, ProgramInputType};
use reqwest::Url;
use async_trait::async_trait;

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
    fn index(&self) -> u8 {
        match self {
            ProgramInput::Resolved(ri) => ri.index,
            ProgramInput::Unresolved(ui) => ui.index,
            _ => 0,
        }
    }
}


#[async_trait]
pub trait InputResolver {
  fn supports(&self, input_type: InputType) -> bool;
  async fn resolve_public_inputs(&self,
    execution_id: &str,
    inputs: Vec<Input>) -> Result<Vec<ProgramInput>, anyhow::Error>;
  async fn resolve_private_inputs(&self,
    execution_id: &str,
    inputs: Vec<Input>) -> Result<Vec<ProgramInput>, anyhow::Error>;
}

pub struct DeafultInputResolver {
  input_set_max_depth: Option<u8>,
  http_client: Arc<reqwest::Client>,
  solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
}

impl DeafultInputResolver {
  
  pub fn new(
    http_client: Arc<reqwest::Client>, 
    solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>) -> Self {
    DeafultInputResolver {
      input_set_max_depth: None,
      http_client,
      solana_rpc_client,
    }
  }

  pub fn new_with_opts(
    http_client: Arc<reqwest::Client>,
    solana_rpc_client: Arc<solana_rpc_client::nonblocking::rpc_client::RpcClient>,
    input_set_max_depth: Option<u8>,
  ) -> Self {
    DeafultInputResolver {
      input_set_max_depth,
      http_client,
      solana_rpc_client,
    }
  } 
}

#[async_trait]
impl InputResolver for DeafultInputResolver {
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

  async fn resolve_public_inputs(&self, execution_id: &str, inputs: Vec<Input>) -> Result<Vec<ProgramInput>, anyhow::Error> {
    todo!()
  }

  async fn resolve_private_inputs(&self, execution_id: &str, inputs: Vec<Input>) -> Result<Vec<ProgramInput>, anyhow::Error> {
    todo!()
  }
}
