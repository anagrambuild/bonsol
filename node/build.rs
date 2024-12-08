//! Automatically detect and update toolchain component changes.

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use toml_edit::{value, DocumentMut};

// NOTE: risc0 is currently on a git branch and should be added to this asap
const TOOLCHAIN_DEPS: &[&str; 2] = &["flatc", "solana" /*, "risc0" */];
const WORKSPACE_DEPS: &[&str; 2] = &["flatbuffers", "solana-sdk" /*, "risc0-core" */];
/// Prefix and suffix syntax for version requirements to strip from the version before replacement
const CARGO_VERSION_REQUIREMENTS: &[char; 6] = &['^', '=', '*', '~', '>', '<'];

fn main() {
    println!("cargo:rerun-if-changed=../Cargo.toml");
    println!("cargo:rerun-if-changed=../rust-toolchain.toml");

    let toolchain_path = Path::new("../rust-toolchain.toml");
    let cargo_path = Path::new("../Cargo.toml");

    let toolchain_contents =
        fs::read_to_string(toolchain_path).expect("Failed to read rust-toolchain.toml");
    let mut toolchain = toolchain_contents
        .parse::<DocumentMut>()
        .expect("Failed to parse rust-toolchain.toml");

    let manifest_contents = fs::read_to_string(cargo_path).expect("Failed to read Cargo.toml");
    let manifest: toml::Value =
        toml::from_str(&manifest_contents).expect("Failed to parse Cargo.toml");

    let mut system_deps = toolchain
        .get_mut("toolchain")
        .and_then(|tc| tc.get_mut("system-dependencies"));
    let workspace_deps = manifest
        .get("workspace")
        .and_then(|ws| ws.get("dependencies"));

    // Iterate over each dependency in TOOLCHAIN_DEPS
    for (tc_dep, ws_dep) in TOOLCHAIN_DEPS.iter().zip(WORKSPACE_DEPS.iter()) {
        if let (Some(tc_ver), Some(ws_ver)) = (
            system_deps
                .as_mut()
                .and_then(|deps| deps.get(tc_dep))
                .and_then(|dep| dep.get("version"))
                .and_then(|version| version.as_str()),
            workspace_deps
                .and_then(|deps| deps.get(ws_dep))
                .and_then(|dep| dep.get("version"))
                .and_then(|version| version.as_str()),
        ) {
            if tc_ver != ws_ver {
                // Replace the version in rust-toolchain.toml with the one from Cargo.toml
                if let Some(dependency) = system_deps.as_mut().and_then(|deps| deps.get_mut(tc_dep))
                {
                    dependency["version"] =
                        value(ws_ver.to_string().replace(CARGO_VERSION_REQUIREMENTS, ""));
                }
            }
        }
    }

    // Write the updated toolchain file back
    let updated_toolchain_contents = toolchain.to_string();
    let mut file =
        File::create(toolchain_path).expect("Failed to open rust-toolchain.toml for writing");
    file.write_all(updated_toolchain_contents.as_bytes())
        .expect("Failed to write updated rust-toolchain.toml");
}
