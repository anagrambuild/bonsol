#![allow(clippy::arithmetic_side_effects)]
#![cfg_attr(not(test), forbid(unsafe_code))]

use solana_program::declare_id;
declare_id!("BoNSrwTtTM4PRkbbPvehk1XzHC65cKfdNSod9FyTejRn");
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod error;
mod verifying_key;
