use {anyhow::Result, tracing::info};

use risc0_zkvm::{sha::Digestible, Groth16ReceiptVerifierParameters};

use bonsol_interface::prover_version::ProverVersion;

pub fn verify_prover_version(required: ProverVersion) -> Result<()> {
    let actual_digest = Groth16ReceiptVerifierParameters::default().digest();
    let prover_digest = actual_digest.to_string();

    match required {
        ProverVersion::V1_0_1 {
            verifier_digest, ..
        } => {
            if verifier_digest != prover_digest {
                return Err(anyhow::anyhow!(
                    "Prover version mismatch, expected: {}, got: {}",
                    verifier_digest,
                    prover_digest
                ));
            }
            info!("Risc0 Prover with digest {}", verifier_digest);
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported prover version"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use {super::*, bonsol_interface::prover_version::VERSION_V1_0_1};

    #[test]
    fn test_verify_prover_version() {
        assert!(verify_prover_version(VERSION_V1_0_1).is_ok());
    }

    #[test]
    fn test_verify_prover_version_fail() {
        let version_malade = ProverVersion::V1_0_1 {
            verifier_digest: "malade",
        };
        let result = verify_prover_version(version_malade);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_default_prover_version_is_supported() {
        let result = verify_prover_version(ProverVersion::default());
        assert!(result.is_ok());
    }
}
