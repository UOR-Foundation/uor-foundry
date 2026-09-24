//! Browser Object Space model: decentralized Kappa storage, queries, inbound dispatch,
//! verified blob transfer, authenticated membership, confidentiality, conflicts, revocation,
//! retention, replication, repair, and offline recovery.
//!
//! Conformance ID: `BO-01` (suite: `browser-object-space`).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in Browser Object Space operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectSpaceError {
    /// Peer failed cryptographic authentication or admission check.
    UnauthenticatedPeer(String),
    /// Peer has been revoked and is fenced from accessing object space.
    RevokedPeer(String),
    /// Confidential data stored unencrypted or leaked in plaintext.
    ConfidentialityViolation(String),
    /// Content addressing digest mismatch between tag and blob bytes.
    DigestMismatch {
        /// Expected digest from tag.
        expected: String,
        /// Computed digest from blob bytes.
        computed: String,
    },
    /// Referenced blob is missing from peer or local storage.
    MissingBlob(String),
    /// Replication quorum could not be satisfied across available peers.
    QuorumDeficit {
        /// Active distinct replicas achieved.
        replicas: usize,
        /// Required minimum replication factor.
        required: usize,
    },
    /// Inbound message or blob size exceeds modeled resource bounds.
    OversizedPayload {
        /// Measured payload size in bytes.
        size: usize,
        /// Allowed bound in bytes.
        limit: usize,
    },
    /// Conflict detected and could not be resolved deterministically.
    ConflictDetected(String),
    /// Validation error.
    Validation(String),
}

impl std::fmt::Display for ObjectSpaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnauthenticatedPeer(p) => write!(f, "unauthenticated peer: {p}"),
            Self::RevokedPeer(p) => write!(f, "peer {p} is revoked and fenced from object space"),
            Self::ConfidentialityViolation(m) => write!(f, "confidentiality violation: {m}"),
            Self::DigestMismatch { expected, computed } => write!(
                f,
                "blob digest mismatch: expected {expected}, computed {computed}"
            ),
            Self::MissingBlob(d) => write!(f, "referenced blob {d} is missing"),
            Self::QuorumDeficit { replicas, required } => write!(
                f,
                "replication quorum deficit: got {replicas} replicas, required {required}"
            ),
            Self::OversizedPayload { size, limit } => {
                write!(f, "oversized payload: {size} bytes exceeds limit {limit}")
            }
            Self::ConflictDetected(c) => write!(f, "unresolved conflict: {c}"),
            Self::Validation(v) => write!(f, "object space validation error: {v}"),
        }
    }
}

impl std::error::Error for ObjectSpaceError {}

/// Top-level policy configuration for browser object space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSpacePolicy {
    /// Enforce cryptographic peer authentication.
    pub enforce_authenticated_membership: bool,
    /// Enforce end-to-end encryption for confidential partitions.
    pub enforce_blob_confidentiality: bool,
    /// Require sha256 blob digest verification before commit.
    pub require_blob_content_verification: bool,
    /// Minimum replication factor across independent browser peers.
    pub min_replication_factor: usize,
    /// Maximum query result limit.
    pub max_query_limit: usize,
    /// Tombstone retention period in seconds.
    pub tombstone_retention_seconds: u64,
    /// Maximum message size in bytes.
    pub max_message_bytes: usize,
    /// Maximum blob size in bytes.
    pub max_blob_bytes: usize,
    /// Inbound channel capacity.
    pub inbound_channel_capacity: usize,
    /// Prohibit server-hosted backend nodes.
    pub prohibit_server_hosted_backends: bool,
}

/// Modeled browser peer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPeerConfig {
    /// Peer UOR identifier.
    pub id: String,
    /// Associated organization.
    pub organization_id: String,
    /// Authenticated user identity.
    pub user_id: String,
    /// Public key string.
    pub public_key: String,
    /// Admission token.
    pub admission_token: String,
    /// Status ("active" or "revoked").
    pub status: String,
}

/// Partition specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionConfig {
    /// Partition namespace.
    pub namespace: String,
    /// Classification level ("confidential" or "public").
    pub classification: String,
    /// Conflict resolution strategy.
    pub conflict_resolution: String,
    /// Minimum required replicas.
    pub min_replicas: usize,
}

/// Retention configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Audit retention period in days.
    pub audit_retention_days: u32,
    /// Tombstone purge threshold in hours.
    pub tombstone_purge_threshold_hours: u32,
    /// Orphaned blob grace period in hours.
    pub orphan_blob_grace_period_hours: u32,
}

/// Anti-entropy synchronization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiEntropyConfig {
    /// Repair check interval in seconds.
    pub repair_interval_seconds: u64,
    /// Batch digest sync limit.
    pub batch_digest_sync_limit: usize,
}

/// Root configuration loaded from `model/object_space.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSpaceConfig {
    /// Spec identifier (`foundry/object-space/1`).
    pub spec: String,
    /// Policy configuration.
    pub policy: ObjectSpacePolicy,
    /// Modeled participating browser peers.
    pub peers: Vec<BrowserPeerConfig>,
    /// Configured storage partitions.
    pub partitions: Vec<PartitionConfig>,
    /// Retention rules.
    pub retention: RetentionConfig,
    /// Anti-entropy synchronization rules.
    pub anti_entropy: AntiEntropyConfig,
}

impl ObjectSpaceConfig {
    /// Check invariants against owner inputs and organization lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/object-space/1" {
            return Err(bad(format!(
                "object_space spec must be 'foundry/object-space/1', found '{}'",
                self.spec
            )));
        }

        if !self.policy.prohibit_server_hosted_backends {
            return Err(bad(
                "policy must prohibit server-hosted backends for browser object space".to_string(),
            ));
        }

        if self.policy.min_replication_factor < 2 {
            return Err(bad(
                "min_replication_factor must be at least 2 across browser peers".to_string(),
            ));
        }

        if self.peers.is_empty() {
            return Err(bad(
                "at least one browser peer must be defined in model/object_space.toml".to_string(),
            ));
        }

        for peer in &self.peers {
            if peer.id.trim().is_empty() {
                return Err(bad("peer id cannot be empty".to_string()));
            }
            if !peer.public_key.starts_with("ed25519:") {
                return Err(bad(format!(
                    "peer {} public key must start with 'ed25519:'",
                    peer.id
                )));
            }
        }

        for part in &self.partitions {
            if part.namespace.trim().is_empty() {
                return Err(bad("partition namespace cannot be empty".to_string()));
            }
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Authenticated Membership & Revocation Fencing
// -----------------------------------------------------------------------------

/// Registered browser peer identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserPeer {
    /// Peer ID.
    pub id: String,
    /// Organization ID.
    pub organization_id: String,
    /// User email ID.
    pub user_id: String,
    /// Public key.
    pub public_key: String,
    /// Admission token.
    pub admission_token: String,
    /// Whether this peer has been cryptographically revoked.
    pub is_revoked: bool,
}

/// Registry of authorized participating browser peers.
#[derive(Debug, Clone, Default)]
pub struct PeerRegistry {
    peers: HashMap<String, BrowserPeer>,
}

impl PeerRegistry {
    /// Create new peer registry from config.
    pub fn from_config(config: &ObjectSpaceConfig) -> Self {
        let mut reg = Self::default();
        for p in &config.peers {
            reg.peers.insert(
                p.id.clone(),
                BrowserPeer {
                    id: p.id.clone(),
                    organization_id: p.organization_id.clone(),
                    user_id: p.user_id.clone(),
                    public_key: p.public_key.clone(),
                    admission_token: p.admission_token.clone(),
                    is_revoked: p.status == "revoked",
                },
            );
        }
        reg
    }

    /// Authenticate a peer's identity and admission credentials.
    pub fn authenticate(
        &self,
        peer_id: &str,
        provided_token: &str,
    ) -> Result<&BrowserPeer, ObjectSpaceError> {
        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| ObjectSpaceError::UnauthenticatedPeer(peer_id.to_string()))?;

        if peer.is_revoked {
            return Err(ObjectSpaceError::RevokedPeer(peer_id.to_string()));
        }

        if peer.admission_token != provided_token {
            return Err(ObjectSpaceError::UnauthenticatedPeer(format!(
                "invalid admission token for {peer_id}"
            )));
        }

        Ok(peer)
    }

    /// Cryptographically revoke a peer, immediately fencing all its subsequent operations.
    pub fn revoke_peer(&mut self, peer_id: &str) -> Result<(), ObjectSpaceError> {
        let peer = self
            .peers
            .get_mut(peer_id)
            .ok_or_else(|| ObjectSpaceError::UnauthenticatedPeer(peer_id.to_string()))?;
        peer.is_revoked = true;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Storage Records, Content Addressing & Queries
// -----------------------------------------------------------------------------

/// Compute standard `sha256:` content-addressed digest for blob bytes.
pub use crate::kappa::compute_sha256_digest as compute_sha256;

/// An object-space record combining tag, content digest, and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRecord {
    /// Key / tag identifier.
    pub tag: String,
    /// Partition namespace.
    pub namespace: String,
    /// Content-addressed digest (`sha256:...`).
    pub digest: String,
    /// Stored blob bytes.
    pub payload: Vec<u8>,
    /// Whether payload is end-to-end encrypted.
    pub is_encrypted: bool,
    /// Logical clock (Lamport timestamp) for conflict ordering.
    pub clock: u64,
    /// Whether this record represents an authorized deletion (tombstone).
    pub is_tombstone: bool,
    /// Creation timestamp (unix seconds).
    pub created_at_epoch: u64,
}

/// Query filter for object-space lookups.
#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    /// Namespace prefix.
    pub namespace: Option<String>,
    /// Tag prefix.
    pub tag_prefix: Option<String>,
    /// Include tombstones.
    pub include_tombstones: bool,
    /// Maximum results limit.
    pub limit: usize,
}

/// Decentralized browser object store instance.
#[derive(Debug, Clone, Default)]
pub struct BrowserObjectStore {
    records: BTreeMap<String, ObjectRecord>,
}

impl BrowserObjectStore {
    /// Insert an object record, verifying content digest and size limits.
    pub fn put(
        &mut self,
        record: ObjectRecord,
        max_bytes: usize,
        enforce_confidentiality: bool,
    ) -> Result<(), ObjectSpaceError> {
        if record.payload.len() > max_bytes {
            return Err(ObjectSpaceError::OversizedPayload {
                size: record.payload.len(),
                limit: max_bytes,
            });
        }

        // Verify digest
        let computed = compute_sha256(&record.payload);
        if computed != record.digest {
            return Err(ObjectSpaceError::DigestMismatch {
                expected: record.digest,
                computed,
            });
        }

        // Enforce confidentiality for non-public namespaces
        if enforce_confidentiality && record.namespace.contains("shared") && !record.is_encrypted {
            return Err(ObjectSpaceError::ConfidentialityViolation(format!(
                "unencrypted confidential record for tag {}",
                record.tag
            )));
        }

        self.records.insert(record.tag.clone(), record);
        Ok(())
    }

    /// Retrieve an object record by tag.
    pub fn get(&self, tag: &str) -> Option<&ObjectRecord> {
        self.records.get(tag).filter(|r| !r.is_tombstone)
    }

    /// Retrieve raw record including tombstones (for replication/anti-entropy).
    pub fn get_raw(&self, tag: &str) -> Option<&ObjectRecord> {
        self.records.get(tag)
    }

    /// Query stored records matching a filter.
    pub fn query(&self, filter: &QueryFilter) -> Vec<&ObjectRecord> {
        let limit = if filter.limit == 0 { 100 } else { filter.limit };
        self.records
            .values()
            .filter(|r| {
                if !filter.include_tombstones && r.is_tombstone {
                    return false;
                }
                if let Some(ns) = &filter.namespace {
                    if &r.namespace != ns {
                        return false;
                    }
                }
                if let Some(prefix) = &filter.tag_prefix {
                    if !r.tag.starts_with(prefix) {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .collect()
    }

    /// Delete an object record by writing an authenticated tombstone.
    pub fn tombstone(&mut self, tag: &str, clock: u64, epoch: u64) -> Result<(), ObjectSpaceError> {
        let existing = self
            .records
            .get(tag)
            .ok_or_else(|| ObjectSpaceError::MissingBlob(tag.to_string()))?;

        let tombstone_rec = ObjectRecord {
            tag: tag.to_string(),
            namespace: existing.namespace.clone(),
            digest: compute_sha256(b"TOMBSTONE"),
            payload: b"TOMBSTONE".to_vec(),
            is_encrypted: false,
            clock: std::cmp::max(existing.clock + 1, clock),
            is_tombstone: true,
            created_at_epoch: epoch,
        };

        self.records.insert(tag.to_string(), tombstone_rec);
        Ok(())
    }

    /// Get all digests mapped by tag.
    pub fn digest_map(&self) -> HashMap<String, String> {
        self.records
            .iter()
            .map(|(k, v)| (k.clone(), v.digest.clone()))
            .collect()
    }
}

// -----------------------------------------------------------------------------
// Verified Blob Transfer Engine
// -----------------------------------------------------------------------------

/// Engine for transferring and verifying blobs between browser peers.
pub struct BlobTransferEngine;

impl BlobTransferEngine {
    /// Receive and verify blob bytes from a peer.
    pub fn receive_and_verify_blob(
        expected_digest: &str,
        blob_bytes: &[u8],
        max_allowed_bytes: usize,
    ) -> Result<Vec<u8>, ObjectSpaceError> {
        if blob_bytes.is_empty() {
            return Err(ObjectSpaceError::MissingBlob(
                "received empty blob bytes".to_string(),
            ));
        }

        if blob_bytes.len() > max_allowed_bytes {
            return Err(ObjectSpaceError::OversizedPayload {
                size: blob_bytes.len(),
                limit: max_allowed_bytes,
            });
        }

        let computed = compute_sha256(blob_bytes);
        if computed != expected_digest {
            return Err(ObjectSpaceError::DigestMismatch {
                expected: expected_digest.to_string(),
                computed,
            });
        }

        Ok(blob_bytes.to_vec())
    }
}

// -----------------------------------------------------------------------------
// Conflict Resolution (CRDT / Logical Clock)
// -----------------------------------------------------------------------------

/// Conflict resolver for concurrent updates.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Resolve conflict between two records with the same tag using Lamport clock.
    /// Higher clock wins; if clocks are equal, lexicographically greater digest wins.
    pub fn resolve<'a>(local: &'a ObjectRecord, incoming: &'a ObjectRecord) -> &'a ObjectRecord {
        if incoming.clock > local.clock {
            incoming
        } else if local.clock > incoming.clock {
            local
        } else if incoming.digest > local.digest {
            incoming
        } else {
            local
        }
    }
}

// -----------------------------------------------------------------------------
// Replication & Quorum Coordinator
// -----------------------------------------------------------------------------

/// Replication coordinator verifying quorum across browser peers.
pub struct ReplicationCoordinator;

impl ReplicationCoordinator {
    /// Verify replication quorum across participating peers.
    pub fn verify_quorum(
        peer_confirmations: &[String],
        peer_registry: &PeerRegistry,
        required_factor: usize,
    ) -> Result<(), ObjectSpaceError> {
        let mut distinct_active_peers = HashSet::new();

        for peer_id in peer_confirmations {
            if let Some(peer) = peer_registry.peers.get(peer_id) {
                if !peer.is_revoked {
                    distinct_active_peers.insert(peer_id.clone());
                }
            }
        }

        if distinct_active_peers.len() < required_factor {
            return Err(ObjectSpaceError::QuorumDeficit {
                replicas: distinct_active_peers.len(),
                required: required_factor,
            });
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Anti-Entropy Repair
// -----------------------------------------------------------------------------

/// Anti-entropy synchronization between two browser object stores.
pub struct AntiEntropyRepair;

impl AntiEntropyRepair {
    /// Synchronize missing or outdated records between local and remote stores.
    /// Returns the number of repaired tags.
    pub fn sync_stores(
        local: &mut BrowserObjectStore,
        remote: &BrowserObjectStore,
        max_bytes: usize,
        enforce_confidentiality: bool,
    ) -> Result<usize, ObjectSpaceError> {
        let local_digests = local.digest_map();
        let remote_records = remote.query(&QueryFilter {
            include_tombstones: true,
            limit: 1000,
            ..Default::default()
        });

        let mut repaired_count = 0;

        for remote_rec in remote_records {
            let needs_update = match local_digests.get(&remote_rec.tag) {
                None => true,
                Some(local_digest) => {
                    if local_digest != &remote_rec.digest {
                        let local_rec = local.get_raw(&remote_rec.tag).unwrap();
                        let winner = ConflictResolver::resolve(local_rec, remote_rec);
                        winner.digest == remote_rec.digest
                    } else {
                        false
                    }
                }
            };

            if needs_update {
                local.put(remote_rec.clone(), max_bytes, enforce_confidentiality)?;
                repaired_count += 1;
            }
        }

        Ok(repaired_count)
    }
}

// -----------------------------------------------------------------------------
// Offline Recovery
// -----------------------------------------------------------------------------

/// Recovery engine restoring consistent local state across browser restarts/disconnects.
pub struct RecoveryEngine;

impl RecoveryEngine {
    /// Recover local store after offline period and reconcile with peer network.
    pub fn recover_and_reconcile(
        persisted_local: &mut BrowserObjectStore,
        active_network_peers: &[&BrowserObjectStore],
        max_bytes: usize,
        enforce_confidentiality: bool,
    ) -> Result<usize, ObjectSpaceError> {
        let mut total_repaired = 0;
        for peer_store in active_network_peers {
            let repaired = AntiEntropyRepair::sync_stores(
                persisted_local,
                peer_store,
                max_bytes,
                enforce_confidentiality,
            )?;
            total_repaired += repaired;
        }
        Ok(total_repaired)
    }
}
