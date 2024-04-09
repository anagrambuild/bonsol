use crate::error::ChannelError;
use crate::verifying_keys::RISC0_VERIFYINGKEY;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use groth16_solana::groth16::Groth16Verifier;
use primitive_types::{U128, U256};
use solana_program::keccak::hashv;
use solana_program::msg;
use std::ops::Neg;
use u256_literal::u256;
use hex_literal::hex;


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



const CONTROL_ID_0_BYTES: [u8;16] = hex!("51a3d73938c3681118ba0a2549f7c188");
const CONTROL_ID_1_BYTES: [u8;16] = hex!("44f39e6e6cef91de6d743e7f5b7a1e67");
const BN254_CONTROL_ID_BYTES: [u8;32] = hex!("10ff834dbef62ccbba201ecd26a772e3036a075aacbaf47200679a11dcdcf10d");

pub fn prepare_inputs(
    image_id: &[u8],
    execution_digest: &[u8],
    output_digest: &[u8],
    system_exit_code: u32,
    user_exit_code: u32,
) -> Result<Vec<u8>, ChannelError> {
    
    let predigest = hashv(&[image_id]);
    let digest = hashv(&[
        "risc0.ReceiptClaim".as_bytes(),
        &[0u8; 32],
        predigest.as_ref(),
        execution_digest,
        output_digest,
        &system_exit_code.to_le_bytes(),
        &user_exit_code.to_le_bytes(),
        &4u16.to_le_bytes(),
    ]);
    msg!("predigest: {:?}", hex::encode(predigest.as_ref()));
    msg!("digest: {:?}", hex::encode(digest.0));
    let mut digest_bytes = digest.0;
    digest_bytes.reverse();
    let (half1, half2) = digest_bytes.split_at(16);
    let mut control_id0_bytes = [0u8; 32];
    let mut control_id1_bytes = [0u8; 32];
    let half1 = U128::from_big_endian(half1.try_into().unwrap());
    let half2 = U128::from_big_endian(half2.try_into().unwrap());
    let half1 = U256::from(half1);
    let half2 = U256::from(half2);
    let mut half1_bytes = [0u8; 32];
    half1.to_big_endian(&mut half1_bytes);
    let mut half2_bytes = [0u8; 32];
    half2.to_big_endian(&mut half2_bytes);
    control_id0_bytes[16..32].copy_from_slice(&CONTROL_ID_0_BYTES);
    control_id1_bytes[16..32].copy_from_slice(&CONTROL_ID_1_BYTES);
    half1_bytes.reverse();
    half2_bytes.reverse();
    control_id0_bytes.reverse();
    control_id1_bytes.reverse();
    let inputs = [
        control_id0_bytes,
        control_id1_bytes,
        half1_bytes,
        half2_bytes,
        BN254_CONTROL_ID_BYTES,
    ]
    .concat();
    Ok(inputs)
}
