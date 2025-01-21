use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use anyhow::{Context, Result};
use bonsol_prover::input_resolver::{ProgramInput, ResolvedInput};
use bonsol_sdk::instructions::CallbackConfig;
use bonsol_sdk::{InputT, InputType, ProgramInputType};
use clap::Args;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use solana_rpc_client::nonblocking::rpc_client;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use crate::error::{BonsolCliError, ParseConfigError};

pub(crate) const MANIFEST_JSON: &str = "manifest.json";
pub(crate) const CARGO_COMMAND: &str = "cargo";
pub(crate) const CARGO_TOML: &str = "Cargo.toml";
pub(crate) const TARGET_DIR: &str = "target";

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
    pub data: String, // hex encoded if binary with hex: prefix
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

/// Attempt to load the RPC URL and keypair file from a solana `config.yaml`.
pub(crate) fn try_load_from_config(config: Option<String>) -> anyhow::Result<(String, String)> {
    let whoami = String::from_utf8_lossy(&std::process::Command::new("whoami").output()?.stdout)
        .trim_end()
        .to_string();
    let default_config_path = solana_cli_config::CONFIG_FILE.as_ref();

    let config_file = config.as_ref().map_or_else(
        || -> anyhow::Result<&String> {
            let inner_err = ParseConfigError::DefaultConfigNotFound {
                whoami: whoami.clone(),
            };
            let context = inner_err.context(None);

            // If no config is given, try to find it at the default location.
            default_config_path
                .and_then(|s| PathBuf::from_str(s).is_ok_and(|p| p.exists()).then_some(s))
                .ok_or(BonsolCliError::ParseConfigError(inner_err))
                .context(context)
        },
        |config| -> anyhow::Result<&String> {
            // Here we throw an error if the user provided a path to a config that does not exist.
            // Instead of using the default location, it's better to show the user the path they
            // expected to use was not valid.
            if !PathBuf::from_str(config)?.exists() {
                let inner_err = ParseConfigError::ConfigNotFound {
                    path: config.into(),
                };
                let context = inner_err.context(None);
                let err: anyhow::Error = BonsolCliError::ParseConfigError(inner_err).into();
                return Err(err.context(context));
            }
            Ok(config)
        },
    )?;
    let config = {
        let mut inner_err = ParseConfigError::Uninitialized;

        let mut maybe_config = solana_cli_config::Config::load(config_file).map_err(|err| {
            let err = ParseConfigError::FailedToLoad {
                path: config.unwrap_or(default_config_path.cloned().unwrap()),
                err: format!("{err:?}"),
            };
            inner_err = err.clone();
            BonsolCliError::ParseConfigError(err).into()
        });
        if maybe_config.is_err() {
            maybe_config = maybe_config.context(inner_err.context(Some(whoami)));
        }
        maybe_config
    }?;
    Ok((config.json_rpc_url, config.keypair_path))
}

pub(crate) fn load_solana_config(
    config: Option<String>,
    rpc_url: Option<String>,
    keypair: Option<String>,
) -> anyhow::Result<(String, solana_sdk::signer::keypair::Keypair)> {
    let (rpc_url, keypair_file) = match rpc_url.zip(keypair) {
        Some(config) => config,
        None => try_load_from_config(config)?,
    };
    Ok((
        rpc_url,
        solana_sdk::signature::read_keypair_file(std::path::Path::new(&keypair_file)).map_err(
            |err| BonsolCliError::FailedToReadKeypair {
                file: keypair_file,
                err: format!("{err:?}"),
            },
        )?,
    ))
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
                let has_hex_prefix = input.data.starts_with("0x");
                if has_hex_prefix {
                    let (is_valid, data) = is_valid_hex(&input.data[2..]);
                    if is_valid {
                        res.push(InputT::public(data));
                    }
                    continue;
                }
                if let Some(n) = is_valid_number(&input.data) {
                    let data = n.into_bytes();
                    res.push(InputT::public(data));
                    continue;
                }
                res.push(InputT::public(input.data.into_bytes()));
            }
            _ => res.push(InputT::new(input_type, Some(input.data.into_bytes()))),
        }
    }
    Ok(res)
}

fn is_valid_hex(s: &str) -> (bool, Vec<u8>) {
    if s.len() % 4 != 0 {
        return (false, vec![]);
    }
    let is_hex_char = |c: char| c.is_ascii_hexdigit();
    if !s.chars().all(is_hex_char) {
        return (false, vec![]);
    }
    let out = hex::decode(s);
    (out.is_ok(), out.unwrap_or_default())
}

#[derive(Debug, PartialEq)]
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
    if let Ok(num) = s.parse::<u64>() {
        return Some(NumberType::Unsigned(num));
    }
    if let Ok(num) = s.parse::<i64>() {
        return Some(NumberType::Integer(num));
    }
    if let Ok(num) = s.parse::<f64>() {
        return Some(NumberType::Float(num));
    }
    None
}

fn proof_parse_entry(index: u8, s: &str) -> Result<ProgramInput> {
    if let Ok(num) = s.parse::<i64>() {
        return Ok(ProgramInput::Resolved(ResolvedInput {
            index,
            data: num.to_le_bytes().to_vec(),
            input_type: ProgramInputType::Private,
        }));
    }
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
    let has_hex_prefix = s.starts_with("0x");
    if has_hex_prefix {
        let (is_valid, data) = is_valid_hex(&s[2..]);
        if is_valid {
            return Ok(ProgramInput::Resolved(ResolvedInput {
                index,
                data,
                input_type: ProgramInputType::Private,
            }));
        } else {
            return Err(anyhow::anyhow!("Invalid hex data"));
        }
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
            .flat_map(|(index, input)| proof_parse_entry(index as u8, &input.data).ok())
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
                entries.push(proof_parse_entry(index, &current_entry)?);
                current_entry.clear();
                continue;
            }
            _ => {}
        }
        current_entry.push(c);
    }
    if !current_entry.is_empty() {
        entries.push(proof_parse_entry(entries.len() as u8, &current_entry)?);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_proof_parse_stdin() {
        let inputs = r#"1234567890abcdef 0x313233343536373839313061626364656667 2.1 2000 -2000 {"attestation":"test"}"#;
        let inputs_parsed = proof_parse_stdin(inputs).unwrap();

        let expected_inputs = vec![
            ProgramInput::Resolved(ResolvedInput {
                index: 0,
                data: "1234567890abcdef".as_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
            ProgramInput::Resolved(ResolvedInput {
                index: 1,
                data: "12345678910abcdefg".as_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
            ProgramInput::Resolved(ResolvedInput {
                index: 2,
                data: 2.1f64.to_le_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
            ProgramInput::Resolved(ResolvedInput {
                index: 3,
                data: 2000u64.to_le_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
            ProgramInput::Resolved(ResolvedInput {
                index: 4,
                data: (-2000i64).to_le_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
            ProgramInput::Resolved(ResolvedInput {
                index: 5,
                data: "{\"attestation\":\"test\"}".as_bytes().to_vec(),
                input_type: ProgramInputType::Private,
            }),
        ];
        assert_eq!(inputs_parsed, expected_inputs);
    }

    #[test]
    fn test_is_valid_number() {
        let num = is_valid_number("1234567890abcdef");
        assert!(num.is_none());
        let num = is_valid_number("1234567890abcdefg");
        assert!(num.is_none());
        let num = is_valid_number("2.1");
        assert!(num.is_some());
        assert_eq!(num.unwrap(), NumberType::Float(2.1));
        let num = is_valid_number("2000");
        assert!(num.is_some());
        assert_eq!(num.unwrap(), NumberType::Unsigned(2000));
        let num = is_valid_number("-2000");
        assert!(num.is_some());
        assert_eq!(num.unwrap(), NumberType::Integer(-2000));
    }

    #[test]
    fn test_execute_transform_cli_inputs() {
        let input = CliInput {
            input_type: "PublicData".to_string(),
            data: "1234567890abcdef".to_string(),
        };
        let hex_input = CliInput {
            input_type: "PublicData".to_string(),
            data: "0x313233343536373839313061626364656667".to_string(),
        };
        let hex_input2 = CliInput {
            input_type: "PublicData".to_string(),
            data: "2.1".to_string(),
        };
        let hex_input3 = CliInput {
            input_type: "PublicData".to_string(),
            data: "2000".to_string(),
        };
        let hex_input4 = CliInput {
            input_type: "PublicData".to_string(),
            data: "-2000".to_string(),
        };
        let inputs = vec![input, hex_input, hex_input2, hex_input3, hex_input4];
        let parsed_inputs = execute_transform_cli_inputs(inputs).unwrap();
        assert_eq!(
            parsed_inputs,
            vec![
                InputT::public("1234567890abcdef".as_bytes().to_vec()),
                InputT::public("12345678910abcdefg".as_bytes().to_vec()),
                InputT::public(2.1f64.to_le_bytes().to_vec()),
                InputT::public(2000u64.to_le_bytes().to_vec()),
                InputT::public((-2000i64).to_le_bytes().to_vec()),
            ]
        );
    }
}
