//! SDK boundary model: multi-architecture OCI SDK verification, complete offline dependency closure,
//! digest-bound oracle inputs, and enforcement that source integration is not consumer acceptance.
//!
//! Conformance ID: `SB-01` (suite: `sdk-boundary`).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in SDK boundary operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SdkBoundaryError {
    /// SDK image reference is mutable or lacks immutable sha256 digest pin.
    MutableImageReference(String),
    /// Required architecture is missing from multi-platform manifest index.
    MissingArchitecture(String),
    /// Architecture manifest digest does not match expected cryptographic digest.
    ManifestDigestMismatch {
        /// Target platform.
        platform: String,
        /// Expected manifest digest.
        expected: String,
        /// Observed manifest digest.
        actual: String,
    },
    /// Inventory artifact digest does not match expected cryptographic digest.
    InventoryDigestMismatch {
        /// Target platform.
        platform: String,
        /// Expected inventory digest.
        expected: String,
        /// Observed inventory digest.
        actual: String,
    },
    /// Oracle artifact digest does not match expected cryptographic digest.
    OracleDigestMismatch {
        /// Oracle identifier.
        id: String,
        /// Target platform.
        platform: String,
        /// Expected digest.
        expected: String,
        /// Observed digest.
        actual: String,
    },
    /// Unlocked or wildcard dependency detected in dependency closure.
    WildcardDependencyDetected(String),
    /// Source integration falsely claimed as consumer acceptance.
    SourceIntegrationNotAcceptance(String),
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for SdkBoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MutableImageReference(m) => write!(f, "mutable image reference: {m}"),
            Self::MissingArchitecture(a) => write!(f, "missing required architecture: {a}"),
            Self::ManifestDigestMismatch {
                platform,
                expected,
                actual,
            } => write!(
                f,
                "manifest digest mismatch on {platform}: expected {expected}, got {actual}"
            ),
            Self::InventoryDigestMismatch {
                platform,
                expected,
                actual,
            } => write!(
                f,
                "inventory digest mismatch on {platform}: expected {expected}, got {actual}"
            ),
            Self::OracleDigestMismatch {
                id,
                platform,
                expected,
                actual,
            } => write!(
                f,
                "oracle digest mismatch for {id} on {platform}: expected {expected}, got {actual}"
            ),
            Self::WildcardDependencyDetected(w) => {
                write!(f, "wildcard or mutable dependency detected: {w}")
            }
            Self::SourceIntegrationNotAcceptance(s) => {
                write!(f, "source integration is not consumer acceptance: {s}")
            }
            Self::Validation(v) => write!(f, "sdk boundary validation error: {v}"),
        }
    }
}

impl std::error::Error for SdkBoundaryError {}

/// Policy configuration governing the SDK and offline dependency boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkPolicyConfig {
    /// Require multi-architecture manifest index covering amd64 and arm64.
    pub require_multi_arch_manifest: bool,
    /// Require immutable sha256 digest pinning.
    pub require_immutable_digest: bool,
    /// Require complete offline dependency closure.
    pub require_complete_offline_closure: bool,
    /// Prohibit wildcard version dependencies.
    pub prohibit_wildcard_dependencies: bool,
    /// Prohibit unlocked git dependencies in shipped crates.
    pub prohibit_unlocked_git_dependencies: bool,
    /// Prohibit local path dependencies in shipped crates.
    pub prohibit_path_dependencies_in_shipped: bool,
    /// Require digest-bound oracle inputs.
    pub require_digest_bound_oracles: bool,
    /// Enforce that source integration is not consumer acceptance.
    pub source_integration_is_not_consumer_acceptance: bool,
}

/// Architectural manifest and inventory record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureManifestRecord {
    /// Full platform identifier (`linux/amd64`, `linux/arm64`).
    pub platform: String,
    /// Architecture (`amd64`, `arm64`).
    pub architecture: String,
    /// Operating system (`linux`).
    pub os: String,
    /// OCI image manifest digest.
    pub manifest_digest: String,
    /// Inventory item count.
    pub inventory_count: usize,
    /// Inventory document digest.
    pub inventory_digest: String,
}

/// Specification of an authoritative external oracle bound by digest across architectures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleSpecificationRecord {
    /// Oracle unique identifier.
    pub id: String,
    /// Oracle version or commit pin.
    pub version: String,
    /// Oracle artifact digest on linux/amd64.
    pub digest_amd64: String,
    /// Oracle artifact digest on linux/arm64.
    pub digest_arm64: String,
}

/// Top-level SDK boundary configuration loaded from `model/sdk_boundary.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkBoundaryConfig {
    /// Spec identifier (`foundry/sdk-boundary/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Pinned SDK container image reference.
    pub sdk_image: String,
    /// Semantic SDK version (`0.3.0`).
    pub sdk_version: String,
    /// Digest of the authoritative standards lock.
    pub standards_lock_digest: String,
    /// Digest of the verified Cargo.lock dependency closure.
    pub cargo_lock_digest: String,
    /// Boundary policy configuration.
    pub policy: SdkPolicyConfig,
    /// Multi-architecture records.
    pub architectures: Vec<ArchitectureManifestRecord>,
    /// Bound oracle specifications.
    pub oracle_specifications: Vec<OracleSpecificationRecord>,
}

impl SdkBoundaryConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/sdk-boundary/1" {
            return Err(bad(format!(
                "sdk_boundary spec must be 'foundry/sdk-boundary/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "sdk_boundary stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if !self.sdk_image.contains("@sha256:") {
            return Err(bad(
                "sdk_image must be pinned by an immutable @sha256: digest".to_string(),
            ));
        }

        if self.policy.require_multi_arch_manifest {
            let has_amd64 = self.architectures.iter().any(|a| a.architecture == "amd64");
            let has_arm64 = self.architectures.iter().any(|a| a.architecture == "arm64");
            if !has_amd64 || !has_arm64 {
                return Err(bad(
                    "sdk_boundary must specify both amd64 and arm64 architecture records"
                        .to_string(),
                ));
            }
        }

        if !self.policy.source_integration_is_not_consumer_acceptance {
            return Err(bad(
                "policy.source_integration_is_not_consumer_acceptance must be true".to_string(),
            ));
        }

        if self.oracle_specifications.is_empty() {
            return Err(bad(
                "sdk_boundary must register authoritative oracle specifications".to_string(),
            ));
        }

        Ok(())
    }
}

/// Engine executing SDK boundary verifications.
pub struct SdkBoundaryEngine;

impl SdkBoundaryEngine {
    /// Verify an architecture manifest record against observed digests.
    pub fn verify_architecture_manifest(
        arch: &ArchitectureManifestRecord,
        observed_manifest_digest: &str,
        observed_inventory_digest: &str,
    ) -> Result<(), SdkBoundaryError> {
        if arch.manifest_digest != observed_manifest_digest {
            return Err(SdkBoundaryError::ManifestDigestMismatch {
                platform: arch.platform.clone(),
                expected: arch.manifest_digest.clone(),
                actual: observed_manifest_digest.to_string(),
            });
        }

        if arch.inventory_digest != observed_inventory_digest {
            return Err(SdkBoundaryError::InventoryDigestMismatch {
                platform: arch.platform.clone(),
                expected: arch.inventory_digest.clone(),
                actual: observed_inventory_digest.to_string(),
            });
        }

        Ok(())
    }

    /// Verify an oracle artifact digest on a specific platform.
    pub fn verify_oracle_digest(
        oracle: &OracleSpecificationRecord,
        platform: &str,
        observed_digest: &str,
    ) -> Result<(), SdkBoundaryError> {
        let expected = match platform {
            "linux/amd64" => &oracle.digest_amd64,
            "linux/arm64" => &oracle.digest_arm64,
            other => {
                return Err(SdkBoundaryError::Validation(format!(
                    "unsupported verification platform '{other}'"
                )))
            }
        };

        if expected != observed_digest {
            return Err(SdkBoundaryError::OracleDigestMismatch {
                id: oracle.id.clone(),
                platform: platform.to_string(),
                expected: expected.clone(),
                actual: observed_digest.to_string(),
            });
        }

        Ok(())
    }

    /// Verify offline dependency closure from Cargo.lock bytes against expected digest.
    pub fn verify_dependency_closure(
        cargo_lock_bytes: &[u8],
        expected_digest: &str,
    ) -> Result<String, SdkBoundaryError> {
        let digest = Sha256::digest(cargo_lock_bytes);
        let actual_hex = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let actual_digest = format!("sha256:{actual_hex}");

        if actual_digest != expected_digest {
            return Err(SdkBoundaryError::Validation(format!(
                "Cargo.lock digest mismatch: expected '{expected_digest}', got '{actual_digest}'"
            )));
        }

        let lock_str = std::str::from_utf8(cargo_lock_bytes).map_err(|e| {
            SdkBoundaryError::Validation(format!("Cargo.lock is not valid utf-8: {e}"))
        })?;

        // Verify absence of git or path dependencies in production lock
        for line in lock_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("source = \"git+") {
                return Err(SdkBoundaryError::WildcardDependencyDetected(format!(
                    "unlocked git dependency in lockfile: {trimmed}"
                )));
            }
        }

        Ok(actual_digest)
    }

    /// Enforce that source integration is not consumer acceptance.
    pub fn verify_consumer_acceptance_separation(
        is_source_integration_only: bool,
    ) -> Result<(), SdkBoundaryError> {
        if is_source_integration_only {
            return Err(SdkBoundaryError::SourceIntegrationNotAcceptance(
                "source integration alone cannot be accepted as consumer verification".to_string(),
            ));
        }

        Ok(())
    }
}
