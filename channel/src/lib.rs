#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]

use solana_program::declare_id;
declare_id!("BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn");
pub mod error;
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
mod verifying_key;
