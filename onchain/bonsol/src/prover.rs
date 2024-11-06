use hex_literal::hex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProverConstants {
    pub control_root: [u8; 32],
    pub bn254_control_id_bytes: [u8; 32],
    pub output_hash: [u8; 32],
    pub receipt_claim_hash: [u8; 32],
}

pub const PROVER_CONSTANTS_V1_0_1: ProverConstants = ProverConstants {
    control_root: hex!("a516a057c9fbf5629106300934d48e0e775d4230e41e503347cad96fcbde7e2e"),
    bn254_control_id_bytes: hex!(
        "0eb6febcf06c5df079111be116f79bd8c7e85dc9448776ef9a59aaf2624ab551"
    ),
    output_hash: hex!("77eafeb366a78b47747de0d7bb176284085ff5564887009a5be63da32d3559d4"),
    receipt_claim_hash: hex!("cb1fefcd1f2d9a64975cbbbf6e161e2914434b0cbb9960b84df5d717e86b48af"),
};

impl Default for ProverConstants {
    fn default() -> Self {
        PROVER_CONSTANTS_V1_0_1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Groth16Prover {
    V1_0_1,
}

impl ProverConstants {
    fn get_constants(&self, prover: Groth16Prover) -> &ProverConstants {
        match prover {
            Groth16Prover::V1_0_1 => &PROVER_CONSTANTS_V1_0_1,
        }
    }
}

impl Default for Groth16Prover {
    fn default() -> Self {
        Groth16Prover::V1_0_1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prover_constant_default() {
        let prover_constants = ProverConstants::default();
        assert_eq!(
            prover_constants.control_root,
            hex!("a516a057c9fbf5629106300934d48e0e775d4230e41e503347cad96fcbde7e2e")
        );
        assert_eq!(
            prover_constants.bn254_control_id_bytes,
            hex!("0eb6febcf06c5df079111be116f79bd8c7e85dc9448776ef9a59aaf2624ab551")
        );
        assert_eq!(
            prover_constants.output_hash,
            hex!("77eafeb366a78b47747de0d7bb176284085ff5564887009a5be63da32d3559d4")
        );
        assert_eq!(
            prover_constants.receipt_claim_hash,
            hex!("cb1fefcd1f2d9a64975cbbbf6e161e2914434b0cbb9960b84df5d717e86b48af")
        );
    }

    #[test]
    fn test_groth16_prover_default() {
        let prover = Groth16Prover::default();
        assert_eq!(prover, Groth16Prover::V1_0_1);
    }
}
