use crate::error::ChannelError;
use crate::verifying_key::VERIFYINGKEY;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::Groth16Verifier;
use hex_literal::hex;
use solana_program::hash::hashv;
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

    let ins: [[u8; 32]; 5] = [
        sized_range::<32>(&inputs[0..32])?,
        sized_range::<32>(&inputs[32..64])?,
        sized_range::<32>(&inputs[64..96])?,
        sized_range::<32>(&inputs[96..128])?,
        sized_range::<32>(&inputs[128..160])?,
    ];

    let mut verifier: Groth16Verifier<5> =
        Groth16Verifier::new(&proof_a, &proof_b, &proof_c, &ins, &VERIFYINGKEY)
            .map_err(|_| ChannelError::InvalidProof)?;
    verifier
        .verify()
        .map_err(|_| ChannelError::ProofVerificationFailed)
}

const CONTROL_ROOT: [u8; 32] =
    hex!("a516a057c9fbf5629106300934d48e0e775d4230e41e503347cad96fcbde7e2e");
const BN254_CONTROL_ID_BYTES: [u8; 32] =
    hex!("0eb6febcf06c5df079111be116f79bd8c7e85dc9448776ef9a59aaf2624ab551");
const OUTPUT_HASH: [u8; 32] =
    hex!("77eafeb366a78b47747de0d7bb176284085ff5564887009a5be63da32d3559d4");
const RECIEPT_CLAIM_HASH: [u8; 32] =
    hex!("cb1fefcd1f2d9a64975cbbbf6e161e2914434b0cbb9960b84df5d717e86b48af");

pub fn output_digest(
    input_digest: &[u8],
    committed_outputs: &[u8],
    assumption_digest: &[u8],
) -> [u8; 32] {
    let jbytes = [input_digest, committed_outputs].concat(); // bad copy here
    let journal = hashv(&[jbytes.as_slice()]);
    hashv(&[
        OUTPUT_HASH.as_ref(),
        journal.as_ref(),
        assumption_digest,
        &2u16.to_le_bytes(),
    ])
    .to_bytes()
}

pub fn prepare_inputs(
    image_id: &str,
    execution_digest: &[u8],
    output_digest: &[u8],
    system_exit_code: u32,
    user_exit_code: u32,
) -> Result<Vec<u8>, ChannelError> {
    let imgbytes = hex::decode(image_id).map_err(|_| ChannelError::InvalidFieldElement)?;
    let mut digest = hashv(&[
        RECIEPT_CLAIM_HASH.as_ref(),
        &[0u8; 32],
        &imgbytes,
        execution_digest,
        output_digest,
        &system_exit_code.to_le_bytes(),
        &user_exit_code.to_le_bytes(),
        &4u16.to_le_bytes(),
    ])
    .to_bytes();
    let (c0,c1) = split_digest(&mut CONTROL_ROOT.clone()).map_err(|_| ChannelError::InvalidFieldElement)?;
    let (half1_bytes, half2_bytes) =
        split_digest(&mut digest).map_err(|_| ChannelError::InvalidFieldElement)?;
    let inputs = [
        c0,
        c1,
        half1_bytes.try_into().unwrap(),
        half2_bytes.try_into().unwrap(),
        BN254_CONTROL_ID_BYTES,
    ]
    .concat();
    Ok(inputs)
}

pub fn split_digest(d: &mut [u8]) -> Result<([u8;32], [u8;32]), ChannelError> {
    d.reverse();
    let (a, b) = d.split_at(16);
    let af = to_fixed_array(a.to_vec());
    let bf = to_fixed_array(b.to_vec());
    Ok((bf, af))
}



fn to_fixed_array(input: Vec<u8>) -> [u8; 32] {
    let mut fixed_array = [0u8; 32];
    let start = core::cmp::max(32, input.len()) - core::cmp::min(32, input.len());
    fixed_array[start..].copy_from_slice(&input[input.len().saturating_sub(32)..]);
    fixed_array
}
