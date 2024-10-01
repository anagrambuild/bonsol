use std::process::Command;

use bonsol_sdk::InputType;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFile {
    pub inputs: Vec<u8>,
}
