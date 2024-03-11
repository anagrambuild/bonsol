use crate::error::ChannelError;
use crate::verifying_keys::RISC0_VERIFYINGKEY;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::Groth16Verifier;
use std::ops::Neg;

type G1 = ark_bn254::g1::G1Affine;

fn sized_range<const N: usize>(slice: &[u8]) -> Result<[u8; N], ChannelError> {
    slice
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)
}

fn change_endianness(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::new();
    for b in bytes.chunks(32) {
        for byte in b.iter().rev() {
            vec.push(*byte);
        }
    }
    vec
}

pub fn verify_risc0(proof: &[u8], inputs: &[u8]) -> Result<bool, ChannelError> {
    let ace: Vec<u8> = change_endianness(&*[&proof[0..64], &[0u8][..]].concat());
    let proof_a: G1 = G1::deserialize_with_mode(&*ace, Compress::No, Validate::No).unwrap();

    let mut proof_a_neg = [0u8; 65];
    G1::serialize_with_mode(&proof_a.neg(), &mut proof_a_neg[..], Compress::No)
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_a = change_endianness(&proof_a_neg[..64])
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_b = proof[64..192]
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_c = proof[192..256]
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let ins: [[u8; 32]; 4] = [
        sized_range::<32>(&inputs[0..32])?,
        sized_range::<32>(&inputs[32..64])?,
        sized_range::<32>(&inputs[64..96])?,
        sized_range::<32>(&inputs[96..128])?,
    ];

    let mut verifier: Groth16Verifier<4> =
        Groth16Verifier::new(&proof_a, &proof_b, &proof_c, &ins, &RISC0_VERIFYINGKEY)
            .map_err(|_| ChannelError::InvalidProof)?;
    verifier
        .verify()
        .map_err(|_| ChannelError::ProofVerificationFailed)
}
