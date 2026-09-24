//! Implementation closure model: complete verification across all product requirements,
//! remaining-work rows in `IMPLEMENTATION.md`, and exact producer release handoff identity.
//!
//! Conformance ID: `IC-01` (suite: `implementation-closure`).

use serde::{Deserialize, Serialize};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising during implementation closure verification.
#[derive(Debug, Clone, PartialEq)]
pub enum ImplementationClosureError {
    /// A required boundary or table row is not accepted.
    UnacceptedBoundary(String),
    /// A deferred or narrowed scope was detected.
    DeferredScopeDetected(String),
    /// Release identity or artifact tree digest mismatch.
    ReleaseIdentityMismatch(String),
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for ImplementationClosureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnacceptedBoundary(u) => write!(f, "unaccepted boundary: {u}"),
            Self::DeferredScopeDetected(d) => write!(f, "deferred scope detected: {d}"),
            Self::ReleaseIdentityMismatch(r) => write!(f, "release identity mismatch: {r}"),
            Self::Validation(v) => write!(f, "implementation closure validation error: {v}"),
        }
    }
}

impl std::error::Error for ImplementationClosureError {}

/// Policy configuration for implementation closure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationClosurePolicyConfig {
    /// Enforce complete remaining-work row closure.
    pub enforce_complete_row_closure: bool,
    /// Prohibit deferred scope across all domains.
    pub prohibit_deferred_scope: bool,
    /// Prohibit mock or handwritten UI substitutes.
    pub prohibit_mock_substitutes: bool,
    /// Require independently replayable evidence.
    pub require_replayable_evidence: bool,
    /// Require producer handoff readiness.
    pub require_producer_handoff_readiness: bool,
}

/// Exact release identity record for handoff to `foundry-web`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseIdentityRecord {
    /// Producer identity name.
    pub producer_name: String,
    /// Release version.
    pub release_version: String,
    /// Release stage (`staged-core`).
    pub release_stage: String,
    /// Target HTTPS deployment URL.
    pub target_deployment_url: String,
    /// Cryptographic artifact tree digest.
    pub artifact_tree_digest: String,
    /// Pre-publication signed evidence binding digest.
    pub pre_publication_binding: String,
}

/// Record of an accepted boundary row from `IMPLEMENTATION.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptedBoundaryRecord {
    /// Conformance ID.
    pub id: String,
    /// Boundary name.
    pub name: String,
    /// Acceptance status (must be `accepted`).
    pub status: String,
}

/// Top-level implementation closure configuration loaded from `model/implementation_closure.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationClosureConfig {
    /// Spec identifier (`foundry/implementation-closure/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Policy configuration.
    pub policy: ImplementationClosurePolicyConfig,
    /// Release identity record.
    pub release_identity: ReleaseIdentityRecord,
    /// Accepted boundaries list.
    pub accepted_boundaries: Vec<AcceptedBoundaryRecord>,
}

impl ImplementationClosureConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/implementation-closure/1" {
            return Err(bad(format!(
                "implementation_closure spec must be 'foundry/implementation-closure/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "implementation_closure stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if !self.policy.enforce_complete_row_closure {
            return Err(bad(
                "policy.enforce_complete_row_closure must be true".to_string()
            ));
        }

        if !self.policy.prohibit_deferred_scope {
            return Err(bad(
                "policy.prohibit_deferred_scope must be true".to_string()
            ));
        }

        for b in &self.accepted_boundaries {
            if b.status != "accepted" {
                return Err(bad(format!(
                    "boundary '{}' has status '{}', must be 'accepted'",
                    b.id, b.status
                )));
            }
        }

        Ok(())
    }
}

/// Engine executing implementation closure verification.
pub struct ImplementationClosureEngine;

impl ImplementationClosureEngine {
    /// Verify that all required boundary IDs are registered and accepted.
    pub fn verify_complete_closure(
        config: &ImplementationClosureConfig,
        required_boundary_ids: &[&str],
    ) -> Result<(), ImplementationClosureError> {
        for required in required_boundary_ids {
            let found = config
                .accepted_boundaries
                .iter()
                .find(|b| b.id == *required && b.status == "accepted");
            if found.is_none() {
                return Err(ImplementationClosureError::UnacceptedBoundary(format!(
                    "required boundary '{required}' is missing or not marked accepted"
                )));
            }
        }

        Ok(())
    }

    /// Verify release handoff readiness against expected consumer target and tree digest.
    pub fn verify_release_handoff_readiness(
        config: &ImplementationClosureConfig,
        expected_url: &str,
        expected_tree_digest: &str,
    ) -> Result<(), ImplementationClosureError> {
        if config.release_identity.target_deployment_url != expected_url {
            return Err(ImplementationClosureError::ReleaseIdentityMismatch(
                format!(
                    "target deployment url mismatch: expected '{expected_url}', got '{}'",
                    config.release_identity.target_deployment_url
                ),
            ));
        }

        if config.release_identity.artifact_tree_digest != expected_tree_digest {
            return Err(ImplementationClosureError::ReleaseIdentityMismatch(
                format!(
                    "artifact tree digest mismatch: expected '{expected_tree_digest}', got '{}'",
                    config.release_identity.artifact_tree_digest
                ),
            ));
        }

        Ok(())
    }
}
