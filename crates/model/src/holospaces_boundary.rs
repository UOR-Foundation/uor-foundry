//! Holospaces boundary model: revalidation of threat-model assumptions against participant/faculty-only
//! session realities, browser-only discovery and private-key possession, rejection of native-relay assumptions
//! and local-witness proxies, and durability under participant session churn.
//!
//! Conformance ID: `HB-01` (suite: `holospaces-boundary`).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in Holospaces boundary operations.
#[derive(Debug, Clone, PartialEq)]
pub enum HolospacesBoundaryError {
    /// Native relay daemon assumed alongside browser tabs, violating browser-only reality.
    NativeRelayAssumptionViolation(String),
    /// Local WebRTC witness or loopback proxy used instead of production discovery evidence.
    LocalWitnessProxyViolation(String),
    /// Out-of-band signaling claimed as proof of automated public discovery.
    OutOfBandSignalingViolation(String),
    /// Private key possession was not proven; public key address alone is insufficient.
    MissingPrivateKeyPossession(String),
    /// All participants are offline; an all-offline participant network cannot execute services.
    AllParticipantsOffline(String),
    /// Active participant replica count is below required durability quorum.
    QuorumDeficit {
        /// Currently active participant replicas.
        active: usize,
        /// Minimum required replicas.
        required: usize,
    },
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for HolospacesBoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NativeRelayAssumptionViolation(n) => {
                write!(f, "native relay assumption violation: {n}")
            }
            Self::LocalWitnessProxyViolation(l) => {
                write!(f, "local witness proxy violation: {l}")
            }
            Self::OutOfBandSignalingViolation(o) => {
                write!(f, "out-of-band signaling violation: {o}")
            }
            Self::MissingPrivateKeyPossession(p) => {
                write!(f, "missing private key possession proof: {p}")
            }
            Self::AllParticipantsOffline(a) => {
                write!(f, "all participants offline: {a}")
            }
            Self::QuorumDeficit { active, required } => {
                write!(
                    f,
                    "quorum deficit: only {active} active replicas, required {required}"
                )
            }
            Self::Validation(v) => write!(f, "holospaces boundary validation error: {v}"),
        }
    }
}

impl std::error::Error for HolospacesBoundaryError {}

/// Threat model rules revalidating Holospaces assumptions for participant-only sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolospacesThreatModelConfig {
    /// Prohibit assuming native relay peers alongside browser tabs.
    pub prohibit_native_relay_assumptions: bool,
    /// Prohibit local WebRTC witness proxies from passing as production discovery evidence.
    pub prohibit_local_witness_proxies: bool,
    /// Prohibit out-of-band signaling from passing as automated discovery evidence.
    pub prohibit_out_of_band_signaling: bool,
    /// Require cryptographic proof of private key possession (not just address/public-key).
    pub require_private_key_possession: bool,
    /// Require modeling participant/faculty-only session churn.
    pub require_participant_session_churn_model: bool,
    /// Prohibit claims of service execution when all participants are offline.
    pub prohibit_all_offline_execution: bool,
}

/// Durability and replica retention policy under session churn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolospacesDurabilityConfig {
    /// Minimum participant replicas required for durability.
    pub min_participant_replicas: usize,
    /// Maximum acceptable offline duration for a participant in hours.
    pub max_acceptable_offline_hours: u64,
    /// Anti-entropy repair frequency in seconds.
    pub anti_entropy_repair_frequency_seconds: u64,
    /// Required quorum percentage for writes (e.g. 60.0).
    pub required_quorum_percentage: f64,
}

/// Discovery and WebRTC signaling policy for browser participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolospacesDiscoveryConfig {
    /// Primary discovery mechanism (veilid-authenticated-relay).
    pub primary_discovery: String,
    /// WebRTC signaling mode (authenticated-in-band).
    pub webrtc_signaling: String,
    /// Prohibit manual or out-of-band rendezvous.
    pub prohibit_out_of_band_rendezvous: bool,
}

/// Top-level Holospaces boundary configuration loaded from `model/holospaces_boundary.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolospacesBoundaryConfig {
    /// Spec identifier (`foundry/holospaces-boundary/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Pinned upstream Holospaces commit hash (`96769f16be454ab1572fddff4613704ccfbebf5e`).
    pub pinned_upstream_commit: String,
    /// Threat model configuration.
    pub threat_model: HolospacesThreatModelConfig,
    /// Durability configuration.
    pub durability: HolospacesDurabilityConfig,
    /// Discovery configuration.
    pub discovery: HolospacesDiscoveryConfig,
}

impl HolospacesBoundaryConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/holospaces-boundary/1" {
            return Err(bad(format!(
                "holospaces_boundary spec must be 'foundry/holospaces-boundary/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "holospaces_boundary stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if self.pinned_upstream_commit != "96769f16be454ab1572fddff4613704ccfbebf5e" {
            return Err(bad(format!(
                "pinned_upstream_commit must match audited commit '96769f16be454ab1572fddff4613704ccfbebf5e', found '{}'",
                self.pinned_upstream_commit
            )));
        }

        if !self.threat_model.prohibit_native_relay_assumptions {
            return Err(bad(
                "prohibit_native_relay_assumptions must be true for browser-only operation"
                    .to_string(),
            ));
        }

        if !self.threat_model.require_private_key_possession {
            return Err(bad(
                "require_private_key_possession must be true to prevent public-key-only impersonation"
                    .to_string(),
            ));
        }

        if self.durability.min_participant_replicas < 3 {
            return Err(bad(
                "durability.min_participant_replicas must be at least 3 for fault tolerance"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Holospaces Mesh Coordinator & Session Realities
// -----------------------------------------------------------------------------

/// A participant or faculty session executing in a browser tab.
#[derive(Debug, Clone)]
pub struct ParticipantSessionRecord {
    /// Unique session identifier.
    pub session_id: String,
    /// Participant category (`student`, `faculty`, `administrator`).
    pub participant_role: String,
    /// Indicates whether session executes in a browser tab.
    pub is_browser_tab: bool,
    /// Indicates whether a co-located native daemon is assumed.
    pub assumes_native_daemon: bool,
    /// Indicates whether private key possession was cryptographically proven.
    pub proven_private_key_possession: bool,
    /// Online status.
    pub is_online: bool,
    /// Object keys held by this participant.
    pub held_object_keys: HashSet<String>,
}

/// Coordinator enforcing revalidated threat-model boundaries across participant sessions.
pub struct HolospacesMeshCoordinator;

impl HolospacesMeshCoordinator {
    /// Validate participant session invariants under browser-only threat model.
    pub fn validate_participant_session(
        session: &ParticipantSessionRecord,
        config: &HolospacesBoundaryConfig,
    ) -> Result<(), HolospacesBoundaryError> {
        if config.threat_model.prohibit_native_relay_assumptions && session.assumes_native_daemon {
            return Err(HolospacesBoundaryError::NativeRelayAssumptionViolation(
                format!(
                    "session '{}' assumes co-located native daemon; invalid in browser-only operation",
                    session.session_id
                ),
            ));
        }

        if config.threat_model.require_private_key_possession
            && !session.proven_private_key_possession
        {
            return Err(HolospacesBoundaryError::MissingPrivateKeyPossession(format!(
                "session '{}' failed to prove private key possession; public key address alone is insufficient",
                session.session_id
            )));
        }

        Ok(())
    }

    /// Evaluate mesh durability and availability under participant churn.
    pub fn evaluate_mesh_durability(
        sessions: &[ParticipantSessionRecord],
        object_key: &str,
        config: &HolospacesBoundaryConfig,
    ) -> Result<usize, HolospacesBoundaryError> {
        let any_online = sessions.iter().any(|s| s.is_online);
        if !any_online && config.threat_model.prohibit_all_offline_execution {
            return Err(HolospacesBoundaryError::AllParticipantsOffline(
                "all participant sessions are offline; an all-offline participant network cannot execute services"
                    .to_string(),
            ));
        }

        let active_replicas = sessions
            .iter()
            .filter(|s| s.is_online && s.held_object_keys.contains(object_key))
            .count();

        if active_replicas < config.durability.min_participant_replicas {
            return Err(HolospacesBoundaryError::QuorumDeficit {
                active: active_replicas,
                required: config.durability.min_participant_replicas,
            });
        }

        Ok(active_replicas)
    }

    /// Evaluate discovery claim validity against production criteria.
    pub fn evaluate_discovery_claim(
        discovery_mechanism: &str,
        uses_out_of_band_signaling: bool,
        is_local_witness_proxy: bool,
        config: &HolospacesBoundaryConfig,
    ) -> Result<(), HolospacesBoundaryError> {
        if config.threat_model.prohibit_local_witness_proxies && is_local_witness_proxy {
            return Err(HolospacesBoundaryError::LocalWitnessProxyViolation(
                "local WebRTC witness tests do not establish production discovery or availability"
                    .to_string(),
            ));
        }

        if config.threat_model.prohibit_out_of_band_signaling && uses_out_of_band_signaling {
            return Err(HolospacesBoundaryError::OutOfBandSignalingViolation(
                "out-of-band signaling cannot substitute for automated production in-band discovery"
                    .to_string(),
            ));
        }

        if discovery_mechanism != config.discovery.primary_discovery {
            return Err(HolospacesBoundaryError::Validation(format!(
                "discovery mechanism '{discovery_mechanism}' does not match required primary discovery '{}'",
                config.discovery.primary_discovery
            )));
        }

        Ok(())
    }
}
