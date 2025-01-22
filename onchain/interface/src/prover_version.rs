use std::{convert::{TryFrom, TryInto}, fmt::Display};

use bonsol_schema::ProverVersion as FBSProverVersion;

pub const DIGEST_V1_0_1_BYTES: &str =
    "310fe598e8e3e92fa805bc272d7f587898bb8b68c4d5d7938db884abaa76e15c";

pub const DIGEST_V1_2_1_BYTES: &str =
    "c101b42bcacd62e35222b1207223250814d05dd41d41f8cadc1f16f86707ae15";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverVersion {
    V1_0_1 {
        verifier_digest: &'static str,
    },
    V1_2_1 {
        verifier_digest: &'static str,
    },
    UnsupportedVersion,
}

impl Display for ProverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProverVersion::V1_0_1 { .. } => write!(f, "V1_0_1"),
            ProverVersion::V1_2_1 { .. } => write!(f, "V1_2_1"),
            ProverVersion::UnsupportedVersion => write!(f, "UnsupportedVersion"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProverVersionError {
    UnsupportedVersion,
}

impl Default for ProverVersion {
    fn default() -> Self {
        VERSION_V1_2_1
    }
}

impl TryFrom<FBSProverVersion> for ProverVersion {
    type Error = ProverVersionError;

    fn try_from(prover_version: FBSProverVersion) -> Result<Self, Self::Error> {
        match prover_version {
            FBSProverVersion::V1_0_1  => Ok(VERSION_V1_0_1),
            FBSProverVersion::V1_2_1 | FBSProverVersion::DEFAULT => Ok(VERSION_V1_2_1),
            _ => Err(ProverVersionError::UnsupportedVersion),
        }
    }
}

impl TryInto<FBSProverVersion> for ProverVersion {
    type Error = ProverVersionError;

    fn try_into(self) -> Result<FBSProverVersion, Self::Error> {
        // this is to allow for a future error where a version is missed
        #[allow(unreachable_patterns)]
        match self {
            ProverVersion::V1_0_1 { .. } => Ok(FBSProverVersion::V1_0_1),
            _ => Err(ProverVersionError::UnsupportedVersion),
        }
    }
}

pub const VERSION_V1_0_1: ProverVersion = ProverVersion::V1_0_1 {
    verifier_digest: DIGEST_V1_0_1_BYTES,
};

pub const VERSION_V1_2_1: ProverVersion = ProverVersion::V1_2_1 {
    verifier_digest: DIGEST_V1_2_1_BYTES,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_version() {
        assert_eq!(ProverVersion::default(), VERSION_V1_2_1);
    }

    #[test]
    fn test_verify_prover_version() {
        assert_eq!(
            VERSION_V1_0_1,
            ProverVersion::V1_0_1 {
                verifier_digest: DIGEST_V1_0_1_BYTES
            }
        );
    }

    #[test]
    fn test_try_from_v1_0_1() {
        let version = ProverVersion::try_from(FBSProverVersion::V1_0_1);
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), VERSION_V1_0_1);
    }

    #[test]
    fn test_try_into_v1_0_1() {
        let fbs_version: Result<FBSProverVersion, ProverVersionError> = VERSION_V1_0_1.try_into();
        assert!(fbs_version.is_ok());
        assert_eq!(fbs_version.unwrap(), FBSProverVersion::V1_0_1);
    }

    #[test]
    fn test_try_from_unsupported_version() {
        let unsupported_version = FBSProverVersion(u16::MAX);
        let version = ProverVersion::try_from(unsupported_version);
        assert!(version.is_err());
        assert_eq!(version.unwrap_err(), ProverVersionError::UnsupportedVersion);
    }

    #[test]
    fn test_try_into_unsupported_version() {
        let unsupported_version = ProverVersion::UnsupportedVersion;
        let fbs_version: Result<FBSProverVersion, ProverVersionError> =
            unsupported_version.try_into();
        assert!(fbs_version.is_err());
        assert_eq!(
            fbs_version.unwrap_err(),
            ProverVersionError::UnsupportedVersion
        );
    }

    #[test]
    fn test_default_into_current_version() {
        let default_version = FBSProverVersion::DEFAULT;
        let version = ProverVersion::try_from(default_version);
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), VERSION_V1_2_1);
    }
}
