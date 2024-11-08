use std::io;
use std::process;

use anyhow::Result;
use solana_sdk::signer::Signer;
use sov_cycle_macros::cycle_tracker as risc0_cycle_tracker;

use crate::build;

// 1. user gives a path to a manifest
// 2. we compile it with bonsol build
// 3. we wrap the compiled bin in a function which has the macro applied to it
// 4. we call the function which runs the program and tracks the cycles, probably get the cycle count from stdout
// 5. apply a function which gets the upper bound of the cycles
// 6. return the estimate to the user
pub fn estimate(
    keypair: &impl Signer,
    zk_program_path: String,
    runtime_args: &[AsRef<str>],
    build: bool,
) -> Result<()> {
    if let Some(Err(err)) = build.then(|| build::build(keypair, zk_program_path)) {
        anyhow::bail!("failed to build zk program: {err:?}")
    }

    let command = todo!();
    let _output = program_entrypoint(command, runtime_args)?;
    Ok(())
}

/// Run the program with the cycle_tracker and return the output
#[cfg_attr(all(target_os = "zkvm"), risc0_cycle_tracker)]
fn program_entrypoint(command: String, runtime_args: &[AsRef<str>]) -> io::Result<process::Output> {
    process::Command::new(command).args(runtime_args).output()
}
