use iop::*;
use anyhow::{Context, Result};
use num_bigint::BigUint;
use num_traits::Num;
use risc0_core::field::baby_bear::BabyBearElem;
use risc0_zkp::core::{
    digest::{Digest, DIGEST_WORDS},
    hash::poseidon_254::digest_to_fr,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io::Write;

pub async fn async_to_json<R: AsyncRead + std::marker::Unpin, W: AsyncWrite + std::marker::Unpin>(
    mut reader: R,
    mut writer: W,
) -> Result<()> {
    let mut iop = vec![0u32; K_SEAL_WORDS];
    reader
        .read_exact(bytemuck::cast_slice_mut(&mut iop))
        .await?;
    let mut mem = Vec::new();
    writeln!(mem, "{{\n  \"iop\": [")?;

    let mut pos = 0;
    for seal_type in K_SEAL_TYPES.iter().take(K_SEAL_ELEMS) {
        if pos != 0 {
            writeln!(mem, ",")?;
        }
        match seal_type {
            IopType::Fp => {
                let value = BabyBearElem::new_raw(iop[pos]).as_u32();
                pos += 1;
                writeln!(mem, "    \"{value}\"")?;
            }
            _ => {
                let digest = Digest::try_from(&iop[pos..pos + DIGEST_WORDS])?;
                let value = digest_to_decimal(&digest)?;
                pos += 8;
                writeln!(mem, "    \"{value}\"")?;
            }
        }
    }
    write!(mem, "  ]\n}}")?;
    writer.write_all(mem.as_slice()).await?;
    writer.flush().await?;
    Ok(())
}

fn digest_to_decimal(digest: &Digest) -> Result<String> {
    to_decimal(&format!("{:?}", digest_to_fr(digest))).context("digest_to_decimal failed")
}

fn to_decimal(s: &str) -> Option<String> {
    s.strip_prefix("Fr(0x")
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|stripped| BigUint::from_str_radix(stripped, 16).ok())
        .map(|n| n.to_str_radix(10))
}


