//! Bare bones upper bound estimator that uses the rv32im
//! emulation utils for fast lookups in the opcode list
//! to extract the cycle count from an elf.
//!
//! This can be extended in a similar fashion to target
//! specific functions as sov-cycle-tracer does and the
//! defaults made into command line args.

use std::{fs, path::PathBuf};

use anyhow::Result;
use risc0_circuit_rv32im::prove::emu::{
    exec::{execute_elf, DEFAULT_SEGMENT_LIMIT_PO2},
    testutil::{NullSyscall, DEFAULT_SESSION_LIMIT},
};

pub fn estimate(elf: PathBuf) -> Result<()> {
    // TODO: We probably want to be able to let the user decide some of this
    let cycles: usize = execute_elf(
        &fs::read(elf)?,
        DEFAULT_SEGMENT_LIMIT_PO2,
        DEFAULT_SESSION_LIMIT,
        &NullSyscall::default(),
        None,
    )?
    .segments
    .iter()
    .try_fold(0, |acc, s| -> Result<usize> {
        let trace = s.preflight()?;
        let segment_cycles = trace.pre.cycles.len() + trace.body.cycles.len();
        Ok(acc + segment_cycles)
    })?;

    println!("number of cycles: {cycles}");

    Ok(())
}
