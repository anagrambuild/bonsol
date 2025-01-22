use std::ops::Neg;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use solana_program::hash::hashv;

use crate::{
    error::ChannelError,
    prover::{PROVER_CONSTANTS_V1_0_1, PROVER_CONSTANTS_V1_2_1},
    verifying_key::VERIFYINGKEY,
};

type G1 = ark_bn254::g1::G1Affine;


pub fn verify_risc0_v1_0_1(proof: &[u8], inputs: &[u8]) -> Result<bool, ChannelError> {
    let ins: [[u8; 32]; 5] = [
        sized_range::<32>(&inputs[0..32])?,
        sized_range::<32>(&inputs[32..64])?,
        sized_range::<32>(&inputs[64..96])?,
        sized_range::<32>(&inputs[96..128])?,
        sized_range::<32>(&inputs[128..160])?,
    ];
    verify_proof::<5>(proof, ins, &VERIFYINGKEY)
}

pub fn verify_risc0_v1_2_1(proof: &[u8], inputs: &[u8]) -> Result<bool, ChannelError> {
    let ins: [[u8; 32]; 5] = [
        sized_range::<32>(&inputs[0..32])?,
        sized_range::<32>(&inputs[32..64])?,
        sized_range::<32>(&inputs[64..96])?,
        sized_range::<32>(&inputs[96..128])?,
        sized_range::<32>(&inputs[128..160])?,
    ];
    verify_proof::<5>(proof, ins, &VERIFYINGKEY)
}

fn verify_proof<const NI: usize>(
    proof: &[u8],
    inputs: [[u8; 32]; NI],
    vkey: &Groth16Verifyingkey,
) -> Result<bool, ChannelError> {
    let ace: Vec<u8> = toggle_endianness_256(&[&proof[0..64], &[0u8][..]].concat());
    let proof_a: G1 = G1::deserialize_with_mode(&*ace, Compress::No, Validate::No).unwrap();

    let mut proof_a_neg = [0u8; 65];
    G1::serialize_with_mode(&proof_a.neg(), &mut proof_a_neg[..], Compress::No)
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_a = toggle_endianness_256(&proof_a_neg[..64])
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_b = proof[64..192]
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let proof_c = proof[192..256]
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)?;

    let mut verifier: Groth16Verifier<NI> =
        Groth16Verifier::new(&proof_a, &proof_b, &proof_c, &inputs, vkey)
            .map_err(|_| ChannelError::InvalidProof)?;
    verifier
        .verify()
        .map_err(|_| ChannelError::ProofVerificationFailed)
}

pub fn output_digest_v1_0_1(
    input_digest: &[u8],
    committed_outputs: &[u8],
    assumption_digest: &[u8],
) -> [u8; 32] {
    let jbytes = [input_digest, committed_outputs].concat(); // bad copy here
    let journal = hashv(&[jbytes.as_slice()]);
    hashv(&[
        PROVER_CONSTANTS_V1_0_1.output_hash.as_ref(),
        journal.as_ref(),
        assumption_digest,
        &2u16.to_le_bytes(),
    ])
    .to_bytes()
}

pub fn prepare_inputs_v1_0_1(
    image_id: &str,
    execution_digest: &[u8],
    output_digest: &[u8],
    system_exit_code: u32,
    user_exit_code: u32,
) -> Result<Vec<u8>, ChannelError> {
    let imgbytes = hex::decode(image_id).map_err(|_| ChannelError::InvalidFieldElement)?;
    let mut digest = hashv(&[
        PROVER_CONSTANTS_V1_0_1.receipt_claim_hash.as_ref(),
        &[0u8; 32],
        &imgbytes,
        execution_digest,
        output_digest,
        &system_exit_code.to_le_bytes(),
        &user_exit_code.to_le_bytes(),
        &4u16.to_le_bytes(),
    ])
    .to_bytes();
    let (c0, c1) = split_digest_reversed(&mut PROVER_CONSTANTS_V1_0_1.control_root.clone())
        .map_err(|_| ChannelError::InvalidFieldElement)?;
    let (half1_bytes, half2_bytes) =
        split_digest_reversed(&mut digest).map_err(|_| ChannelError::InvalidFieldElement)?;
    let inputs = [
        c0,
        c1,
        half1_bytes.try_into().unwrap(),
        half2_bytes.try_into().unwrap(),
        PROVER_CONSTANTS_V1_0_1.bn254_control_id_bytes,
    ]
    .concat();
    Ok(inputs)
}

pub fn output_digest_v1_2_1(
    input_digest: &[u8],
    committed_outputs: &[u8],
    assumption_digest: &[u8],
) -> [u8; 32] {
    let jbytes = [input_digest, committed_outputs].concat(); // bad copy here
    let journal = hashv(&[jbytes.as_slice()]);
    hashv(&[
        PROVER_CONSTANTS_V1_2_1.output_hash.as_ref(),
        journal.as_ref(),
        assumption_digest,
        &2u16.to_le_bytes(),
    ])
    .to_bytes()
}

pub fn prepare_inputs_v1_2_1(
    image_id: &str,
    execution_digest: &[u8],
    output_digest: &[u8],
    system_exit_code: u32,
    user_exit_code: u32,
) -> Result<Vec<u8>, ChannelError> {
    let imgbytes = hex::decode(image_id).map_err(|_| ChannelError::InvalidFieldElement)?;
    let mut digest = hashv(&[
        PROVER_CONSTANTS_V1_2_1.receipt_claim_hash.as_ref(),
        &[0u8; 32],
        &imgbytes,
        execution_digest,
        output_digest,
        &system_exit_code.to_le_bytes(),
        &user_exit_code.to_le_bytes(),
        &4u16.to_le_bytes(),
    ])
    .to_bytes();
    let (c0, c1) = split_digest_reversed(&mut PROVER_CONSTANTS_V1_2_1.control_root.clone())
        .map_err(|_| ChannelError::InvalidFieldElement)?;
    let (half1_bytes, half2_bytes) =
        split_digest_reversed(&mut digest).map_err(|_| ChannelError::InvalidFieldElement)?;
    let inputs = [
        c0,
        c1,
        half1_bytes.try_into().unwrap(),
        half2_bytes.try_into().unwrap(),
        PROVER_CONSTANTS_V1_2_1.bn254_control_id_bytes,
    ]
    .concat();
    Ok(inputs)
}

/**
 * Reverse and split a digest into two halves
 * The first half is the left half of the digest
 * The second half is the right half of the digest
 *
 * @param d: The digest to split
 * @return A tuple containing the left and right halves of the digest
 */
pub fn split_digest_reversed_256(d: &mut [u8]) -> Result<([u8; 32], [u8; 32]), ChannelError> {
    split_digest_reversed::<32>(d)
}

fn split_digest_reversed<const N: usize>(d: &mut [u8]) -> Result<([u8; N], [u8; N]), ChannelError> {
    if d.len() != N {
        return Err(ChannelError::UnexpectedProofSystem);
    }
    d.reverse();
    let split_index = (N + 1) / 2;
    let (a, b) = d.split_at(split_index);
    let af = to_fixed_array(a);
    let bf = to_fixed_array(b);
    Ok((bf, af))
}

fn to_fixed_array<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut fixed_array = [0u8; N];
    if input.len() >= N {
        // Copy the last N bytes of input into fixed_array
        fixed_array.copy_from_slice(&input[input.len() - N..]);
    } else {
        // Copy input into the end of fixed_array
        let start = N - input.len();
        fixed_array[start..].copy_from_slice(input);
    }
    fixed_array
}

fn sized_range<const N: usize>(slice: &[u8]) -> Result<[u8; N], ChannelError> {
    slice
        .try_into()
        .map_err(|_| ChannelError::InvalidInstruction)
}

// hello ethereum! Toggle endianness of a slice of bytes assuming 256 bit word size
fn toggle_endianness_256(bytes: &[u8]) -> Vec<u8> {
    toggle_endianness::<32>(bytes)
}

fn toggle_endianness<const N: usize>(bytes: &[u8]) -> Vec<u8> {
    let mut vec = Vec::with_capacity(bytes.len());
    let chunk_size = N;

    for chunk in bytes.chunks(chunk_size) {
        // Reverse the chunk and extend the vector
        vec.extend(chunk.iter().rev());
    }

    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_endianness() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let expected = [8u8, 7, 6, 5, 4, 3, 2, 1];
        assert_eq!(toggle_endianness::<8>(&bytes), expected);
    }

    #[test]
    fn test_toggle_endianness_odd() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7];
        let expected = [7u8, 6, 5, 4, 3, 2, 1];
        assert_eq!(toggle_endianness::<7>(&bytes), expected);
    }

    #[test]
    fn test_toggle_endianness_quad_word() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let expected = [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
        assert_eq!(toggle_endianness_256(&bytes), expected);
    }

    #[test]
    fn test_split_digest() {
        let mut digest = [1u8; 32];
        digest[0] = 103;
        let (a, b) = split_digest_reversed(&mut digest).unwrap();
        let expect_digest_right = to_fixed_array::<32>(&[1u8; 16]);
        let mut expect_digest_left = expect_digest_right;
        expect_digest_left[31] = 103;
        assert_eq!(a, expect_digest_left);
        assert_eq!(b, expect_digest_right);
    }

    #[test]
    fn test_split_digest_odd() {
        let mut digest = [1u8; 31];
        digest[0] = 103;
        let (a, b) = split_digest_reversed(&mut digest).unwrap();
        let expect_digest_right = to_fixed_array::<31>(&[1u8; 16]);
        let mut expect_digest_left = to_fixed_array::<31>(&[1u8; 15]);
        expect_digest_left[30] = 103;
        assert_eq!(a, expect_digest_left);
        assert_eq!(b, expect_digest_right);
    }

    #[test]
    fn test_split_digest_16() {
        let digest = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let (a, b) = split_digest_reversed::<16>(&mut digest.to_vec()).unwrap();
        let expect_digest_left = to_fixed_array::<16>(&[7, 6, 5, 4, 3, 2, 1, 0]);
        let expect_digest_right = to_fixed_array::<16>(&[15, 14, 13, 12, 11, 10, 9, 8]);
        assert_eq!(a, expect_digest_left);
        assert_eq!(b, expect_digest_right);
    }

    #[test]
    fn test_split_digest_8() {
        let digest = [0, 1, 2, 3, 4, 5, 6, 7];
        let (a, b) = split_digest_reversed::<8>(&mut digest.to_vec()).unwrap();
        let expect_digest_left = to_fixed_array::<8>(&[3, 2, 1, 0]);
        let expect_digest_right = to_fixed_array::<8>(&[7, 6, 5, 4]);
        assert_eq!(a, expect_digest_left);
        assert_eq!(b, expect_digest_right);
    }

    #[test]
    fn test_invalid_digest_wrong_size() {
        let mut d1 = [1u8; 31];
        assert!(split_digest_reversed_256(&mut d1).is_err());
        let mut d2 = [1u8; 33];
        assert!(split_digest_reversed_256(&mut d2).is_err());
    }

    #[test]
    fn test_sized_range() {
        let slice = [1u8; 32];
        let expected = [1u8; 32];
        assert_eq!(sized_range::<32>(&slice).unwrap(), expected);
    }
}
