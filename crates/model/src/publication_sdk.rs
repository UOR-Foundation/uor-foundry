//! Publication SDK model: source-free acquisition, readiness verification, target authorization,
//! confined atomic artifact export, live deployment verification, and accepted-release rollback.
//!
//! Conformance ID: `PS-01` (suite: `publication-sdk`).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;
use crate::producer_release::{BrowserArtifactRecord, ProducerReleaseConfig};

/// Errors arising in Publication SDK operations.
#[derive(Debug, Clone, PartialEq)]
pub enum PublicationError {
    /// Source-free acquisition failed because application source was required.
    SourceRequiredError(String),
    /// Producer readiness verification failed due to missing or unaccepted evidence.
    ReadinessVerificationFailed(String),
    /// Target URL or authorization decision is missing, invalid, or unauthorized.
    TargetNotAuthorized(String),
    /// Byte substitution detected: exported artifact does not match immutable producer digest.
    ByteSubstitutionDetected {
        /// Asset path.
        file: String,
        /// Expected SHA-256 digest from producer release.
        expected: String,
        /// Actual SHA-256 digest observed in export.
        actual: String,
    },
    /// Partial artifact export detected (missing required distribution files).
    PartialExportRejected(String),
    /// Stale or unaligned release evidence presented.
    StaleEvidenceRejected(String),
    /// Automatic rollback was executed following live verification failure.
    RollbackExecuted {
        /// Failure reason that triggered the rollback.
        reason: String,
        /// Digest of the restored previous stable release.
        restored_digest: String,
    },
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceRequiredError(s) => write!(f, "source-free violation: {s}"),
            Self::ReadinessVerificationFailed(r) => write!(f, "readiness verification failed: {r}"),
            Self::TargetNotAuthorized(t) => write!(f, "target authorization failed: {t}"),
            Self::ByteSubstitutionDetected { file, expected, actual } => write!(
                f,
                "byte substitution detected for {file}: expected {expected}, actual {actual}"
            ),
            Self::PartialExportRejected(p) => write!(f, "partial export rejected: {p}"),
            Self::StaleEvidenceRejected(s) => write!(f, "stale evidence rejected: {s}"),
            Self::RollbackExecuted { reason, restored_digest } => write!(
                f,
                "automatic rollback executed ({reason}); restored previous release {restored_digest}"
            ),
            Self::Validation(v) => write!(f, "publication SDK validation error: {v}"),
        }
    }
}

impl std::error::Error for PublicationError {}

/// Lifecycle states across publication SDK handoff and deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationState {
    /// Acquired source-free from immutable producer release without compiling.
    Acquired,
    /// Producer readiness and pre-publication evidence verified.
    ReadinessVerified,
    /// Target deployment decision bound and authorized.
    TargetAuthorized,
    /// Confined atomic export completed with unchanged bytes.
    Exported,
    /// Deployed live and verified on target HTTPS origin.
    PublishedLive,
    /// Rolled back to previous stable release following post-deploy check failure.
    RolledBack,
}

/// Publication policy governing handoff, export, and rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationPolicyConfig {
    /// Enforce source-free acquisition (no application source or rebuilding).
    pub enforce_source_free_acquisition: bool,
    /// Enforce atomic artifact export without intermediate partial states.
    pub enforce_atomic_artifact_export: bool,
    /// Require exact producer release identity binding.
    pub require_exact_producer_binding: bool,
    /// Require explicit target authorization decision.
    pub require_target_authorization_decision: bool,
    /// Prohibit byte substitution across all exported assets.
    pub prohibit_byte_substitution: bool,
    /// Prohibit stale, partial, or unaligned pre-publication evidence.
    pub prohibit_stale_or_partial_evidence: bool,
    /// Enable automated rollback to accepted previous release on verification failure.
    pub enable_accepted_release_rollback: bool,
}

/// Configuration describing expected producer release handoff properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerHandoffConfig {
    /// Expected producer release specification version.
    pub producer_release_spec: String,
    /// Expected producer pipeline identity.
    pub expected_producer_identity: String,
    /// Expected release stage (`staged-core`).
    pub expected_stage: String,
    /// Expected producer binding digest.
    pub producer_binding_digest: String,
}

/// Configuration for atomic browser artifact export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicExportConfig {
    /// Command/API used for export (`export-browser`).
    pub export_api: String,
    /// Target output directory.
    pub output_dir: String,
    /// Required assets that must all be atomically exported.
    pub required_assets: Vec<String>,
    /// Guarantee that bytes remain unchanged from producer release.
    pub preserve_unchanged_bytes: bool,
}

/// Policy for automatic rollback on live deployment verification failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPolicyConfig {
    /// Retained digest of the previously accepted release.
    pub retained_previous_release_digest: String,
    /// Flag indicating rollback triggers on verification failure.
    pub rollback_trigger_on_verification_failure: bool,
    /// Require atomic reversion of deployed assets.
    pub atomic_reversion_required: bool,
}

/// Top-level publication SDK configuration loaded from `model/publication_sdk.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationSdkConfig {
    /// Spec identifier (`foundry/publication-sdk/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Target publication domain (`https://uor-foundation.github.io/foundry-web/`).
    pub target_domain: String,
    /// Publication policy.
    pub policy: PublicationPolicyConfig,
    /// Producer handoff configuration.
    pub producer_handoff: ProducerHandoffConfig,
    /// Atomic export configuration.
    pub atomic_export: AtomicExportConfig,
    /// Rollback policy configuration.
    pub rollback_policy: RollbackPolicyConfig,
}

impl PublicationSdkConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
        producer_cfg: &ProducerReleaseConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/publication-sdk/1" {
            return Err(bad(format!(
                "publication_sdk spec must be 'foundry/publication-sdk/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "publication_sdk stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if !self.target_domain.starts_with("https://") {
            return Err(bad("target_domain must be an HTTPS origin".to_string()));
        }

        // Validate producer handoff alignment
        if self.producer_handoff.expected_producer_identity != producer_cfg.producer_identity.name {
            return Err(bad(format!(
                "expected_producer_identity '{}' does not match producer release name '{}'",
                self.producer_handoff.expected_producer_identity,
                producer_cfg.producer_identity.name
            )));
        }

        if self.producer_handoff.producer_binding_digest
            != producer_cfg.pre_publication_evidence.binding_digest
        {
            return Err(bad(format!(
                "producer_binding_digest '{}' does not match producer pre_publication_evidence '{}'",
                self.producer_handoff.producer_binding_digest,
                producer_cfg.pre_publication_evidence.binding_digest
            )));
        }

        // Validate atomic export assets match producer browser artifacts
        let producer_artifact_paths: HashSet<&str> = producer_cfg
            .browser_artifacts
            .iter()
            .map(|a| a.path.as_str())
            .collect();

        for req in &self.atomic_export.required_assets {
            if !producer_artifact_paths.contains(req.as_str()) {
                return Err(bad(format!(
                    "required export asset '{req}' is missing from producer release browser artifacts"
                )));
            }
        }

        if self
            .rollback_policy
            .retained_previous_release_digest
            .is_empty()
        {
            return Err(bad(
                "retained_previous_release_digest cannot be empty".to_string()
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Publication SDK Engine & Handoff Package
// -----------------------------------------------------------------------------

/// Record of an exported browser asset with size and SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportedAssetRecord {
    /// Relative path (e.g. index.html).
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// SHA-256 content digest.
    pub sha256: String,
}

/// Self-contained immutable publication handoff package delivered to foundry-web.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationHandoffPackage {
    /// Producer identity name.
    pub producer_identity: String,
    /// Cryptographic binding digest of the producer release.
    pub binding_digest: String,
    /// Flag verifying this package was acquired source-free.
    pub source_free: bool,
    /// Release stage.
    pub stage: String,
    /// List of exported browser assets with verified digests.
    pub exported_assets: Vec<ExportedAssetRecord>,
    /// Authorized target URL (empty until target authorization).
    pub target_url: String,
    /// Authorization key / signature (empty until target authorization).
    pub authorization_key: String,
    /// Current publication lifecycle state.
    pub state: PublicationState,
}

/// Engine executing Publication SDK acquisition, readiness, export, verification, and rollback.
pub struct PublicationSdkEngine;

impl PublicationSdkEngine {
    /// Acquire release source-free from producer configuration.
    pub fn acquire_source_free(
        producer_release: &ProducerReleaseConfig,
    ) -> Result<PublicationHandoffPackage, PublicationError> {
        if producer_release.pre_publication_evidence.release_state != "PRODUCER_READY" {
            return Err(PublicationError::ReadinessVerificationFailed(format!(
                "producer release is in state '{}', requires PRODUCER_READY",
                producer_release.pre_publication_evidence.release_state
            )));
        }

        let assets: Vec<ExportedAssetRecord> = producer_release
            .browser_artifacts
            .iter()
            .map(|a| ExportedAssetRecord {
                path: a.path.clone(),
                size_bytes: a.size_bytes,
                sha256: a.sha256.clone(),
            })
            .collect();

        Ok(PublicationHandoffPackage {
            producer_identity: producer_release.producer_identity.name.clone(),
            binding_digest: producer_release
                .pre_publication_evidence
                .binding_digest
                .clone(),
            source_free: true,
            stage: producer_release.stage.clone(),
            exported_assets: assets,
            target_url: String::new(),
            authorization_key: String::new(),
            state: PublicationState::Acquired,
        })
    }

    /// Verify producer readiness and binding integrity against publication configuration.
    pub fn verify_readiness(
        package: &mut PublicationHandoffPackage,
        config: &PublicationSdkConfig,
    ) -> Result<(), PublicationError> {
        if !package.source_free {
            return Err(PublicationError::SourceRequiredError(
                "package acquisition violated source-free invariant".to_string(),
            ));
        }

        if package.producer_identity != config.producer_handoff.expected_producer_identity {
            return Err(PublicationError::ReadinessVerificationFailed(format!(
                "producer identity mismatch: expected '{}', found '{}'",
                config.producer_handoff.expected_producer_identity, package.producer_identity
            )));
        }

        if package.binding_digest != config.producer_handoff.producer_binding_digest {
            return Err(PublicationError::StaleEvidenceRejected(format!(
                "stale binding digest: expected '{}', found '{}'",
                config.producer_handoff.producer_binding_digest, package.binding_digest
            )));
        }

        package.state = PublicationState::ReadinessVerified;
        Ok(())
    }

    /// Bind and authorize target deployment destination.
    pub fn authorize_target(
        package: &mut PublicationHandoffPackage,
        target_url: &str,
        authorization_key: &str,
    ) -> Result<(), PublicationError> {
        if package.state != PublicationState::ReadinessVerified {
            return Err(PublicationError::TargetNotAuthorized(format!(
                "cannot authorize target in state '{:?}'; requires ReadinessVerified",
                package.state
            )));
        }

        if !target_url.starts_with("https://") {
            return Err(PublicationError::TargetNotAuthorized(format!(
                "target URL must be HTTPS origin, found '{target_url}'"
            )));
        }

        if authorization_key.is_empty() {
            return Err(PublicationError::TargetNotAuthorized(
                "authorization key cannot be empty".to_string(),
            ));
        }

        package.target_url = target_url.to_string();
        package.authorization_key = authorization_key.to_string();
        package.state = PublicationState::TargetAuthorized;
        Ok(())
    }

    /// Execute confined atomic artifact export, verifying byte integrity against producer records.
    pub fn export_atomic_browser(
        package: &mut PublicationHandoffPackage,
        producer_artifacts: &[BrowserArtifactRecord],
        config: &PublicationSdkConfig,
    ) -> Result<Vec<ExportedAssetRecord>, PublicationError> {
        if package.state != PublicationState::TargetAuthorized {
            return Err(PublicationError::Validation(format!(
                "cannot export in state '{:?}'; requires TargetAuthorized",
                package.state
            )));
        }

        let producer_map: std::collections::HashMap<&str, &BrowserArtifactRecord> =
            producer_artifacts
                .iter()
                .map(|a| (a.path.as_str(), a))
                .collect();

        // Verify all required assets are present and check for byte substitution
        for req in &config.atomic_export.required_assets {
            match producer_map.get(req.as_str()) {
                Some(prod_art) => {
                    // Check corresponding asset in package
                    let pkg_art = package
                        .exported_assets
                        .iter()
                        .find(|a| a.path == *req)
                        .ok_or_else(|| {
                            PublicationError::PartialExportRejected(format!(
                                "required asset '{req}' missing from handoff package"
                            ))
                        })?;

                    if pkg_art.sha256 != prod_art.sha256
                        || pkg_art.size_bytes != prod_art.size_bytes
                    {
                        return Err(PublicationError::ByteSubstitutionDetected {
                            file: req.clone(),
                            expected: prod_art.sha256.clone(),
                            actual: pkg_art.sha256.clone(),
                        });
                    }
                }
                None => {
                    return Err(PublicationError::PartialExportRejected(format!(
                        "required asset '{req}' missing from producer artifacts"
                    )));
                }
            }
        }

        package.state = PublicationState::Exported;
        Ok(package.exported_assets.clone())
    }

    /// Verify live post-deployment assets against expected package and rollback atomically on failure.
    pub fn verify_live_deployment_and_rollback_on_failure(
        package: &mut PublicationHandoffPackage,
        live_deployed_assets: &[ExportedAssetRecord],
        previous_release_digest: &str,
    ) -> Result<PublicationState, PublicationError> {
        if package.state != PublicationState::Exported {
            return Err(PublicationError::Validation(format!(
                "cannot verify live deployment in state '{:?}'; requires Exported",
                package.state
            )));
        }

        let mut mismatch = None;
        if live_deployed_assets.len() != package.exported_assets.len() {
            mismatch = Some(format!(
                "asset count mismatch: expected {}, deployed {}",
                package.exported_assets.len(),
                live_deployed_assets.len()
            ));
        } else {
            let deployed_map: std::collections::HashMap<&str, &ExportedAssetRecord> =
                live_deployed_assets
                    .iter()
                    .map(|a| (a.path.as_str(), a))
                    .collect();

            for exp in &package.exported_assets {
                match deployed_map.get(exp.path.as_str()) {
                    Some(dep) => {
                        if dep.sha256 != exp.sha256 || dep.size_bytes != exp.size_bytes {
                            mismatch = Some(format!(
                                "asset '{}' corrupted or substituted: expected '{}', deployed '{}'",
                                exp.path, exp.sha256, dep.sha256
                            ));
                            break;
                        }
                    }
                    None => {
                        mismatch = Some(format!("asset '{}' missing on live deployment", exp.path));
                        break;
                    }
                }
            }
        }

        if let Some(err_msg) = mismatch {
            package.state = PublicationState::RolledBack;
            return Err(PublicationError::RollbackExecuted {
                reason: err_msg,
                restored_digest: previous_release_digest.to_string(),
            });
        }

        package.state = PublicationState::PublishedLive;
        Ok(PublicationState::PublishedLive)
    }
}
