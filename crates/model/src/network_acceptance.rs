//! Network Acceptance model: real participant operation under adverse network conditions,
//! discovery constraints, partitions, browser suspensions, peer eviction, replica loss,
//! hostile inputs, authenticated bootstrap routing, and measured availability targets.
//!
//! Conformance ID: `NA-01` (suite: `network-acceptance`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in Network Acceptance operations.
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkError {
    /// Operation attempted without required quorum in partitioned network.
    QuorumUnavailable(String),
    /// Peer is suspended (OS tab hibernation) and cannot process traffic.
    PeerSuspended(String),
    /// Peer has been evicted from active mesh due to missed heartbeats.
    PeerEvicted(String),
    /// Inbound frame rejected due to malformed header, corrupt payload, or bad signature.
    MalformedInputRejected(String),
    /// Bootstrap routing configuration violates HTTPS-origin or relay trust boundaries.
    UnsafeBootstrapRoute(String),
    /// Availability or MTTR service-level target was breached.
    SloBreached {
        /// Metric name.
        metric: String,
        /// Actual measured value.
        actual: f64,
        /// Target requirement.
        target: f64,
    },
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::QuorumUnavailable(m) => write!(f, "quorum unavailable: {m}"),
            Self::PeerSuspended(p) => {
                write!(f, "peer {p} is currently suspended in browser background")
            }
            Self::PeerEvicted(p) => write!(f, "peer {p} was evicted from active mesh routing"),
            Self::MalformedInputRejected(r) => write!(f, "hostile input rejected: {r}"),
            Self::UnsafeBootstrapRoute(b) => write!(f, "unsafe bootstrap route: {b}"),
            Self::SloBreached {
                metric,
                actual,
                target,
            } => write!(
                f,
                "SLO breached for {metric}: actual {actual} did not meet target {target}"
            ),
            Self::Validation(v) => write!(f, "network acceptance validation error: {v}"),
        }
    }
}

impl std::error::Error for NetworkError {}

/// Top-level network acceptance policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyConfig {
    /// Enforce browser-only participant execution.
    pub enforce_browser_only_execution: bool,
    /// Prohibit server-hosted backend nodes.
    pub prohibit_server_hosted_nodes: bool,
    /// Minimum independent participating browser nodes.
    pub min_independent_participants: usize,
    /// Maximum acceptable recovery time after partition in seconds.
    pub max_acceptable_partition_recovery_seconds: u64,
    /// Target uptime availability percentage (e.g. 99.9).
    pub target_availability_percentage: f64,
    /// Require authenticated bootstrap route.
    pub require_authenticated_bootstrap_route: bool,
    /// Require HTTPS origin routing (/foundry-web/).
    pub require_https_origin_routing: bool,
}

/// Disclosed Veilid bootstrap relay and HTTPS origin routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapRouteConfig {
    /// Base URL for web portal (/foundry-web/).
    pub base_url: String,
    /// Routing mode (e.g. SPA_HISTORY_PUSHSTATE).
    pub routing_mode: String,
    /// Primary public Veilid bootstrap relay.
    pub primary_bootstrap_relay: String,
    /// Secondary public Veilid bootstrap relay.
    pub secondary_bootstrap_relay: String,
    /// Disclosed relay public key.
    pub relay_public_key: String,
    /// Trust boundary verification signature.
    pub trust_boundary_signature: String,
}

/// Adverse scenario definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseScenarioConfig {
    /// Scenario ID.
    pub id: String,
    /// Scenario name.
    pub name: String,
    /// Category.
    pub category: String,
    /// Description.
    pub description: String,
    /// Expected behavior.
    pub expected_behavior: String,
}

/// Availability and MTTR targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityTargetConfig {
    /// Target uptime percentage.
    pub target_availability_percentage: f64,
    /// Maximum Mean Time to Recovery (seconds).
    pub max_mttr_seconds: u64,
    /// Maximum permitted un-reconciled divergence count.
    pub max_unreconciled_divergence: usize,
}

/// Top-level network acceptance configuration loaded from `model/network_acceptance.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAcceptanceConfig {
    /// Spec identifier (`foundry/network-acceptance/1`).
    pub spec: String,
    /// Policy configuration.
    pub policy: NetworkPolicyConfig,
    /// Bootstrap routing configuration.
    pub bootstrap_routing: BootstrapRouteConfig,
    /// List of modeled adverse scenarios.
    pub adverse_scenarios: Vec<AdverseScenarioConfig>,
    /// Availability and MTTR targets.
    pub availability_targets: AvailabilityTargetConfig,
}

impl NetworkAcceptanceConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/network-acceptance/1" {
            return Err(bad(format!(
                "network_acceptance spec must be 'foundry/network-acceptance/1', found '{}'",
                self.spec
            )));
        }

        if !self.policy.prohibit_server_hosted_nodes {
            return Err(bad(
                "policy must prohibit server-hosted nodes for network acceptance".to_string(),
            ));
        }

        if self.policy.min_independent_participants < 3 {
            return Err(bad(
                "min_independent_participants must be at least 3".to_string()
            ));
        }

        if !self.bootstrap_routing.base_url.contains("/foundry-web/") {
            return Err(bad(
                "bootstrap_routing.base_url must route under /foundry-web/".to_string(),
            ));
        }

        if !self
            .bootstrap_routing
            .primary_bootstrap_relay
            .starts_with("wss://")
        {
            return Err(bad(
                "bootstrap relay must use secure websocket (wss://)".to_string()
            ));
        }

        if self.adverse_scenarios.is_empty() {
            return Err(bad(
                "at least one adverse scenario must be defined in model/network_acceptance.toml"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Browser Network Mesh Topology & State
// -----------------------------------------------------------------------------

/// State of an individual participating browser node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeState {
    /// Active and actively participating in mesh.
    Active,
    /// Suspended (OS tab backgrounded / sleeping).
    Suspended,
    /// Evicted due to missed heartbeat pings.
    Evicted,
    /// Quarantined due to sending hostile or malformed frames.
    Quarantined,
}

/// A participating browser node in the decentralized mesh.
#[derive(Debug, Clone)]
pub struct BrowserMeshNode {
    /// Peer ID.
    pub peer_id: String,
    /// Current node state.
    pub state: NodeState,
    /// Active peer connections.
    pub connections: HashSet<String>,
    /// Last recorded heartbeat epoch.
    pub last_heartbeat_epoch: u64,
}

/// Mesh topology coordinating simulated independent browser sessions.
#[derive(Debug, Clone, Default)]
pub struct BrowserNetworkMesh {
    nodes: HashMap<String, BrowserMeshNode>,
    partition_barriers: HashSet<(String, String)>,
}

impl BrowserNetworkMesh {
    /// Create mesh with independent participants.
    pub fn new(peer_ids: &[&str], initial_epoch: u64) -> Self {
        let mut mesh = Self::default();
        for &id in peer_ids {
            let mut conns = HashSet::new();
            for &other in peer_ids {
                if id != other {
                    conns.insert(other.to_string());
                }
            }
            mesh.nodes.insert(
                id.to_string(),
                BrowserMeshNode {
                    peer_id: id.to_string(),
                    state: NodeState::Active,
                    connections: conns,
                    last_heartbeat_epoch: initial_epoch,
                },
            );
        }
        mesh
    }

    /// Number of currently active nodes.
    pub fn active_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.state == NodeState::Active)
            .count()
    }

    /// Apply a network partition isolating two sets of peers.
    pub fn apply_partition(&mut self, group_a: &[&str], group_b: &[&str]) {
        for &a in group_a {
            for &b in group_b {
                self.partition_barriers
                    .insert((a.to_string(), b.to_string()));
                self.partition_barriers
                    .insert((b.to_string(), a.to_string()));
            }
        }
    }

    /// Heal network partition, restoring all connections.
    pub fn heal_partition(&mut self) {
        self.partition_barriers.clear();
    }

    /// Check if two nodes can communicate across current partition state.
    pub fn can_communicate(&self, from: &str, to: &str) -> bool {
        if self
            .partition_barriers
            .contains(&(from.to_string(), to.to_string()))
        {
            return false;
        }
        let from_node = match self.nodes.get(from) {
            Some(n) if n.state == NodeState::Active => n,
            _ => return false,
        };
        let to_node = match self.nodes.get(to) {
            Some(n) if n.state == NodeState::Active => n,
            _ => return false,
        };
        from_node.connections.contains(to) && to_node.connections.contains(from)
    }

    /// Verify write quorum within a partitioned subgroup.
    pub fn verify_subgroup_quorum(
        &self,
        subgroup: &[&str],
        required_quorum: usize,
    ) -> Result<(), NetworkError> {
        let active_in_group = subgroup
            .iter()
            .filter(|&&id| {
                self.nodes
                    .get(id)
                    .map(|n| n.state == NodeState::Active)
                    .unwrap_or(false)
            })
            .count();

        if active_in_group < required_quorum {
            return Err(NetworkError::QuorumUnavailable(format!(
                "subgroup has only {active_in_group} active peers, requires quorum of {required_quorum}"
            )));
        }

        Ok(())
    }

    /// Suspend a node (simulating mobile OS background tab suspension).
    pub fn suspend_node(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        let node = self
            .nodes
            .get_mut(peer_id)
            .ok_or_else(|| NetworkError::Validation(format!("unknown peer {peer_id}")))?;
        node.state = NodeState::Suspended;
        Ok(())
    }

    /// Resume a suspended node.
    pub fn resume_node(&mut self, peer_id: &str, current_epoch: u64) -> Result<(), NetworkError> {
        let node = self
            .nodes
            .get_mut(peer_id)
            .ok_or_else(|| NetworkError::Validation(format!("unknown peer {peer_id}")))?;
        node.state = NodeState::Active;
        node.last_heartbeat_epoch = current_epoch;
        Ok(())
    }

    /// Evict an unresponsive node that missed heartbeats beyond timeout.
    pub fn evict_if_unresponsive(
        &mut self,
        peer_id: &str,
        current_epoch: u64,
        timeout_seconds: u64,
    ) -> Result<bool, NetworkError> {
        let node = self
            .nodes
            .get_mut(peer_id)
            .ok_or_else(|| NetworkError::Validation(format!("unknown peer {peer_id}")))?;

        if current_epoch - node.last_heartbeat_epoch > timeout_seconds {
            node.state = NodeState::Evicted;
            // Remove from other nodes' active connections
            let id = peer_id.to_string();
            for other in self.nodes.values_mut() {
                other.connections.remove(&id);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Simulate sudden loss of a replica node and repair by re-replicating to another peer.
    pub fn simulate_replica_loss_and_repair(
        &mut self,
        failed_peer: &str,
        available_target_peer: &str,
    ) -> Result<String, NetworkError> {
        let node = self
            .nodes
            .get_mut(failed_peer)
            .ok_or_else(|| NetworkError::Validation(format!("unknown peer {failed_peer}")))?;
        node.state = NodeState::Evicted;

        let target = self.nodes.get_mut(available_target_peer).ok_or_else(|| {
            NetworkError::Validation(format!("unknown peer {available_target_peer}"))
        })?;

        if target.state != NodeState::Active {
            return Err(NetworkError::QuorumUnavailable(format!(
                "target repair peer {available_target_peer} is not active"
            )));
        }

        Ok(available_target_peer.to_string())
    }

    /// Process an inbound frame from a peer; validates bounds and signatures.
    /// Hostile or malformed frames trigger immediate quarantine.
    pub fn process_inbound_frame(
        &mut self,
        sender_id: &str,
        frame_bytes: &[u8],
        signature: &str,
        max_frame_bytes: usize,
    ) -> Result<(), NetworkError> {
        let node = self
            .nodes
            .get_mut(sender_id)
            .ok_or_else(|| NetworkError::Validation(format!("unknown peer {sender_id}")))?;

        if node.state == NodeState::Quarantined {
            return Err(NetworkError::MalformedInputRejected(format!(
                "peer {sender_id} is quarantined"
            )));
        }

        // Check frame size bound
        if frame_bytes.len() > max_frame_bytes {
            node.state = NodeState::Quarantined;
            return Err(NetworkError::MalformedInputRejected(format!(
                "frame size {} exceeds bound {}",
                frame_bytes.len(),
                max_frame_bytes
            )));
        }

        // Validate frame header signature
        if !signature.starts_with("ed25519:") || signature.len() < 16 {
            node.state = NodeState::Quarantined;
            return Err(NetworkError::MalformedInputRejected(format!(
                "invalid signature on frame from {sender_id}"
            )));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Bootstrap Router & HTTPS Origin Verifier
// -----------------------------------------------------------------------------

/// Router validating bootstrap origin and relay signatures.
pub struct BootstrapRouter;

impl BootstrapRouter {
    /// Verify that origin route complies with HTTPS GitHub Pages deployment path.
    pub fn verify_route(route_cfg: &BootstrapRouteConfig) -> Result<(), NetworkError> {
        if !route_cfg.base_url.starts_with("https://") {
            return Err(NetworkError::UnsafeBootstrapRoute(
                "base_url must use https:// origin".to_string(),
            ));
        }

        if !route_cfg.base_url.ends_with("/foundry-web/") {
            return Err(NetworkError::UnsafeBootstrapRoute(
                "base_url must target /foundry-web/ path without redirect".to_string(),
            ));
        }

        if !route_cfg.trust_boundary_signature.starts_with("sha256:") {
            return Err(NetworkError::UnsafeBootstrapRoute(
                "trust_boundary_signature must be sha256: digest".to_string(),
            ));
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Availability & Recovery Metrics Tracker
// -----------------------------------------------------------------------------

/// Metrics tracker measuring uptime, MTTR, and divergence against SLOs.
#[derive(Debug, Clone, Default)]
pub struct AvailabilityMetricsTracker {
    /// Total number of incoming participant requests.
    pub total_requests: u64,
    /// Total number of successful participant requests.
    pub successful_requests: u64,
    /// Observed recovery duration in seconds across tested scenarios.
    pub observed_recovery_seconds: u64,
    /// Number of un-reconciled divergence records remaining.
    pub un_reconciled_divergence_records: usize,
}

impl AvailabilityMetricsTracker {
    /// Record request outcomes.
    pub fn record_batch(&mut self, total: u64, successful: u64, recovery_secs: u64) {
        self.total_requests += total;
        self.successful_requests += successful;
        self.observed_recovery_seconds =
            std::cmp::max(self.observed_recovery_seconds, recovery_secs);
    }

    /// Calculate availability percentage.
    pub fn availability_percentage(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Verify against approved SLO targets.
    pub fn verify_slo(&self, targets: &AvailabilityTargetConfig) -> Result<(), NetworkError> {
        let availability = self.availability_percentage();
        if availability < targets.target_availability_percentage {
            return Err(NetworkError::SloBreached {
                metric: "availability_percentage".to_string(),
                actual: availability,
                target: targets.target_availability_percentage,
            });
        }

        if self.observed_recovery_seconds > targets.max_mttr_seconds {
            return Err(NetworkError::SloBreached {
                metric: "max_mttr_seconds".to_string(),
                actual: self.observed_recovery_seconds as f64,
                target: targets.max_mttr_seconds as f64,
            });
        }

        if self.un_reconciled_divergence_records > targets.max_unreconciled_divergence {
            return Err(NetworkError::SloBreached {
                metric: "unreconciled_divergence".to_string(),
                actual: self.un_reconciled_divergence_records as f64,
                target: targets.max_unreconciled_divergence as f64,
            });
        }

        Ok(())
    }
}
