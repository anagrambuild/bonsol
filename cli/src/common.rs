use std::process::Command;

use bonsol_sdk::{input_resolver::{ProgramInput, ResolvedInput}, instructions::{CallbackConfig, ExecutionConfig}, InputT, InputType, ProgramInputType};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize, Serializer};
use std::fs::File;
use anyhow::Result;

pub fn cargo_has_plugin(plugin_name: &str) -> bool {
    Command::new("cargo")
        .args(&["--list"])
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
pub struct CliInput {
    pub input_type: String,
    pub data: String, // base64 encoded if binary
}

#[derive(Debug, Clone)]
pub struct CliInputType(InputType);

impl Serialize for CliInputType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            InputType::PublicData => serializer.serialize_str("Public"),
            InputType::PublicAccountData => serializer.serialize_str("PublicAccountData"),
            InputType::PublicUrl => serializer.serialize_str("PublicUrl"),
            InputType::Private => serializer.serialize_str("Private"),
            InputType::InputSet => serializer.serialize_str("InputSet"),
            InputType::PublicProof => serializer.serialize_str("PublicProof"),
            InputType::PrivateLocal => serializer.serialize_str("PrivateLocal"),
            _ => Err(serde::ser::Error::custom("Invalid input type")),
        }
    }
}

impl<'de> Deserialize<'de> for CliInputType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Public" => Ok(CliInputType(InputType::PublicData)),
            "PublicAccountData" => Ok(CliInputType(InputType::PublicAccountData)),
            "PublicUrl" => Ok(CliInputType(InputType::PublicUrl)),
            "Private" => Ok(CliInputType(InputType::Private)),
            "InputSet" => Ok(CliInputType(InputType::InputSet)),
            "PublicProof" => Ok(CliInputType(InputType::PublicProof)),
            "PrivateLocal" => Ok(CliInputType(InputType::PrivateLocal)),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid input type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequestFile {
    pub image_id: Option<String>,
    pub execution_config: ExecutionConfig,
    pub execution_id: Option<String>,
    pub tip: Option<u64>,
    pub max_block_height: Option<u64>,
    pub inputs: Option<Vec<CliInput>>,
    pub callback_config: Option<CallbackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFile {
    pub inputs: Vec<CliInput>,
}

pub fn get_inputs(inputs_file: Option<String>, stdin: Option<String>) -> Result<Vec<CliInput>> {
    let inputs = if let Some(istr) = inputs_file {
        let ifile = File::open(istr)?;
        let input_file: InputFile = serde_json::from_reader(&ifile)?;
        input_file.inputs
    } else if let Some(stdin) = stdin {
        let input_file: InputFile = serde_json::from_str(&stdin)?;
        input_file.inputs
    } else {
        return Err(anyhow::anyhow!("No inputs provided"));
    };
    Ok(inputs)
}

pub fn transform_inputs(inputs: Vec<CliInput>) -> Result<Vec<InputT>> {
    let mut res = vec![];
    for input in inputs.into_iter() {
        let input_type = serde_json::from_str::<CliInputType>(&input.input_type)?.0;
        match input_type {
            InputType::PublicData => {
                let data = base64::decode(&input.data)?;
                res.push(InputT::public(data));
            }
            _ => {
                res.push(InputT::new(input_type, Some(input.data.into_bytes())))
            }
        }
    }
    Ok(res)
}

//
pub fn resolve_inputs_for_local_proving(
    inputs: Vec<CliInput>,
) -> Result<Vec<ProgramInput>, anyhow::Error> {
    let mut res = vec![];
    for (index, input) in inputs.into_iter().enumerate() {
        let input_type = serde_json::from_str::<CliInputType>(&input.input_type)?.0;

        match input_type {
            InputType::PrivateLocal => {
                let data = base64::decode(&input.data)?;
                res.push(ProgramInput::Resolved(ResolvedInput{
                    index: index as u8,
                    data: data,
                    input_type: ProgramInputType::Private,
                }));
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid input type, local proving only supports private inputs"));
            }
        }
    }
    Ok(res)
}