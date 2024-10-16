#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]
use solana_program::declare_id;
pub mod actions;
mod assertions;
pub mod error;
pub mod program;
pub mod proof_handling;
pub mod utilities;
mod verifying_key;

declare_id!("BoNsHRcyLLNdtnoDf8hiCNZpyehMC4FDMxs6NTxFi3ew");

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
