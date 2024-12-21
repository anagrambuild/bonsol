use std::path::PathBuf;

use assert_cmd::Command;

use super::bonsol_build;
use crate::tests::bonsol_cmd;

fn bonsol_estimate() -> Command {
    let mut cmd = bonsol_cmd();
    cmd.arg("estimate");
    cmd
}

fn build_test_image(image_path: &PathBuf) {
    let mut cmd = bonsol_build();
    cmd.args(&[
        "-z",
        image_path
            .to_str()
            .expect("failed to convert image path to str"),
    ]);
    cmd.assert().success();
}

#[test]
fn estimate_simple() {
    let mut bonsol_estimate = bonsol_estimate();
    let image_path = bonsol_estimate
        .get_current_dir()
        .unwrap()
        .join("images")
        .join("simple");

    build_test_image(&image_path);
    let input_file = bonsol_estimate
        .get_current_dir()
        .unwrap()
        .join("testing-examples")
        .join("example-input-file.json");

    bonsol_estimate.args(&[
        "--manifest-path",
        image_path.join("manifest.json").to_str().unwrap(),
        "--input-file",
        input_file.to_str().unwrap(),
    ]);
    bonsol_estimate.assert().success().stdout(
        predicates::str::is_match(r##"User cycles: 3380\nTotal cycles: 65536\nSegments: 1"##)
            .unwrap(),
    );
}
