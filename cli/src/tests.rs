use std::path::Path;

use assert_cmd::Command;

mod estimate;

pub(crate) fn bonsol_cmd() -> Command {
    let mut cmd = Command::cargo_bin("bonsol").unwrap();
    // the test directory must be the project root
    cmd.current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap());
    cmd
}

pub(crate) fn bonsol_build() -> Command {
    let mut cmd = bonsol_cmd();
    let keypair = cmd
        .get_current_dir()
        .unwrap()
        .join("cli")
        .join("src")
        .join("tests")
        .join("test_data")
        .join("test_id.json");
    cmd.args(&[
        "--keypair",
        keypair.to_str().unwrap(),
        "--rpc-url",
        "http://localhost:8899",
    ])
    .arg("build");
    cmd
}
