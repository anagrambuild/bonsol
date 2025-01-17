use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Define schema directory and target directory for generated Rust code.
    let schema_dir = Path::new("../schemas");
    let generated_src =
        PathBuf::from(env::var("GENERATED_CODE_DIR").unwrap_or_else(|_| "src".to_string()));

    // Collect all .fbs files in the schema directory.
    let file_list: Vec<_> = fs::read_dir(schema_dir)
        .expect("Schema directory not found")
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("fbs") {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    // Build flatc arguments.
    let mut args = vec![
        "--gen-mutable",
        "--gen-object-api",
        "--reflect-names",
        "--rust",
        "-o",
        generated_src
            .to_str()
            .expect("Invalid path for generated source"),
    ];

    // Add schema file paths to the arguments.
    args.extend(file_list.iter().map(|path| path.to_str().unwrap()));

    // Execute flatc.
    let compile_status = Command::new("flatc")
        .args(&args)
        .status()
        .expect("Failed to execute flatc command");

    assert!(
        compile_status.success(),
        "flatc failed to compile schema files: {:?}",
        file_list
    );

    // Set an environment variable with the generated source path, stripping "src/" if present.
    let generated_path = generated_src.strip_prefix("src").unwrap_or(&generated_src);
    println!("cargo:rustc-env=GENERATED_SRC={}", generated_path.display());

    // Instruct Cargo to re-run this script if schema files change.
    println!("cargo:rerun-if-changed={}", schema_dir.display());
}
