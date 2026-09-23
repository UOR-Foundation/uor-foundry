//! Conformance tests for Kappa browser-service receiver and verified blob reconciliation (KB-01).

use repo_model::{
    compute_sha256_digest, create_inbound_channel, InMemoryObjectStore, InboundMessage,
    InboundService, KappaError, Model, ReconciliationEngine, Tag, TransportPeer,
};

/// A test peer implementation for simulating network peers during reconciliation.
struct MockPeer {
    id: String,
    tags: Vec<Tag>,
    blobs: std::collections::BTreeMap<String, Vec<u8>>,
}

impl MockPeer {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            tags: Vec::new(),
            blobs: std::collections::BTreeMap::new(),
        }
    }

    fn add_entry(&mut self, tag_name: &str, data: Vec<u8>) -> String {
        let digest = compute_sha256_digest(&data);
        self.tags.push(Tag {
            name: tag_name.to_string(),
            kappa: digest.clone(),
        });
        self.blobs.insert(digest.clone(), data);
        digest
    }
}

impl TransportPeer for MockPeer {
    fn id(&self) -> &str {
        &self.id
    }

    fn pull_tags(&self, _ns: &str) -> Result<Vec<Tag>, KappaError> {
        Ok(self.tags.clone())
    }

    fn pull_blob(&self, _ns: &str, digest: &str) -> Result<Vec<u8>, KappaError> {
        self.blobs
            .get(digest)
            .cloned()
            .ok_or_else(|| KappaError::BlobNotFound(digest.to_string()))
    }
}

/// KB-01: The browser object-space service retains inbound transport receiver semantics
/// and executes verified blob reconciliation, rejecting partial, unverified, or
/// disconnected state updates.
#[test]
fn kappa_browser_service_retains_receiver_and_verifies_blobs_kb_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates kappa service configuration");

    let kappa_cfg = &model.kappa;

    // 1. Verify inbound receiver semantics are retained and serviced
    let (tx, rx) = create_inbound_channel(kappa_cfg.service.inbound_channel_capacity);
    let mut service = InboundService::new(rx, kappa_cfg.service.max_message_bytes);
    assert!(
        service.is_receiver_intact(),
        "inbound receiver channel must be intact"
    );

    // Send inbound transport messages
    let msg1 = InboundMessage {
        peer_id: "peer-node-alpha".to_string(),
        op: "diff_pull".to_string(),
        payload: b"namespace=uor-foundry".to_vec(),
    };
    let msg2 = InboundMessage {
        peer_id: "peer-node-beta".to_string(),
        op: "inbound_event".to_string(),
        payload: b"status=active".to_vec(),
    };
    tx.send(msg1.clone()).expect("send msg1");
    tx.send(msg2.clone()).expect("send msg2");

    let mut received = Vec::new();
    let processed = service
        .process_inbound(|msg| {
            received.push(msg.clone());
            Ok(())
        })
        .expect("processing inbound messages succeeds");

    assert_eq!(processed, 2, "must process 2 inbound messages");
    assert_eq!(service.processed_count(), 2);
    assert_eq!(received[0].peer_id, "peer-node-alpha");
    assert_eq!(received[1].peer_id, "peer-node-beta");

    // 2. Verify reconciliation preserves referenced blob bytes with content verification
    let mut local_store = InMemoryObjectStore::new();
    let mut peer = MockPeer::new("peer-node-gamma");

    let payload1 = b"Foundry Object Space Artifact: Staged Core v1".to_vec();
    let payload2 = b"Foundry Object Space Schema: Catalog 2026".to_vec();
    let digest1 = peer.add_entry("main", payload1.clone());
    let digest2 = peer.add_entry("schema/catalog", payload2.clone());

    let mut engine = ReconciliationEngine::new(&mut local_store, kappa_cfg);
    let report = engine
        .reconcile(&kappa_cfg.service.namespace, &peer)
        .expect("reconciliation succeeds");

    assert_eq!(report.namespaces_reconciled, 1);
    assert_eq!(report.tags_pulled, 2);
    assert_eq!(report.blobs_transferred, 2);
    assert_eq!(report.bytes_transferred, payload1.len() + payload2.len());
    assert!(report.errors.is_empty());

    // Verify local store has both tags and verified blob bytes
    assert_eq!(
        local_store.get_tag(&kappa_cfg.service.namespace, "main"),
        Some(digest1.clone())
    );
    assert_eq!(
        local_store.get_tag(&kappa_cfg.service.namespace, "schema/catalog"),
        Some(digest2.clone())
    );

    let blob1 = local_store
        .get_blob(&digest1)
        .expect("blob1 must be persisted in local store");
    assert_eq!(blob1.data, payload1);
    assert_eq!(blob1.digest, digest1);

    let blob2 = local_store
        .get_blob(&digest2)
        .expect("blob2 must be persisted in local store");
    assert_eq!(blob2.data, payload2);
    assert_eq!(blob2.digest, digest2);
}

#[test]
fn kappa_reconciliation_rejects_missing_blob() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut local_store = InMemoryObjectStore::new();

    // Plant defect: peer advertises tag pointing to digest, but blob is missing from peer
    let mut peer = MockPeer::new("peer-missing-blob");
    peer.tags.push(Tag {
        name: "unbacked-tag".to_string(),
        kappa: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
    });
    // Deliberately do not add blob to peer.blobs

    let mut engine = ReconciliationEngine::new(&mut local_store, &model.kappa);
    let err = engine
        .reconcile(&model.kappa.service.namespace, &peer)
        .expect_err("reconciliation must fail when referenced blob is missing");

    match err {
        KappaError::BlobNotFound(d) => {
            assert!(d.contains("0000000000000000000000000000000000000000000000000000000000000000"))
        }
        other => panic!("expected BlobNotFound, got: {other}"),
    }

    // Verify tag was NOT committed
    assert_eq!(
        local_store.get_tag(&model.kappa.service.namespace, "unbacked-tag"),
        None,
        "unbacked tag must not be committed to local store"
    );
}

#[test]
fn kappa_reconciliation_rejects_corrupted_blob_digest() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut local_store = InMemoryObjectStore::new();

    // Plant defect: peer advertises tag with digest D, but provides blob data hashing to D'
    let mut peer = MockPeer::new("peer-tampered");
    let genuine_data = b"genuine authentic content".to_vec();
    let genuine_digest = compute_sha256_digest(&genuine_data);

    let tampered_data = b"malicious or corrupted tampered content".to_vec();

    peer.tags.push(Tag {
        name: "tampered-tag".to_string(),
        kappa: genuine_digest.clone(),
    });
    // Associate the genuine digest key with corrupted data payload
    peer.blobs.insert(genuine_digest.clone(), tampered_data);

    let mut engine = ReconciliationEngine::new(&mut local_store, &model.kappa);
    let err = engine
        .reconcile(&model.kappa.service.namespace, &peer)
        .expect_err("reconciliation must fail on cryptographic content mismatch");

    match err {
        KappaError::DigestMismatch { expected, actual } => {
            assert_eq!(expected, genuine_digest);
            assert_ne!(actual, genuine_digest);
        }
        other => panic!("expected DigestMismatch, got: {other}"),
    }

    // Verify corrupted tag was NOT committed
    assert_eq!(
        local_store.get_tag(&model.kappa.service.namespace, "tampered-tag"),
        None,
        "tag referencing corrupted blob must not be committed"
    );
    assert!(
        !local_store.has_blob(&genuine_digest),
        "corrupted blob must not be stored"
    );
}

#[test]
fn kappa_reconciliation_rejects_oversized_payload() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut local_store = InMemoryObjectStore::new();

    let mut peer = MockPeer::new("peer-oversized");
    // Create payload exceeding max_blob_bytes
    let oversized_bytes = vec![0u8; model.kappa.service.max_blob_bytes + 1024];
    peer.add_entry("huge-file", oversized_bytes);

    let mut engine = ReconciliationEngine::new(&mut local_store, &model.kappa);
    let err = engine
        .reconcile(&model.kappa.service.namespace, &peer)
        .expect_err("reconciliation must reject oversized blob payload");

    match err {
        KappaError::PayloadTooLarge {
            max_bytes,
            actual_bytes,
        } => {
            assert_eq!(max_bytes, model.kappa.service.max_blob_bytes);
            assert_eq!(actual_bytes, model.kappa.service.max_blob_bytes + 1024);
        }
        other => panic!("expected PayloadTooLarge, got: {other}"),
    }
}

#[test]
fn kappa_service_rejects_dropped_inbound_receiver() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let (_tx, rx) = create_inbound_channel(16);
    let mut service = InboundService::new(rx, model.kappa.service.max_message_bytes);

    // Simulate the upstream bug where receiver is dropped/discarded
    service.simulate_discard_receiver();
    assert!(
        !service.is_receiver_intact(),
        "receiver should report not intact"
    );

    let err = service
        .process_inbound(|_| Ok(()))
        .expect_err("servicing dropped receiver must fail");

    assert_eq!(err, KappaError::ReceiverDropped);
}

#[test]
fn kappa_object_store_rejects_tag_without_blob() {
    let mut store = InMemoryObjectStore::new();
    let dummy_digest = "sha256:1111111111111111111111111111111111111111111111111111111111111111";

    let err = store
        .set_tag("uor-foundry", "unverified", dummy_digest)
        .expect_err("setting tag without blob in store must fail");

    match err {
        KappaError::UnverifiedState(msg) => {
            assert!(msg.contains("referenced blob bytes are missing"));
        }
        other => panic!("expected UnverifiedState, got: {other}"),
    }
}

#[test]
fn kappa_model_rejects_dropped_inbound_receiver_config() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: set require_inbound_servicing to false
    model.kappa.service.require_inbound_servicing = false;

    let err = model
        .kappa
        .check(&model.owner_inputs)
        .expect_err("model must reject dropped inbound servicing configuration");

    assert!(
        err.to_string()
            .contains("require_inbound_servicing must be true"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn kappa_model_rejects_unverified_blob_config() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: disable blob content verification
    model.kappa.reconciliation.require_blob_content_verification = false;

    let err = model
        .kappa
        .check(&model.owner_inputs)
        .expect_err("model must reject disabled blob content verification");

    assert!(
        err.to_string()
            .contains("require_blob_content_verification must be true"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn kappa_model_rejects_excessive_blob_bounds() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: max_blob_bytes exceeds owner workload bounds
    model.kappa.service.max_blob_bytes =
        (model.owner_inputs.resilience_bounds.max_blob_bytes + 1) as usize;

    let err = model
        .kappa
        .check(&model.owner_inputs)
        .expect_err("model must reject blob bound exceeding owner inputs");

    assert!(
        err.to_string().contains("exceeds owner workload bounds"),
        "unexpected diagnostic: {err}"
    );
}
