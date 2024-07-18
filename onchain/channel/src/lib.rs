#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]
use solana_program::declare_id;
mod assertions;
pub mod error;
pub mod program;
pub mod proof_handling;
mod verifying_key;
pub mod utilities;
pub mod actions;

declare_id!("BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

