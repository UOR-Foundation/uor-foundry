//! Veilid Bootstrap model: authenticated secure bootstrap routes, browser transport integration,
//! relay public-key verification, outbound-relay limitation mitigations, and network discovery.
//!
//! Conformance ID: `VB-01` (suite: `veilid-bootstrap`).

use serde::{Deserialize, Serialize};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in Veilid bootstrap and transport operations.
#[derive(Debug, Clone, PartialEq)]
pub enum VeilidBootstrapError {
    /// Insecure transport endpoint (non-WSS) rejected under HTTPS-origin policy.
    InsecureEndpointRejected(String),
    /// Unauthenticated bootstrap relay or mismatched public-key signature rejected.
    UnauthenticatedRelayRejected(String),
    /// Veilid 0.5.7 relay selector returned no outbound relay without configured fallback.
    OutboundRelayMissing(String),
    /// Unsupported assumption detected (e.g. feature-flag-only claim without transport evidence).
    UnsupportedAssumption(String),
    /// Origin routing path does not match required HTTPS /foundry-web/ path.
    OriginPathMismatch(String),
    /// Peer connection or dial timeout occurred.
    ConnectionFailed(String),
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for VeilidBootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsecureEndpointRejected(e) => {
                write!(
                    f,
                    "insecure transport endpoint rejected (requires WSS): {e}"
                )
            }
            Self::UnauthenticatedRelayRejected(r) => {
                write!(f, "unauthenticated relay rejected: {r}")
            }
            Self::OutboundRelayMissing(m) => {
                write!(f, "outbound relay missing: {m}")
            }
            Self::UnsupportedAssumption(u) => {
                write!(f, "unsupported assumption rejected: {u}")
            }
            Self::OriginPathMismatch(o) => {
                write!(f, "origin path mismatch: {o}")
            }
            Self::ConnectionFailed(c) => {
                write!(f, "connection failed: {c}")
            }
            Self::Validation(v) => write!(f, "veilid bootstrap validation error: {v}"),
        }
    }
}

impl std::error::Error for VeilidBootstrapError {}

/// Top-level policy governing Veilid bootstrap routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeilidPolicyConfig {
    /// Require authenticated bootstrap route.
    pub require_authenticated_bootstrap_route: bool,
    /// Require HTTPS origin routing.
    pub require_https_origin_routing: bool,
    /// Prohibit feature-flag-only claims without live transport evidence.
    pub prohibit_feature_flag_only_claims: bool,
    /// Prohibit unauthenticated or untrusted relays.
    pub prohibit_unauthenticated_relays: bool,
    /// Require outbound relay limitation mitigation.
    pub require_outbound_relay_mitigation: bool,
    /// Enforce WSS transport encryption.
    pub enforce_wss_transport_encryption: bool,
}

/// A disclosed bootstrap peer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapPeerConfig {
    /// Canonical Veilid peer identifier.
    pub peer_id: String,
    /// Transport endpoint URL (must be wss://).
    pub endpoint: String,
    /// Authenticated Ed25519 public key.
    pub public_key: String,
    /// Cryptographic trust root SHA-256 digest.
    pub trust_root_digest: String,
    /// Role (e.g. primary-bootstrap, secondary-bootstrap).
    pub role: String,
}

/// Routing configuration for browser origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeilidRoutingConfig {
    /// Expected HTTPS origin.
    pub https_origin: String,
    /// Base path (/foundry-web/).
    pub base_path: String,
    /// Dial timeout in seconds.
    pub dial_timeout_seconds: u64,
    /// Maximum reconnection attempts.
    pub max_reconnect_attempts: usize,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval_seconds: u64,
    /// Disclosed public bootstrap peers.
    pub bootstrap_peers: Vec<BootstrapPeerConfig>,
}

/// Relay selector and outbound relay mitigation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelaySelectorConfig {
    /// Minimum reputable relays required in table.
    pub min_reputable_relays: usize,
    /// Pinned protocol version (0.5.7).
    pub protocol_version: String,
    /// Maximum routing table capacity.
    pub routing_table_capacity: usize,
    /// Enable outbound relay fallback when native selector returns none.
    pub enable_outbound_relay_fallback: bool,
    /// Fallback relay endpoint URL.
    pub fallback_relay_endpoint: String,
}

/// Top-level configuration loaded from `model/veilid_bootstrap.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeilidBootstrapConfig {
    /// Spec identifier (`foundry/veilid-bootstrap/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Policy configuration.
    pub policy: VeilidPolicyConfig,
    /// Routing configuration.
    pub routing: VeilidRoutingConfig,
    /// Relay selector configuration.
    pub relay_selector: RelaySelectorConfig,
}

impl VeilidBootstrapConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/veilid-bootstrap/1" {
            return Err(bad(format!(
                "veilid_bootstrap spec must be 'foundry/veilid-bootstrap/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "veilid_bootstrap stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if !self.routing.https_origin.starts_with("https://") {
            return Err(bad("routing.https_origin must be https:// URL".to_string()));
        }

        if self.routing.base_path != "/foundry-web/" {
            return Err(bad("routing.base_path must be '/foundry-web/'".to_string()));
        }

        if self.routing.bootstrap_peers.len() < 2 {
            return Err(bad(
                "routing.bootstrap_peers must include at least 2 independent bootstrap relays"
                    .to_string(),
            ));
        }

        for peer in &self.routing.bootstrap_peers {
            if !peer.endpoint.starts_with("wss://") {
                return Err(bad(format!(
                    "peer endpoint '{}' must use secure wss:// scheme",
                    peer.endpoint
                )));
            }
            if !peer.public_key.starts_with("ed25519:") {
                return Err(bad(format!(
                    "peer '{}' public_key must be ed25519: format",
                    peer.peer_id
                )));
            }
            if !peer.trust_root_digest.starts_with("sha256:") {
                return Err(bad(format!(
                    "peer '{}' trust_root_digest must be sha256: format",
                    peer.peer_id
                )));
            }
        }

        if self.relay_selector.protocol_version != "0.5.7" {
            return Err(bad(format!(
                "relay_selector.protocol_version must be '0.5.7', found '{}'",
                self.relay_selector.protocol_version
            )));
        }

        if !self.relay_selector.enable_outbound_relay_fallback {
            return Err(bad(
                "enable_outbound_relay_fallback must be true to mitigate Veilid 0.5.7 outbound relay limitation".to_string(),
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Veilid Transport Session & Bootstrap Engine
// -----------------------------------------------------------------------------

/// Active transport session established over secure WebSocket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VeilidTransportSession {
    /// Connected peer ID.
    pub connected_peer: String,
    /// Remote WSS endpoint.
    pub endpoint: String,
    /// Verified WSS TLS transport encryption flag.
    pub is_encrypted_wss: bool,
    /// Authenticated peer public key.
    pub authenticated_public_key: String,
    /// Flag indicating whether an outbound relay was successfully allocated.
    pub outbound_relay_allocated: bool,
    /// Outbound relay address if allocated.
    pub outbound_relay_address: Option<String>,
    /// Last successful ping epoch.
    pub last_ping_epoch: u64,
}

/// Engine executing Veilid secure bootstrap verification, relay selection, and session lifecycle.
pub struct VeilidBootstrapEngine;

impl VeilidBootstrapEngine {
    /// Verify that a bootstrap peer complies with security and trust root requirements.
    pub fn verify_bootstrap_peer(peer: &BootstrapPeerConfig) -> Result<(), VeilidBootstrapError> {
        if !peer.endpoint.starts_with("wss://") {
            return Err(VeilidBootstrapError::InsecureEndpointRejected(format!(
                "endpoint '{}' is not secure WSS",
                peer.endpoint
            )));
        }

        if !peer.public_key.starts_with("ed25519:") || peer.public_key.len() < 32 {
            return Err(VeilidBootstrapError::UnauthenticatedRelayRejected(format!(
                "invalid public key format for peer '{}'",
                peer.peer_id
            )));
        }

        if !peer.trust_root_digest.starts_with("sha256:") {
            return Err(VeilidBootstrapError::UnauthenticatedRelayRejected(format!(
                "invalid trust root digest for peer '{}'",
                peer.peer_id
            )));
        }

        Ok(())
    }

    /// Establish a simulated transport connection to an authenticated bootstrap peer.
    pub fn connect_bootstrap(
        peer: &BootstrapPeerConfig,
        origin_url: &str,
    ) -> Result<VeilidTransportSession, VeilidBootstrapError> {
        Self::verify_bootstrap_peer(peer)?;

        if !origin_url.starts_with("https://") {
            return Err(VeilidBootstrapError::OriginPathMismatch(format!(
                "origin URL '{origin_url}' must be HTTPS"
            )));
        }

        if !origin_url.contains("/foundry-web/") {
            return Err(VeilidBootstrapError::OriginPathMismatch(format!(
                "origin URL '{origin_url}' must target /foundry-web/ path"
            )));
        }

        Ok(VeilidTransportSession {
            connected_peer: peer.peer_id.clone(),
            endpoint: peer.endpoint.clone(),
            is_encrypted_wss: true,
            authenticated_public_key: peer.public_key.clone(),
            outbound_relay_allocated: false,
            outbound_relay_address: None,
            last_ping_epoch: 1000,
        })
    }

    /// Allocate an outbound relay, mitigating the Veilid 0.5.7 empty-relay limitation.
    pub fn allocate_outbound_relay(
        session: &mut VeilidTransportSession,
        selector_cfg: &RelaySelectorConfig,
    ) -> Result<String, VeilidBootstrapError> {
        // In Veilid 0.5.7, the native browser relay selector returns no outbound relay
        // when operating from a WebAssembly context without native UDP/TCP sockets.
        // We explicitly detect and mitigate this limitation using the authenticated fallback relay.
        if !selector_cfg.enable_outbound_relay_fallback {
            return Err(VeilidBootstrapError::OutboundRelayMissing(
                "Veilid 0.5.7 relay selector returned no outbound relay and fallback is disabled"
                    .to_string(),
            ));
        }

        let allocated_relay = format!("{}/relay", selector_cfg.fallback_relay_endpoint);
        session.outbound_relay_allocated = true;
        session.outbound_relay_address = Some(allocated_relay.clone());
        Ok(allocated_relay)
    }

    /// Simulate peer churn and automatic reconnection to secondary bootstrap peer.
    pub fn simulate_peer_churn_and_reconnect(
        session: &mut VeilidTransportSession,
        peers: &[BootstrapPeerConfig],
    ) -> Result<(), VeilidBootstrapError> {
        let alternative = peers
            .iter()
            .find(|p| p.peer_id != session.connected_peer)
            .ok_or_else(|| {
                VeilidBootstrapError::ConnectionFailed(
                    "no alternative bootstrap peer available for failover".to_string(),
                )
            })?;

        Self::verify_bootstrap_peer(alternative)?;

        session.connected_peer = alternative.peer_id.clone();
        session.endpoint = alternative.endpoint.clone();
        session.authenticated_public_key = alternative.public_key.clone();
        session.last_ping_epoch += 30; // 30s elapsed during reconnect
        Ok(())
    }

    /// Enforce rejection of unsupported assumptions: e.g. claiming WSS support by feature flag alone
    /// without establishing an actual authenticated transport session.
    pub fn reject_feature_flag_only_claim(
        has_live_session: bool,
        feature_claim: &str,
    ) -> Result<(), VeilidBootstrapError> {
        if !has_live_session {
            return Err(VeilidBootstrapError::UnsupportedAssumption(format!(
                "feature claim '{feature_claim}' rejected: build feature flag alone does not satisfy transport requirement without verified live session"
            )));
        }
        Ok(())
    }
}
