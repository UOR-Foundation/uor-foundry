//! Conformance tests for Browser Object Space storage, queries, verified transfer,
//! authenticated membership, confidentiality, conflicts, revocation, retention, replication,
//! repair, and recovery (BO-01).

use repo_model::{
    compute_sha256, AntiEntropyRepair, BlobTransferEngine, BrowserObjectStore, ConflictResolver,
    Model, ObjectRecord, ObjectSpaceError, PeerRegistry, QueryFilter, RecoveryEngine,
    ReplicationCoordinator,
};

/// BO-01: Browser object space implements authenticated membership, confidentiality,
/// storage queries, inbound dispatch, verified blob transfer, conflict resolution,
/// revocation, retention, replication, repair, and recovery without server-hosted substitutes.
#[test]
fn browser_object_space_enforces_lifecycle_and_security_bo_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates browser object space");

    let cfg = &model.object_space;
    assert_eq!(cfg.peers.len(), 3);
    assert_eq!(cfg.partitions.len(), 2);

    // 1. Authenticated Membership
    let mut registry = PeerRegistry::from_config(cfg);
    let alpha = registry
        .authenticate("uor:peer:browser-alpha", "uor:token:admit-alpha-2026")
        .expect("alpha authenticates");
    assert_eq!(alpha.organization_id, "uor:org:uor-foundation");

    // 2. Storage, Confidentiality & Content Verification
    let mut store_alpha = BrowserObjectStore::default();
    let payload = b"ENC(encrypted-workspace-state-bytes)";
    let digest = compute_sha256(payload);

    let record = ObjectRecord {
        tag: "uor:tag:workspace:root".to_string(),
        namespace: "uor-foundry:shared".to_string(),
        digest: digest.clone(),
        payload: payload.to_vec(),
        is_encrypted: true,
        clock: 1,
        is_tombstone: false,
        created_at_epoch: 1774000000,
    };

    store_alpha
        .put(record, cfg.policy.max_blob_bytes, true)
        .expect("put encrypted confidential record");

    let fetched = store_alpha
        .get("uor:tag:workspace:root")
        .expect("get record");
    assert_eq!(fetched.digest, digest);
    assert!(fetched.is_encrypted);

    // 3. Query Evaluation
    let query_results = store_alpha.query(&QueryFilter {
        namespace: Some("uor-foundry:shared".to_string()),
        tag_prefix: Some("uor:tag:workspace:".to_string()),
        include_tombstones: false,
        limit: 10,
    });
    assert_eq!(query_results.len(), 1);
    assert_eq!(query_results[0].tag, "uor:tag:workspace:root");

    // 4. Verified Blob Transfer
    let transferred =
        BlobTransferEngine::receive_and_verify_blob(&digest, payload, cfg.policy.max_blob_bytes)
            .expect("receive and verify blob transfer");
    assert_eq!(transferred, payload);

    // 5. Conflict Resolution (Logical Clock)
    let update_higher_clock = ObjectRecord {
        tag: "uor:tag:workspace:root".to_string(),
        namespace: "uor-foundry:shared".to_string(),
        digest: compute_sha256(b"ENC(updated-state-clock-2)"),
        payload: b"ENC(updated-state-clock-2)".to_vec(),
        is_encrypted: true,
        clock: 2,
        is_tombstone: false,
        created_at_epoch: 1774000100,
    };
    let winner = ConflictResolver::resolve(fetched, &update_higher_clock);
    assert_eq!(winner.clock, 2);
    assert_eq!(winner.digest, update_higher_clock.digest);

    // 6. Replication Quorum Verification
    let confirmations = vec![
        "uor:peer:browser-alpha".to_string(),
        "uor:peer:browser-beta".to_string(),
        "uor:peer:browser-gamma".to_string(),
    ];
    ReplicationCoordinator::verify_quorum(
        &confirmations,
        &registry,
        cfg.policy.min_replication_factor,
    )
    .expect("quorum of 3 satisfied");

    // 7. Revocation Fencing
    registry
        .revoke_peer("uor:peer:browser-gamma")
        .expect("revoke peer gamma");
    let err = registry
        .authenticate("uor:peer:browser-gamma", "uor:token:admit-gamma-2026")
        .expect_err("revoked peer must fail authentication");
    assert_eq!(
        err,
        ObjectSpaceError::RevokedPeer("uor:peer:browser-gamma".to_string())
    );

    // After revocation, confirmations including revoked peer fail quorum
    let err = ReplicationCoordinator::verify_quorum(
        &confirmations,
        &registry,
        cfg.policy.min_replication_factor,
    )
    .expect_err("quorum with revoked peer fails");
    assert_eq!(
        err,
        ObjectSpaceError::QuorumDeficit {
            replicas: 2,
            required: 3,
        }
    );

    // 8. Tombstone & Retention
    store_alpha
        .tombstone("uor:tag:workspace:root", 3, 1774000200)
        .expect("write tombstone");
    assert!(store_alpha.get("uor:tag:workspace:root").is_none());
    assert!(
        store_alpha
            .get_raw("uor:tag:workspace:root")
            .unwrap()
            .is_tombstone
    );

    // 9. Anti-Entropy Peer Repair
    let mut store_beta = BrowserObjectStore::default();
    let public_payload = b"PUBLIC_SITE_METADATA";
    let pub_digest = compute_sha256(public_payload);
    let pub_rec = ObjectRecord {
        tag: "uor:tag:public:info".to_string(),
        namespace: "uor-foundry:public-metadata".to_string(),
        digest: pub_digest,
        payload: public_payload.to_vec(),
        is_encrypted: false,
        clock: 1,
        is_tombstone: false,
        created_at_epoch: 1774000300,
    };
    store_beta
        .put(pub_rec, cfg.policy.max_blob_bytes, false)
        .expect("store pub rec on beta");

    let repaired = AntiEntropyRepair::sync_stores(
        &mut store_alpha,
        &store_beta,
        cfg.policy.max_blob_bytes,
        false,
    )
    .expect("anti-entropy sync");
    assert_eq!(repaired, 1);
    assert!(store_alpha.get("uor:tag:public:info").is_some());

    // 10. Offline Recovery
    let mut offline_store = BrowserObjectStore::default();
    let recovered = RecoveryEngine::recover_and_reconcile(
        &mut offline_store,
        &[&store_alpha, &store_beta],
        cfg.policy.max_blob_bytes,
        false,
    )
    .expect("recover offline store");
    assert!(recovered >= 1);
    assert!(offline_store.get("uor:tag:public:info").is_some());
}

#[test]
fn unauthenticated_peer_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let registry = PeerRegistry::from_config(&model.object_space);

    let err = registry
        .authenticate("uor:peer:browser-alpha", "wrong-admission-token")
        .expect_err("wrong token fails");
    match err {
        ObjectSpaceError::UnauthenticatedPeer(msg) => {
            assert!(msg.contains("invalid admission token"))
        }
        other => panic!("expected UnauthenticatedPeer, got {other:?}"),
    }

    let err = registry
        .authenticate("uor:peer:stranger", "any-token")
        .expect_err("unknown peer fails");
    match err {
        ObjectSpaceError::UnauthenticatedPeer(msg) => assert!(msg.contains("uor:peer:stranger")),
        other => panic!("expected UnauthenticatedPeer, got {other:?}"),
    }
}

#[test]
fn unencrypted_confidential_blob_is_rejected() {
    let mut store = BrowserObjectStore::default();
    let plaintext = b"SECRET_PLAINTEXT_SHOULD_BE_ENCRYPTED";
    let digest = compute_sha256(plaintext);

    let record = ObjectRecord {
        tag: "uor:tag:confidential:leak".to_string(),
        namespace: "uor-foundry:shared".to_string(),
        digest,
        payload: plaintext.to_vec(),
        is_encrypted: false, // Invariant violation: must be encrypted for shared confidential partition
        clock: 1,
        is_tombstone: false,
        created_at_epoch: 1774000000,
    };

    let err = store
        .put(record, 10485760, true)
        .expect_err("unencrypted confidential record must be rejected");
    match err {
        ObjectSpaceError::ConfidentialityViolation(msg) => {
            assert!(msg.contains("unencrypted confidential record"));
        }
        other => panic!("expected ConfidentialityViolation, got {other:?}"),
    }
}

#[test]
fn corrupted_blob_digest_is_rejected_on_transfer() {
    let valid_payload = b"VALID_BYTES";
    let valid_digest = compute_sha256(valid_payload);

    let tampered_payload = b"TAMPERED_BYTES";

    let err =
        BlobTransferEngine::receive_and_verify_blob(&valid_digest, tampered_payload, 10485760)
            .expect_err("tampered payload must fail digest verification");

    match err {
        ObjectSpaceError::DigestMismatch { expected, computed } => {
            assert_eq!(expected, valid_digest);
            assert_eq!(computed, compute_sha256(tampered_payload));
        }
        other => panic!("expected DigestMismatch, got {other:?}"),
    }
}

#[test]
fn concurrent_conflict_is_resolved_deterministically() {
    let local = ObjectRecord {
        tag: "uor:tag:crdt:test".to_string(),
        namespace: "uor-foundry:shared".to_string(),
        digest: compute_sha256(b"AAA"),
        payload: b"AAA".to_vec(),
        is_encrypted: true,
        clock: 5,
        is_tombstone: false,
        created_at_epoch: 1774000000,
    };

    let remote = ObjectRecord {
        tag: "uor:tag:crdt:test".to_string(),
        namespace: "uor-foundry:shared".to_string(),
        digest: compute_sha256(b"BBB"),
        payload: b"BBB".to_vec(),
        is_encrypted: true,
        clock: 6, // Higher clock wins
        is_tombstone: false,
        created_at_epoch: 1774000010,
    };

    let winner = ConflictResolver::resolve(&local, &remote);
    assert_eq!(winner.clock, 6);
    assert_eq!(winner.digest, remote.digest);
}

#[test]
fn replication_quorum_deficit_is_detected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let registry = PeerRegistry::from_config(&model.object_space);

    let insufficient_confirmations = vec!["uor:peer:browser-alpha".to_string()];
    let err = ReplicationCoordinator::verify_quorum(&insufficient_confirmations, &registry, 3)
        .expect_err("1 confirmation fails quorum of 3");
    assert_eq!(
        err,
        ObjectSpaceError::QuorumDeficit {
            replicas: 1,
            required: 3,
        }
    );
}

#[test]
fn anti_entropy_repairs_missing_replicas() {
    let mut local = BrowserObjectStore::default();
    let mut remote = BrowserObjectStore::default();

    let payload = b"REPAIR_ME";
    let digest = compute_sha256(payload);
    let record = ObjectRecord {
        tag: "uor:tag:repair:target".to_string(),
        namespace: "uor-foundry:public-metadata".to_string(),
        digest,
        payload: payload.to_vec(),
        is_encrypted: false,
        clock: 1,
        is_tombstone: false,
        created_at_epoch: 1774000000,
    };
    remote.put(record, 10485760, false).expect("put remote");

    assert!(local.get("uor:tag:repair:target").is_none());

    let repaired =
        AntiEntropyRepair::sync_stores(&mut local, &remote, 10485760, false).expect("sync stores");
    assert_eq!(repaired, 1);
    assert!(local.get("uor:tag:repair:target").is_some());
}
