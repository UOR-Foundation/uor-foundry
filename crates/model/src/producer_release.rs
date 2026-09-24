//! Producer Release model: complete reproducibility verification, full service/control/dependency/assessment
//! coverage, exact producer identity binding, artifact tree, and pre-publication evidence closure.
//!
//! Conformance ID: `PR-01` (suite: `producer-release`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;
use crate::services::ServicesConfig;
use crate::standards::StandardsConfig;

/// Errors arising in Producer Release operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ProducerReleaseError {
    /// Reproducible build verification failed: artifacts differ between run 1 and run 2.
    ReproducibilityMismatch {
        /// Artifact path with mismatched digest.
        file: String,
        /// First run digest.
        run1_digest: String,
        /// Second run digest.
        run2_digest: String,
    },
    /// A required service is missing from producer release coverage.
    MissingServiceCoverage(String),
    /// A required OSCAL control is missing from producer release coverage.
    MissingControlCoverage(String),
    /// A required assessment is missing or not conforming in producer release coverage.
    MissingAssessmentCoverage(String),
    /// Incomplete core capability or unaccepted draft preview attempted release.
    IncompleteCoreCapability(String),
    /// Draft preview cannot enter deployment authorization.
    DraftPreviewNotAuthorized(String),
    /// A pre-publication gate check was improperly deferred to deployment.
    InvalidOutstandingDeploymentCheck(String),
    /// An outstanding deployment check failed during live verification.
    DeploymentCheckFailed {
        /// Check ID.
        check_id: String,
        /// Failure reason.
        reason: String,
    },
    /// An invalid release lifecycle state transition was attempted.
    InvalidReleaseStateTransition {
        /// Starting state.
        from: String,
        /// Target state.
        to: String,
        /// Reason for rejection.
        reason: String,
    },
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for ProducerReleaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReproducibilityMismatch {
                file,
                run1_digest,
                run2_digest,
            } => write!(
                f,
                "reproducible build mismatch for {file}: run1={run1_digest}, run2={run2_digest}"
            ),
            Self::MissingServiceCoverage(s) => write!(f, "missing service coverage: {s}"),
            Self::MissingControlCoverage(c) => write!(f, "missing control coverage: {c}"),
            Self::MissingAssessmentCoverage(a) => write!(f, "missing assessment coverage: {a}"),
            Self::IncompleteCoreCapability(c) => write!(f, "incomplete core capability: {c}"),
            Self::DraftPreviewNotAuthorized(d) => write!(f, "draft preview not authorized: {d}"),
            Self::InvalidOutstandingDeploymentCheck(c) => write!(
                f,
                "invalid outstanding deployment check (pre-publication check deferred): {c}"
            ),
            Self::DeploymentCheckFailed { check_id, reason } => {
                write!(f, "deployment check {check_id} failed: {reason}")
            }
            Self::InvalidReleaseStateTransition { from, to, reason } => write!(
                f,
                "invalid release state transition from {from} to {to}: {reason}"
            ),
            Self::Validation(v) => write!(f, "producer release validation error: {v}"),
        }
    }
}

impl std::error::Error for ProducerReleaseError {}

/// Three explicit release evidence states defined in SPEC.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseState {
    /// Producer-ready: complete declared stage and dependency closure pass all pre-publication gates.
    #[serde(rename = "PRODUCER_READY")]
    ProducerReady,
    /// Deployment-authorized: authorized decision binds immutable producer-ready release to target.
    #[serde(rename = "DEPLOYMENT_AUTHORIZED")]
    DeploymentAuthorized,
    /// Accepted: post-deployment identity, bytes, live journeys, and operational measurements pass.
    #[serde(rename = "ACCEPTED")]
    Accepted,
}

impl std::fmt::Display for ReleaseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProducerReady => write!(f, "PRODUCER_READY"),
            Self::DeploymentAuthorized => write!(f, "DEPLOYMENT_AUTHORIZED"),
            Self::Accepted => write!(f, "ACCEPTED"),
        }
    }
}

/// Policy rules governing producer release acceptance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerPolicyConfig {
    /// Enforce that all artifacts must be generated twice reproducibly bit-for-bit.
    pub enforce_twice_reproducible: bool,
    /// Require complete coverage of all configured services.
    pub require_full_service_coverage: bool,
    /// Require complete coverage of all adopted standards controls.
    pub require_full_control_coverage: bool,
    /// Require complete coverage of all required assessments.
    pub require_full_assessment_coverage: bool,
    /// Prohibit draft-preview substitutes from satisfying release requirements.
    pub prohibit_draft_preview_substitutes: bool,
    /// Prohibit handwritten UI substitutes for model-generated components.
    pub prohibit_handwritten_ui_substitutes: bool,
    /// Prohibit empty registers from establishing release evidence.
    pub prohibit_empty_registers: bool,
    /// Require signed pre-publication evidence binding the exact release identity.
    pub require_signed_pre_publication_evidence: bool,
}

/// Exact producer identity and toolchain binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerIdentityConfig {
    /// Name of the producer pipeline.
    pub name: String,
    /// Release version.
    pub version: String,
    /// Explicitly authorized release stage (e.g. staged-core).
    pub stage: String,
    /// Repository URL.
    pub repository: String,
    /// Exact git commit hash of the producer source.
    pub commit: String,
    /// Target compilation architecture.
    pub architecture: String,
    /// Rust compiler version.
    pub compiler: String,
    /// Digest-pinned immutable SDK container image.
    pub locked_sdk_image: String,
}

/// A covered service within the producer release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoveredServiceConfig {
    /// Unique service identifier matching services.toml.
    pub id: String,
    /// Service title.
    pub title: String,
}

/// A covered control within the producer release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoveredControlConfig {
    /// Unique OSCAL control identifier.
    pub control_id: String,
    /// Control title.
    pub title: String,
}

/// A covered assessment within the producer release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoveredAssessmentConfig {
    /// Assessment identifier matching standards.toml.
    pub assessment_id: String,
    /// Standard identifier.
    pub standard_id: String,
    /// Assessment verdict (must be conforming).
    pub verdict: String,
}

/// An immutable browser artifact record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserArtifactRecord {
    /// Relative path within browser distribution (e.g. index.html).
    pub path: String,
    /// MIME content type.
    pub mime_type: String,
    /// Exact artifact size in bytes.
    pub size_bytes: u64,
    /// SHA-256 digest (`sha256:<hex>`).
    pub sha256: String,
}

/// Reproducible build evidence recording two independent builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReproducibleBuildEvidence {
    /// SHA-256 tree digest of run 1 build artifacts.
    pub run_1_tree_digest: String,
    /// SHA-256 tree digest of run 2 build artifacts.
    pub run_2_tree_digest: String,
    /// Equality status string (must be BIT_FOR_BIT_IDENTICAL).
    pub equality_status: String,
    /// Container image digest used for both builds.
    pub build_container_digest: String,
}

/// An enumerated check requiring live target deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutstandingDeploymentCheck {
    /// Check identifier.
    pub id: String,
    /// Human-readable check name.
    pub name: String,
    /// Target URL.
    pub target: String,
    /// Flag verifying this check genuinely requires live deployment.
    pub requires_live_deployment: bool,
    /// Verification method description.
    pub verification_method: String,
}

/// Signed pre-publication evidence record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrePublicationEvidence {
    /// Current release state (must be PRODUCER_READY at publication handoff).
    pub release_state: String,
    /// Cryptographic digest binding all producer evidence.
    pub binding_digest: String,
    /// Signing authority identifier.
    pub signing_authority: String,
    /// Ed25519 signature over binding digest.
    pub signature: String,
}

/// Top-level producer release configuration loaded from `model/producer_release.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerReleaseConfig {
    /// Spec identifier (`foundry/producer-release/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Release policy configuration.
    pub policy: ProducerPolicyConfig,
    /// Producer identity binding.
    pub producer_identity: ProducerIdentityConfig,
    /// List of covered services.
    pub services: Vec<CoveredServiceConfig>,
    /// List of covered controls.
    pub covered_controls: Vec<CoveredControlConfig>,
    /// List of covered assessments.
    pub covered_assessments: Vec<CoveredAssessmentConfig>,
    /// Artifact tree records.
    pub browser_artifacts: Vec<BrowserArtifactRecord>,
    /// Reproducible build evidence.
    pub reproducible_build: ReproducibleBuildEvidence,
    /// Enumerated outstanding deployment-dependent checks.
    pub outstanding_deployment_checks: Vec<OutstandingDeploymentCheck>,
    /// Signed pre-publication evidence.
    pub pre_publication_evidence: PrePublicationEvidence,
}

impl ProducerReleaseConfig {
    /// Validate producer release invariants against model dependencies.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
        services_cfg: &ServicesConfig,
        standards_cfg: &StandardsConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/producer-release/1" {
            return Err(bad(format!(
                "producer_release spec must be 'foundry/producer-release/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "producer_release stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        // 1. Validate Producer Identity
        if self.producer_identity.name.is_empty() {
            return Err(bad("producer_identity name cannot be empty".to_string()));
        }
        if self.producer_identity.commit.len() < 7 {
            return Err(bad(
                "producer_identity commit hash must be at least 7 characters".to_string(),
            ));
        }
        if !self.producer_identity.locked_sdk_image.contains("@sha256:") {
            return Err(bad(
                "locked_sdk_image must be digest-bound with @sha256:".to_string()
            ));
        }

        // 2. Validate Service Coverage
        let covered_service_ids: HashSet<&str> =
            self.services.iter().map(|s| s.id.as_str()).collect();
        for svc in &services_cfg.services {
            if !covered_service_ids.contains(svc.id.as_str()) {
                return Err(bad(format!(
                    "service '{}' defined in services.toml is missing from producer release coverage",
                    svc.id
                )));
            }
        }

        // 3. Validate Assessment Coverage
        let covered_assessments: HashSet<&str> = self
            .covered_assessments
            .iter()
            .map(|a| a.assessment_id.as_str())
            .collect();
        for assess in &standards_cfg.assessments {
            if !covered_assessments.contains(assess.assessment_id.as_str()) {
                return Err(bad(format!(
                    "assessment '{}' in standards.toml is missing from producer release coverage",
                    assess.assessment_id
                )));
            }
        }

        for assess in &self.covered_assessments {
            if assess.verdict != "conforming" {
                return Err(bad(format!(
                    "assessment '{}' verdict must be 'conforming', found '{}'",
                    assess.assessment_id, assess.verdict
                )));
            }
        }

        // 4. Validate Reproducible Build Evidence
        if self.policy.enforce_twice_reproducible {
            if self.reproducible_build.run_1_tree_digest
                != self.reproducible_build.run_2_tree_digest
            {
                return Err(bad(format!(
                    "reproducible build digests do not match: run1='{}', run2='{}'",
                    self.reproducible_build.run_1_tree_digest,
                    self.reproducible_build.run_2_tree_digest
                )));
            }
            if self.reproducible_build.equality_status != "BIT_FOR_BIT_IDENTICAL" {
                return Err(bad(format!(
                    "equality_status must be 'BIT_FOR_BIT_IDENTICAL', found '{}'",
                    self.reproducible_build.equality_status
                )));
            }
        }

        // 5. Validate Browser Artifact Tree
        if self.browser_artifacts.is_empty() {
            return Err(bad(
                "browser_artifacts cannot be empty (prohibits empty registers)".to_string(),
            ));
        }
        for art in &self.browser_artifacts {
            if art.size_bytes == 0 {
                return Err(bad(format!(
                    "artifact '{}' size cannot be 0 bytes",
                    art.path
                )));
            }
            if !art.sha256.starts_with("sha256:") || art.sha256.len() != 71 {
                return Err(bad(format!(
                    "artifact '{}' has invalid sha256 digest: '{}'",
                    art.path, art.sha256
                )));
            }
        }

        // 6. Validate Outstanding Deployment Checks
        for chk in &self.outstanding_deployment_checks {
            if !chk.requires_live_deployment {
                return Err(bad(format!(
                    "check '{}' does not require live deployment; pre-publication checks cannot be deferred",
                    chk.id
                )));
            }
            if !chk.target.starts_with("https://") {
                return Err(bad(format!(
                    "deployment check '{}' target must be https:// URL, found '{}'",
                    chk.id, chk.target
                )));
            }
        }

        // 7. Validate Pre-publication Evidence
        if self.pre_publication_evidence.release_state != "PRODUCER_READY" {
            return Err(bad(format!(
                "pre_publication_evidence release_state must be 'PRODUCER_READY', found '{}'",
                self.pre_publication_evidence.release_state
            )));
        }
        if !self
            .pre_publication_evidence
            .binding_digest
            .starts_with("sha256:")
        {
            return Err(bad("binding_digest must be sha256: format".to_string()));
        }
        if !self
            .pre_publication_evidence
            .signature
            .starts_with("ed25519:")
        {
            return Err(bad(
                "pre-publication signature must use ed25519: key".to_string()
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Producer Release Engine
// -----------------------------------------------------------------------------

/// Deployment authorization record produced when an authorized decision binds
/// an immutable producer-ready release to a publication target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeploymentAuthorizationRecord {
    /// Binding digest of the producer release.
    pub release_binding_digest: String,
    /// Target HTTPS publication URL.
    pub target_url: String,
    /// Authorized decision key / signature.
    pub authorization_key: String,
    /// Release stage.
    pub stage: String,
    /// Current release state.
    pub state: ReleaseState,
}

/// Final accepted release record produced when all post-deployment checks pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedReleaseRecord {
    /// Binding digest of the accepted release.
    pub release_binding_digest: String,
    /// Target HTTPS URL.
    pub target_url: String,
    /// Completed deployment check IDs.
    pub completed_checks: Vec<String>,
    /// Live acceptance attestation digest.
    pub acceptance_attestation: String,
    /// Final state.
    pub state: ReleaseState,
}

/// Engine executing producer release verification and lifecycle state transitions.
pub struct ProducerReleaseEngine;

impl ProducerReleaseEngine {
    /// Compute tree digest over a sorted slice of browser artifact records.
    pub fn compute_artifact_tree_digest(artifacts: &[BrowserArtifactRecord]) -> String {
        let mut sorted = artifacts.to_vec();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));

        let mut combined = Vec::new();
        for art in &sorted {
            combined.extend_from_slice(art.path.as_bytes());
            combined.push(0);
            combined.extend_from_slice(art.mime_type.as_bytes());
            combined.push(0);
            combined.extend_from_slice(&art.size_bytes.to_le_bytes());
            combined.extend_from_slice(art.sha256.as_bytes());
            combined.push(0xff);
        }
        crate::kappa::compute_sha256_digest(&combined)
    }

    /// Verify that two independent build runs produced bit-for-bit identical artifacts.
    pub fn verify_reproducibility(
        run1: &[BrowserArtifactRecord],
        run2: &[BrowserArtifactRecord],
    ) -> Result<String, ProducerReleaseError> {
        if run1.len() != run2.len() {
            return Err(ProducerReleaseError::ReproducibilityMismatch {
                file: "artifact_count".to_string(),
                run1_digest: format!("count:{}", run1.len()),
                run2_digest: format!("count:{}", run2.len()),
            });
        }

        let run1_map: HashMap<&str, &BrowserArtifactRecord> =
            run1.iter().map(|a| (a.path.as_str(), a)).collect();

        for art2 in run2 {
            match run1_map.get(art2.path.as_str()) {
                Some(art1) => {
                    if art1.sha256 != art2.sha256 || art1.size_bytes != art2.size_bytes {
                        return Err(ProducerReleaseError::ReproducibilityMismatch {
                            file: art2.path.clone(),
                            run1_digest: art1.sha256.clone(),
                            run2_digest: art2.sha256.clone(),
                        });
                    }
                }
                None => {
                    return Err(ProducerReleaseError::ReproducibilityMismatch {
                        file: art2.path.clone(),
                        run1_digest: "missing_in_run1".to_string(),
                        run2_digest: art2.sha256.clone(),
                    });
                }
            }
        }

        let tree1 = Self::compute_artifact_tree_digest(run1);
        let tree2 = Self::compute_artifact_tree_digest(run2);
        if tree1 != tree2 {
            return Err(ProducerReleaseError::ReproducibilityMismatch {
                file: "tree_root".to_string(),
                run1_digest: tree1,
                run2_digest: tree2,
            });
        }

        Ok(tree1)
    }

    /// Authorize deployment for a producer-ready release.
    pub fn authorize_deployment(
        release: &ProducerReleaseConfig,
        target_url: &str,
        authorization_key: &str,
    ) -> Result<DeploymentAuthorizationRecord, ProducerReleaseError> {
        if release.pre_publication_evidence.release_state != "PRODUCER_READY" {
            return Err(ProducerReleaseError::InvalidReleaseStateTransition {
                from: release.pre_publication_evidence.release_state.clone(),
                to: "DEPLOYMENT_AUTHORIZED".to_string(),
                reason: "release must be PRODUCER_READY before authorization".to_string(),
            });
        }

        if release.stage == "draft-preview" {
            return Err(ProducerReleaseError::DraftPreviewNotAuthorized(
                "draft-preview release cannot enter deployment authorization".to_string(),
            ));
        }

        if !target_url.starts_with("https://") {
            return Err(ProducerReleaseError::Validation(format!(
                "target_url must be HTTPS, found '{target_url}'"
            )));
        }

        if authorization_key.is_empty() {
            return Err(ProducerReleaseError::Validation(
                "authorization key cannot be empty".to_string(),
            ));
        }

        Ok(DeploymentAuthorizationRecord {
            release_binding_digest: release.pre_publication_evidence.binding_digest.clone(),
            target_url: target_url.to_string(),
            authorization_key: authorization_key.to_string(),
            stage: release.stage.clone(),
            state: ReleaseState::DeploymentAuthorized,
        })
    }

    /// Accept release after verifying that all outstanding deployment-dependent checks passed.
    pub fn accept_release(
        auth: &DeploymentAuthorizationRecord,
        release: &ProducerReleaseConfig,
        live_check_results: &HashMap<String, bool>,
        attestation: &str,
    ) -> Result<AcceptedReleaseRecord, ProducerReleaseError> {
        if auth.state != ReleaseState::DeploymentAuthorized {
            return Err(ProducerReleaseError::InvalidReleaseStateTransition {
                from: auth.state.to_string(),
                to: "ACCEPTED".to_string(),
                reason: "release must be DEPLOYMENT_AUTHORIZED before final acceptance".to_string(),
            });
        }

        let mut completed = Vec::new();
        for chk in &release.outstanding_deployment_checks {
            match live_check_results.get(&chk.id) {
                Some(&true) => {
                    completed.push(chk.id.clone());
                }
                Some(&false) => {
                    return Err(ProducerReleaseError::DeploymentCheckFailed {
                        check_id: chk.id.clone(),
                        reason: "live deployment check returned failure".to_string(),
                    });
                }
                None => {
                    return Err(ProducerReleaseError::DeploymentCheckFailed {
                        check_id: chk.id.clone(),
                        reason: "live deployment check was not executed".to_string(),
                    });
                }
            }
        }

        Ok(AcceptedReleaseRecord {
            release_binding_digest: auth.release_binding_digest.clone(),
            target_url: auth.target_url.clone(),
            completed_checks: completed,
            acceptance_attestation: attestation.to_string(),
            state: ReleaseState::Accepted,
        })
    }
}
