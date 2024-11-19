use std::path::Path;

use assert_cmd::Command;

mod estimate;

pub(crate) fn bonsol_cmd() -> Command {
    let mut cmd = Command::cargo_bin("bonsol").unwrap();
    cmd.current_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .canonicalize()
            .unwrap(), // the test directory must be the project root
    );
    cmd
}
