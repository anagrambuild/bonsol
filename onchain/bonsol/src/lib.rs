#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod actions;
pub mod error;
pub mod program;
pub mod proof_handling;
pub mod prover;
pub mod utilities;

mod assertions;
mod verifying_key;

use solana_program::declare_id;

declare_id!("BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
