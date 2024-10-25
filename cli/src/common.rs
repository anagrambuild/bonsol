use anyhow::Result;
use bonsol_prover::input_resolver::{ProgramInput, ResolvedInput};
use bonsol_sdk::instructions::{CallbackConfig, ExecutionConfig};
use bonsol_sdk::{InputT, InputType, ProgramInputType};
use clap::Args;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use solana_rpc_client::nonblocking::rpc_client;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use std::fs::File;
use std::process::Command;
use std::str::FromStr;

pub fn cargo_has_plugin(plugin_name: &str) -> bool {
    Command::new("cargo")
        .args(["--list"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.trim().starts_with(plugin_name))
        })
        .unwrap_or(false)
}

pub fn has_executable(executable: &str) -> bool {
    Command::new("which")
        .arg(executable)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkProgramManifest {
    pub name: String,
    pub binary_path: String,
    pub image_id: String,
    pub input_order: Vec<String>,
    pub signature: String,
    pub size: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Args)]
#[serde(rename_all = "camelCase")]
pub struct CliInput {
    pub input_type: String,
    pub data: String, // base64 encoded if binary
}

#[derive(Debug, Clone)]
pub struct CliInputType(InputType);
impl ToString for CliInputType {
    fn to_string(&self) -> String {
        match self.0 {
            InputType::PublicData => "PublicData".to_string(),
            InputType::PublicAccountData => "PublicAccountData".to_string(),
            InputType::PublicUrl => "PublicUrl".to_string(),
            InputType::Private => "Private".to_string(),
            InputType::InputSet => "InputSet".to_string(),
            InputType::PublicProof => "PublicProof".to_string(),
            InputType::PrivateLocal => "PrivateUrl".to_string(),
            _ => "InvalidInputType".to_string(),
        }
    }
}

impl FromStr for CliInputType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PublicData" => Ok(CliInputType(InputType::PublicData)),
            "PublicAccountData" => Ok(CliInputType(InputType::PublicAccountData)),
            "PublicUrl" => Ok(CliInputType(InputType::PublicUrl)),
            "Private" => Ok(CliInputType(InputType::Private)),
            "InputSet" => Ok(CliInputType(InputType::InputSet)),
            "PublicProof" => Ok(CliInputType(InputType::PublicProof)),
            "PrivateUrl" => Ok(CliInputType(InputType::PrivateLocal)),
            _ => Err(anyhow::anyhow!("Invalid input type")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequestFile {
    pub image_id: Option<String>,
    pub execution_config: CliExecutionConfig,
    pub execution_id: Option<String>,
    pub tip: Option<u64>,
    pub expiry: Option<u64>,
    pub inputs: Option<Vec<CliInput>>,
    pub callback_config: Option<CliCallbackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliExecutionConfig {
    pub verify_input_hash: Option<bool>,
    pub input_hash: Option<String>,
    pub forward_output: Option<bool>,
}

impl From<CliExecutionConfig> for ExecutionConfig {
    fn from(val: CliExecutionConfig) -> Self {
        ExecutionConfig {
            verify_input_hash: val.verify_input_hash.unwrap_or(true),
            input_hash: val.input_hash.map(|v| bs58::decode(v).into_vec().unwrap()),
            forward_output: val.forward_output.unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CliCallbackConfig {
    #[serde(with = "bonsol_sdk::instructions::serde_helpers::optpubkey")]
    pub program_id: Option<Pubkey>,
    pub instruction_prefix: Option<Vec<u8>>,
    pub extra_accounts: Option<Vec<CliAccountMeta>>,
}

impl From<CliCallbackConfig> for CallbackConfig {
    fn from(val: CliCallbackConfig) -> Self {
        CallbackConfig {
            program_id: val.program_id.unwrap_or_default(),
            instruction_prefix: val.instruction_prefix.unwrap_or_default(),
            extra_accounts: val
                .extra_accounts
                .map(|v| v.into_iter().map(|a| a.into()).collect())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CliAccountMeta {
    #[serde(default, with = "bonsol_sdk::instructions::serde_helpers::pubkey")]
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<CliAccountMeta> for AccountMeta {
    fn from(val: CliAccountMeta) -> Self {
        AccountMeta {
            pubkey: val.pubkey,
            is_signer: val.is_signer,
            is_writable: val.is_writable,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFile {
    pub inputs: Vec<CliInput>,
}

pub async fn sol_check(rpc_client: String, pubkey: Pubkey) -> bool {
    let rpc_client = rpc_client::RpcClient::new(rpc_client);
    if let Ok(account) = rpc_client.get_account(&pubkey).await {
        return account.lamports > 0;
    }
    false
}

pub fn execute_get_inputs(
    inputs_file: Option<String>,
    stdin: Option<String>,
) -> Result<Vec<CliInput>> {
    if let Some(std) = stdin {
        let parsed = serde_json::from_str::<InputFile>(&std)
            .map_err(|e| anyhow::anyhow!("Error parsing stdin: {:?}", e))?;
        return Ok(parsed.inputs);
    }

    if let Some(istr) = inputs_file {
        let ifile = File::open(istr)?;
        let parsed: InputFile = serde_json::from_reader(&ifile)
            .map_err(|e| anyhow::anyhow!("Error parsing inputs file: {:?}", e))?;
        return Ok(parsed.inputs);
    }

    Err(anyhow::anyhow!("No inputs provided"))
}

pub fn proof_get_inputs(
    inputs_file: Option<String>,
    stdin: Option<String>,
) -> Result<Vec<ProgramInput>> {
    if let Some(std) = stdin {
        return proof_parse_stdin(&std);
    }
    if let Some(istr) = inputs_file {
        return proof_parse_input_file(&istr);
    }
    Err(anyhow::anyhow!("No inputs provided"))
}

pub fn execute_transform_cli_inputs(inputs: Vec<CliInput>) -> Result<Vec<InputT>> {
    let mut res = vec![];
    for input in inputs.into_iter() {
        let input_type = CliInputType::from_str(&input.input_type)?.0;
        match input_type {
            InputType::PublicData => {
                if is_valid_base64(&input.data) {
                    let data = general_purpose::STANDARD.decode(&input.data)?;
                    res.push(InputT::public(data));
                }
                if let Some(n) = is_valid_number(&input.data) {
                    let data = n.into_bytes();
                    res.push(InputT::public(data));
                }

                res.push(InputT::public(input.data.into_bytes()));
            }
            _ => res.push(InputT::new(input_type, Some(input.data.into_bytes()))),
        }
    }
    Ok(res)
}

fn is_valid_base64(s: &str) -> bool {
    if s.len() % 4 != 0 {
        return false;
    }
    let is_base64_char = |c: char| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=';
    if !s.chars().all(is_base64_char) {
        return false;
    }
    let padding_count = s.chars().rev().take_while(|&c| c == '=').count();
    if padding_count > 2 {
        return false;
    }
    general_purpose::STANDARD.decode(s).is_ok()
}

pub enum NumberType {
    Float(f64),
    Unsigned(u64),
    Integer(i64),
    // TODO: add BigInt
}

impl NumberType {
    fn into_bytes(&self) -> Vec<u8> {
        match self {
            NumberType::Float(f) => f.to_le_bytes().to_vec(),
            NumberType::Unsigned(u) => u.to_le_bytes().to_vec(),
            NumberType::Integer(i) => i.to_le_bytes().to_vec(),
        }
    }
}

fn is_valid_number(s: &str) -> Option<NumberType> {
    if let Ok(num) = s.parse::<f64>() {
        return Some(NumberType::Float(num));
    }
    if let Ok(num) = s.parse::<u64>() {
        return Some(NumberType::Unsigned(num));
    }
    if let Ok(num) = s.parse::<i64>() {
        return Some(NumberType::Integer(num));
    }
    None
}

fn parse_entry(index: u8, s: &str) -> Result<ProgramInput> {
    if let Ok(num) = s.parse::<f64>() {
        return Ok(ProgramInput::Resolved(ResolvedInput {
            index,
            data: num.to_le_bytes().to_vec(),
            input_type: ProgramInputType::Private,
        }));
    }
    if let Ok(num) = s.parse::<u64>() {
        return Ok(ProgramInput::Resolved(ResolvedInput {
            index,
            data: num.to_le_bytes().to_vec(),
            input_type: ProgramInputType::Private,
        }));
    }
    if let Ok(num) = s.parse::<i64>() {
        return Ok(ProgramInput::Resolved(ResolvedInput {
            index,
            data: num.to_le_bytes().to_vec(),
            input_type: ProgramInputType::Private,
        }));
    }
    if is_valid_base64(s) {
        let decoded = general_purpose::STANDARD
            .decode(s)
            .map_err(|e| anyhow::anyhow!("Error decoding base64 input: {:?}", e))?;
        return Ok(ProgramInput::Resolved(ResolvedInput {
            index,
            data: decoded,
            input_type: ProgramInputType::Private,
        }));
    }

    return Ok(ProgramInput::Resolved(ResolvedInput {
        index,
        data: s.as_bytes().to_vec(),
        input_type: ProgramInputType::Private,
    }));
}

fn proof_parse_input_file(input_file: &str) -> Result<Vec<ProgramInput>> {
    if let Ok(ifile) = serde_json::from_str::<InputFile>(input_file) {
        let len = ifile.inputs.len();
        let parsed: Vec<ProgramInput> = ifile
            .inputs
            .into_iter()
            .enumerate()
            .flat_map(|(index, input)| parse_entry(index as u8, &input.data).ok())
            .collect();
        if parsed.len() != len {
            return Err(anyhow::anyhow!("Invalid input file"));
        }
        return Ok(parsed);
    }
    Err(anyhow::anyhow!("Invalid input file"))
}

fn proof_parse_stdin(input: &str) -> Result<Vec<ProgramInput>> {
    let mut entries = Vec::new();
    let mut current_entry = String::new();
    let mut in_quotes = false;
    let mut in_brackets = 0;
    for c in input.chars() {
        match c {
            '"' if !in_quotes => in_quotes = true,
            '"' if in_quotes => in_quotes = false,
            '{' | '[' if !in_quotes => in_brackets += 1,
            '}' | ']' if !in_quotes => in_brackets -= 1,
            ' ' if !in_quotes && in_brackets == 0 && !current_entry.is_empty() => {
                let index = entries.len() as u8;
                println!("{}", current_entry);
                entries.push(parse_entry(index, &current_entry)?);
                current_entry.clear();
                continue;
            }
            _ => {}
        }
        current_entry.push(c);
    }
    if !current_entry.is_empty() {
        entries.push(parse_entry(entries.len() as u8, &current_entry)?);
    }
    Ok(entries)
}

pub fn rand_id(chars: usize) -> String {
    let mut rng = rand::thread_rng();
    (&mut rng)
        .sample_iter(Alphanumeric)
        .take(chars)
        .map(char::from)
        .collect()
}
