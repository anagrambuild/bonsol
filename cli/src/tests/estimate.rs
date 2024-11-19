use assert_cmd::Command;

use crate::tests::bonsol_cmd;

fn bonsol_estimate() -> Command {
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
    .arg("estimate");
    cmd
}

#[test]
fn estimate_simple() {
    let mut bonsol_estimate = bonsol_estimate();
    let image_path = bonsol_estimate
        .get_current_dir()
        .unwrap()
        .join("images")
        .join("simple")
        .join("manifest.json");
    bonsol_estimate.args(&["--manifest-path", image_path.to_str().unwrap()]);
    bonsol_estimate
        .assert()
        .success()
        .stdout(predicates::str::is_match(r##"number of cycles: "##).unwrap());
}
