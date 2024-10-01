use anyhow::Result;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

pub fn init_project(project_name: &str, dir: Option<String>) -> Result<()> {
    let pwd = std::env::current_dir()?;
    let project_path = if let Some(dir) = dir {
        let dir = Path::new(&dir);
        if dir.is_relative() {
            pwd.join(dir).to_path_buf()
        } else {
            dir.to_path_buf()
        }
    } else {
        pwd.join(project_name).to_path_buf()
    };
    if project_path.exists() {
        return Err(anyhow::anyhow!("Project already exists"));
    }

    // Create src directory
    fs::create_dir_all(project_path.join("src"))?;

    // Initialize Tera
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("Cargo.toml", CARGO_TEMPLATE),
        ("src/main.rs", MAIN_TEMPLATE),
        ("README.md", README_TEMPLATE),
    ])?;

    // Create context
    let mut context = Context::new();
    context.insert("project_name", project_name);

    // Render and write templates
    for (template_name, file_name) in &[
        ("Cargo.toml", "Cargo.toml"),
        ("src/main.rs", "src/main.rs"),
        ("README.md", "README.md"),
    ] {
        let content = tera.render(template_name, &context)?;
        fs::write(format!("{}/{}", project_name, file_name), content)?;
    }

    println!("Project '{}' initialized successfully!", project_name);
    Ok(())
}

const CARGO_TEMPLATE: &str = r#"
[package]
name = "{{ project_name }}"
version = "0.1.0"
edition = "2021"

[package.metadata.zkprogram]
input_order = ["Public"]


[workspace]

[dependencies]
risc0-zkvm = {git = "https://github.com/anagrambuild/risc0", branch = "v1.0.1-bonsai-fix", default-features = false, features = ["std"]}

[dependencies.sha2]
git = "https://github.com/risc0/RustCrypto-hashes"
tag = "sha2-v0.10.6-risczero.0"
"#;

const MAIN_TEMPLATE: &str = r#"
use risc0_zkvm::{guest::{env, sha::Impl},sha::{Digest, Sha256}};

fn main() {
    let mut input_1 = Vec::new();
    env::read_slice(&mut input_1);
    let digest = Impl::hash_bytes(&[input_1.as_slice()]);
    env::commit_slice(digest.as_bytes());
}
"#;

const README_TEMPLATE: &str = r#"
# {{ project_name }}

This is a Bonsol zkprogram, built on risc0
"#;
