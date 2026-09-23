//! Kappa browser object-space service and verified reconciliation boundary.
//!
//! Replaces the upstream Kappa baseline gap (`2af86560`), where Veilid startup
//! discarded the inbound receiver channel (`crates/kappa-server/src/main.rs:768`)
//! and reconciliation copied tags/digests without fetching or persisting the
//! underlying referenced blob bytes (`crates/kappa-transport-veilid/src/reconcile.rs:175-188`).
//!
//! Provides the normative browser object-space model, inbound receiver retention,
//! cryptographic content verification, and fault-rejection rules.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, SyncSender};

use crate::owner_inputs::OwnerInputs;
use crate::ModelError;

/// Declarative specification for the Kappa browser-service boundary and reconciliation.
#[derive(Debug, Clone, Deserialize)]
pub struct KappaConfig {
    /// Service configuration parameters.
    pub service: ServiceConfig,
    /// Reconciliation and integrity parameters.
    pub reconciliation: ReconciliationConfig,
    /// Supported object-space protocols.
    pub protocols: ProtocolsConfig,
    /// Fault and negative testing rules.
    pub fault_rules: FaultRulesConfig,
}

/// Service-level parameters for the browser object-space.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    /// Default namespace for object-space storage.
    pub namespace: String,
    /// Capacity of the inbound receiver channel buffer.
    pub inbound_channel_capacity: usize,
    /// Maximum allowed size for an inbound message in bytes.
    pub max_message_bytes: usize,
    /// Maximum allowed size for an object blob in bytes.
    pub max_blob_bytes: usize,
    /// Requirement that inbound receiver channel is retained and serviced.
    pub require_inbound_servicing: bool,
}

/// Reconciliation and blob integrity parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct ReconciliationConfig {
    /// Whether blob content must be verified against its digest before commit.
    pub require_blob_content_verification: bool,
    /// Normative hash algorithm for content addressing.
    pub hash_algorithm: String,
    /// Whether partial or unverified reconciliation must be rejected.
    pub reject_partial_reconciliation: bool,
    /// Whether tag updates must occur atomically only after blob persistence.
    pub atomic_tag_update: bool,
}

/// Protocols supported by the browser object-space service.
#[derive(Debug, Clone, Deserialize)]
pub struct ProtocolsConfig {
    /// List of supported protocol identifiers.
    pub supported: Vec<String>,
}

/// Rules for rejecting faulty, partial, or malformed reconciliation.
#[derive(Debug, Clone, Deserialize)]
pub struct FaultRulesConfig {
    /// Reject startup or operation where the inbound receiver is dropped.
    pub reject_dropped_receiver: bool,
    /// Reject reconciliation when a referenced blob is missing from peer.
    pub reject_missing_blob: bool,
    /// Reject reconciliation when blob content does not match the tag digest.
    pub reject_corrupted_blob_digest: bool,
    /// Reject reconciliation when received blob is truncated.
    pub reject_truncated_blob: bool,
    /// Reject payloads exceeding maximum workload bounds.
    pub reject_oversized_payload: bool,
}

impl KappaConfig {
    /// Cross-check configuration invariants against owner-controlled bounds (OI-01).
    pub fn check(&self, owner_inputs: &OwnerInputs) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);

        if self.service.namespace.trim().is_empty() {
            return Err(bad("kappa service namespace must not be empty".to_string()));
        }

        if self.service.inbound_channel_capacity == 0 {
            return Err(bad(
                "kappa inbound_channel_capacity must be greater than zero".to_string(),
            ));
        }

        if !self.service.require_inbound_servicing {
            return Err(bad(
                "kappa require_inbound_servicing must be true: inbound receiver cannot be dropped"
                    .to_string(),
            ));
        }

        if self.service.max_message_bytes as u64 > owner_inputs.resilience_bounds.max_message_bytes
        {
            return Err(bad(format!(
                "kappa max_message_bytes ({}) exceeds owner workload bounds ({})",
                self.service.max_message_bytes, owner_inputs.resilience_bounds.max_message_bytes
            )));
        }

        if self.service.max_blob_bytes as u64 > owner_inputs.resilience_bounds.max_blob_bytes {
            return Err(bad(format!(
                "kappa max_blob_bytes ({}) exceeds owner workload bounds ({})",
                self.service.max_blob_bytes, owner_inputs.resilience_bounds.max_blob_bytes
            )));
        }

        if !self.reconciliation.require_blob_content_verification {
            return Err(bad(
                "kappa require_blob_content_verification must be true: blobs must be verified"
                    .to_string(),
            ));
        }

        if self.reconciliation.hash_algorithm != "sha256" {
            return Err(bad(format!(
                "kappa hash_algorithm '{}' unsupported; must be 'sha256'",
                self.reconciliation.hash_algorithm
            )));
        }

        if !self.reconciliation.reject_partial_reconciliation {
            return Err(bad(
                "kappa reject_partial_reconciliation must be true: partial reconciliation forbidden"
                    .to_string(),
            ));
        }

        if !self.reconciliation.atomic_tag_update {
            return Err(bad(
                "kappa atomic_tag_update must be true: tags committed only after blob storage"
                    .to_string(),
            ));
        }

        if !self.fault_rules.reject_dropped_receiver
            || !self.fault_rules.reject_missing_blob
            || !self.fault_rules.reject_corrupted_blob_digest
            || !self.fault_rules.reject_truncated_blob
            || !self.fault_rules.reject_oversized_payload
        {
            return Err(bad(
                "all kappa fault_rules must be enabled to enforce integrity boundaries".to_string(),
            ));
        }

        Ok(())
    }
}

/// Normative error type for Kappa browser object-space service and reconciliation.
///
/// Sanctioned by R5 in `xtask/src/audit.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KappaError {
    /// Inbound transport receiver was dropped, discarded, or closed prematurely.
    ReceiverDropped,
    /// Inbound receiver queue has exceeded capacity.
    ReceiverBufferFull,
    /// Referenced blob was not found for the specified content digest.
    BlobNotFound(String),
    /// Content hash verification failed: expected digest does not match actual bytes.
    DigestMismatch {
        /// Expected content digest from tag.
        expected: String,
        /// Actual content digest computed from bytes.
        actual: String,
    },
    /// Blob was truncated or incomplete.
    BlobTruncated {
        /// Expected byte length.
        expected_len: usize,
        /// Actual received byte length.
        actual_len: usize,
    },
    /// Inbound message or blob payload exceeds maximum allowed size.
    PayloadTooLarge {
        /// Configured bound.
        max_bytes: usize,
        /// Observed payload size.
        actual_bytes: usize,
    },
    /// Requested namespace does not exist.
    NamespaceNotFound(String),
    /// Requested tag does not exist.
    TagNotFound(String),
    /// Tag update was rejected because referenced blob bytes are missing or unverified.
    UnverifiedState(String),
    /// Hash algorithm is not supported.
    UnsupportedHashAlgorithm(String),
}

impl std::fmt::Display for KappaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReceiverDropped => {
                write!(f, "inbound receiver channel was dropped or disconnected")
            }
            Self::ReceiverBufferFull => write!(f, "inbound receiver queue buffer is full"),
            Self::BlobNotFound(d) => write!(f, "referenced blob not found for digest: {d}"),
            Self::DigestMismatch { expected, actual } => write!(
                f,
                "cryptographic content mismatch: expected digest '{expected}', computed '{actual}'"
            ),
            Self::BlobTruncated {
                expected_len,
                actual_len,
            } => write!(
                f,
                "blob payload truncated: expected {expected_len} bytes, received {actual_len}"
            ),
            Self::PayloadTooLarge {
                max_bytes,
                actual_bytes,
            } => write!(
                f,
                "payload size {actual_bytes} bytes exceeds maximum allowed limit {max_bytes}"
            ),
            Self::NamespaceNotFound(ns) => write!(f, "namespace '{ns}' not found"),
            Self::TagNotFound(tag) => write!(f, "tag '{tag}' not found"),
            Self::UnverifiedState(msg) => write!(f, "unverified state rejected: {msg}"),
            Self::UnsupportedHashAlgorithm(algo) => {
                write!(f, "unsupported content hash algorithm: {algo}")
            }
        }
    }
}

impl std::error::Error for KappaError {}

/// Compute the canonical SHA-256 content digest for arbitrary bytes.
///
/// Formats the digest with the standard prefix `sha256:<hex>`.
pub fn compute_sha256_digest(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in result {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    format!("sha256:{hex}")
}

/// An immutable, content-addressed blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blob {
    /// Canonical content digest (e.g. `sha256:<hex>`).
    pub digest: String,
    /// Raw payload bytes.
    pub data: Vec<u8>,
}

impl Blob {
    /// Create a new blob, computing its canonical digest.
    pub fn new(data: Vec<u8>) -> Self {
        let digest = compute_sha256_digest(&data);
        Self { digest, data }
    }

    /// Verify that this blob's data matches its declared digest.
    pub fn verify(&self) -> Result<(), KappaError> {
        let actual = compute_sha256_digest(&self.data);
        if actual == self.digest {
            Ok(())
        } else {
            Err(KappaError::DigestMismatch {
                expected: self.digest.clone(),
                actual,
            })
        }
    }
}

/// A named tag reference pointing to a content digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Tag name within the namespace (e.g. `main`, `v1.0.0`).
    pub name: String,
    /// Content digest referenced by this tag.
    pub kappa: String,
}

/// In-memory object store for browser-service execution and testing.
#[derive(Debug, Default, Clone)]
pub struct InMemoryObjectStore {
    namespaces: BTreeMap<String, BTreeMap<String, String>>,
    blobs: BTreeMap<String, Blob>,
}

impl InMemoryObjectStore {
    /// Create an empty object store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a content-verified blob.
    ///
    /// Refuses to store corrupted blobs where digest does not match data.
    pub fn put_blob(&mut self, blob: Blob) -> Result<(), KappaError> {
        blob.verify()?;
        self.blobs.insert(blob.digest.clone(), blob);
        Ok(())
    }

    /// Retrieve a blob by its content digest.
    pub fn get_blob(&self, digest: &str) -> Option<&Blob> {
        self.blobs.get(digest)
    }

    /// Check if a blob exists in the store.
    pub fn has_blob(&self, digest: &str) -> bool {
        self.blobs.contains_key(digest)
    }

    /// Set a tag pointing to a content digest.
    ///
    /// Atomic guarantee: tag can ONLY be set if the referenced blob is already
    /// present and verified in the blob store.
    pub fn set_tag(&mut self, ns: &str, tag_name: &str, digest: &str) -> Result<(), KappaError> {
        if !self.has_blob(digest) {
            return Err(KappaError::UnverifiedState(format!(
                "refusing to set tag '{tag_name}' to digest '{digest}': referenced blob bytes are missing"
            )));
        }
        let tags = self.namespaces.entry(ns.to_string()).or_default();
        tags.insert(tag_name.to_string(), digest.to_string());
        Ok(())
    }

    /// Retrieve the digest for a named tag in a namespace.
    pub fn get_tag(&self, ns: &str, tag_name: &str) -> Option<String> {
        self.namespaces
            .get(ns)
            .and_then(|tags| tags.get(tag_name).cloned())
    }

    /// List all tags in a namespace.
    pub fn list_tags(&self, ns: &str) -> Vec<Tag> {
        match self.namespaces.get(ns) {
            Some(tags) => tags
                .iter()
                .map(|(name, kappa)| Tag {
                    name: name.clone(),
                    kappa: kappa.clone(),
                })
                .collect(),
            None => Vec::new(),
        }
    }
}

/// An inbound message received from a transport peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundMessage {
    /// Identity of the sending peer.
    pub peer_id: String,
    /// Protocol message operation (e.g. `diff_pull`, `blob_pull`).
    pub op: String,
    /// Message payload bytes.
    pub payload: Vec<u8>,
}

/// Inbound receiver service that replaces discarded `_inbound_rx`.
pub struct InboundService {
    receiver: Option<Receiver<InboundMessage>>,
    processed_count: usize,
    max_message_bytes: usize,
}

impl InboundService {
    /// Create a new inbound service holding the receiver channel.
    pub fn new(receiver: Receiver<InboundMessage>, max_message_bytes: usize) -> Self {
        Self {
            receiver: Some(receiver),
            processed_count: 0,
            max_message_bytes,
        }
    }

    /// Process all pending inbound messages from the channel.
    ///
    /// Fails with `KappaError::ReceiverDropped` if the channel was dropped.
    pub fn process_inbound<F>(&mut self, mut handler: F) -> Result<usize, KappaError>
    where
        F: FnMut(&InboundMessage) -> Result<(), KappaError>,
    {
        let rx = self.receiver.as_ref().ok_or(KappaError::ReceiverDropped)?;
        let mut count = 0;

        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    if msg.payload.len() > self.max_message_bytes {
                        return Err(KappaError::PayloadTooLarge {
                            max_bytes: self.max_message_bytes,
                            actual_bytes: msg.payload.len(),
                        });
                    }
                    handler(&msg)?;
                    count += 1;
                    self.processed_count += 1;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if count == 0 && self.processed_count == 0 {
                        return Err(KappaError::ReceiverDropped);
                    }
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Check if the receiver channel is intact.
    pub fn is_receiver_intact(&self) -> bool {
        self.receiver.is_some()
    }

    /// Explicitly discard receiver (to test fault detection).
    pub fn simulate_discard_receiver(&mut self) {
        self.receiver = None;
    }

    /// Number of successfully processed messages.
    pub fn processed_count(&self) -> usize {
        self.processed_count
    }
}

/// Transport peer interface for reconciliation.
pub trait TransportPeer {
    /// Unique peer identifier.
    fn id(&self) -> &str;
    /// Pull tags advertised by this peer for the given namespace.
    fn pull_tags(&self, ns: &str) -> Result<Vec<Tag>, KappaError>;
    /// Fetch blob payload bytes for a referenced content digest.
    fn pull_blob(&self, ns: &str, digest: &str) -> Result<Vec<u8>, KappaError>;
}

/// Report summarizing reconciliation results.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    /// Number of namespaces processed.
    pub namespaces_reconciled: usize,
    /// Number of tags updated.
    pub tags_pulled: usize,
    /// Number of missing blobs fetched and stored.
    pub blobs_transferred: usize,
    /// Total bytes transferred.
    pub bytes_transferred: usize,
    /// Non-fatal diagnostic errors encountered.
    pub errors: Vec<String>,
}

/// Reconciliation engine that guarantees verified blob transfer before tag commits.
pub struct ReconciliationEngine<'a> {
    store: &'a mut InMemoryObjectStore,
    config: &'a KappaConfig,
}

impl<'a> ReconciliationEngine<'a> {
    /// Create a new reconciliation engine.
    pub fn new(store: &'a mut InMemoryObjectStore, config: &'a KappaConfig) -> Self {
        Self { store, config }
    }

    /// Reconcile a namespace with a remote peer.
    ///
    /// For every tag advertised by the peer:
    /// 1. Verifies if local store already has the tag and its blob.
    /// 2. If the referenced blob is missing locally:
    ///    a. Fetches blob bytes from the peer.
    ///    b. Validates payload length against `max_blob_bytes`.
    ///    c. Cryptographically verifies content digest against tag digest.
    ///    d. Persists blob in local store.
    /// 3. Only after the blob is securely stored, commits the tag.
    ///
    /// If any blob is missing, truncated, or fails digest verification,
    /// the reconciliation aborts or records an error, leaving local state uncorrupted.
    pub fn reconcile<P: TransportPeer>(
        &mut self,
        ns: &str,
        peer: &P,
    ) -> Result<ReconcileReport, KappaError> {
        let mut report = ReconcileReport::default();
        let remote_tags = peer.pull_tags(ns)?;

        for tag in remote_tags {
            // Check if local store already has this exact tag and blob
            let local_tag = self.store.get_tag(ns, &tag.name);
            let blob_present = self.store.has_blob(&tag.kappa);

            if local_tag.as_deref() == Some(&tag.kappa) && blob_present {
                continue;
            }

            // If blob is missing, fetch and verify it
            if !blob_present {
                let blob_bytes = peer.pull_blob(ns, &tag.kappa)?;

                if blob_bytes.len() > self.config.service.max_blob_bytes {
                    return Err(KappaError::PayloadTooLarge {
                        max_bytes: self.config.service.max_blob_bytes,
                        actual_bytes: blob_bytes.len(),
                    });
                }

                if self.config.reconciliation.require_blob_content_verification {
                    let actual_digest = compute_sha256_digest(&blob_bytes);
                    if actual_digest != tag.kappa {
                        if self.config.reconciliation.reject_partial_reconciliation {
                            return Err(KappaError::DigestMismatch {
                                expected: tag.kappa.clone(),
                                actual: actual_digest,
                            });
                        } else {
                            report.errors.push(format!(
                                "digest mismatch for tag '{}': expected {}, got {}",
                                tag.name, tag.kappa, actual_digest
                            ));
                            continue;
                        }
                    }
                }

                let blob_len = blob_bytes.len();
                let blob = Blob {
                    digest: tag.kappa.clone(),
                    data: blob_bytes,
                };

                self.store.put_blob(blob)?;
                report.blobs_transferred += 1;
                report.bytes_transferred += blob_len;
            }

            // Commit tag only after verified blob is in store
            self.store.set_tag(ns, &tag.name, &tag.kappa)?;
            report.tags_pulled += 1;
        }

        if report.tags_pulled > 0 || report.blobs_transferred > 0 {
            report.namespaces_reconciled += 1;
        }

        Ok(report)
    }
}

/// Inbound transport channel pair.
pub type InboundChannel = (SyncSender<InboundMessage>, Receiver<InboundMessage>);

/// Create an inbound transport channel pair with configured capacity.
pub fn create_inbound_channel(capacity: usize) -> InboundChannel {
    std::sync::mpsc::sync_channel(capacity)
}
