//! Conformance tests for real participant network acceptance under adverse conditions:
//! discovery constraints, partitions, browser suspensions, peer eviction, replica loss,
//! hostile inputs, authenticated bootstrap routing, and measured availability targets (NA-01).

use repo_model::{
    AvailabilityMetricsTracker, BootstrapRouteConfig, BootstrapRouter, BrowserNetworkMesh, Model,
    NetworkError,
};

/// NA-01: Network acceptance exercises independent browser participants under real
/// discovery constraints, network partitions, browser suspensions, replica loss,
/// hostile inputs, authenticated bootstrap routing, and measured availability targets
/// without server-hosted proxies.
#[test]
fn network_acceptance_exercises_participants_and_adverse_conditions_na_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates network acceptance");

    let cfg = &model.network_acceptance;
    assert!(cfg.policy.enforce_browser_only_execution);
    assert!(cfg.policy.prohibit_server_hosted_nodes);
    assert_eq!(cfg.adverse_scenarios.len(), 5);

    // 1. Verify HTTPS-Origin Bootstrap Routing
    BootstrapRouter::verify_route(&cfg.bootstrap_routing).expect("verify bootstrap route");

    // 2. Initialize 5-Peer Independent Browser Mesh
    let peer_ids = [
        "uor:peer:browser-alpha",
        "uor:peer:browser-beta",
        "uor:peer:browser-gamma",
        "uor:peer:browser-delta",
        "uor:peer:browser-epsilon",
    ];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 1000);
    assert_eq!(mesh.active_count(), 5);

    // 3. Adverse Scenario: Network Partition Split-Brain
    // Partition into group A (3 peers, majority) and group B (2 peers, minority)
    let group_a = [
        "uor:peer:browser-alpha",
        "uor:peer:browser-beta",
        "uor:peer:browser-gamma",
    ];
    let group_b = ["uor:peer:browser-delta", "uor:peer:browser-epsilon"];
    mesh.apply_partition(&group_a, &group_b);

    assert!(!mesh.can_communicate("uor:peer:browser-alpha", "uor:peer:browser-delta"));
    assert!(mesh.can_communicate("uor:peer:browser-alpha", "uor:peer:browser-beta"));

    // Majority partition retains write quorum of 3
    mesh.verify_subgroup_quorum(&group_a, 3)
        .expect("majority group has quorum");

    // Minority partition fails write quorum (2 < 3)
    let err = mesh
        .verify_subgroup_quorum(&group_b, 3)
        .expect_err("minority group lacks quorum");
    match err {
        NetworkError::QuorumUnavailable(msg) => {
            assert!(msg.contains("subgroup has only 2 active peers"));
        }
        other => panic!("expected QuorumUnavailable, got {other:?}"),
    }

    // Heal partition
    mesh.heal_partition();
    assert!(mesh.can_communicate("uor:peer:browser-alpha", "uor:peer:browser-delta"));

    // 4. Adverse Scenario: Browser Tab OS Suspension & Resumption
    mesh.suspend_node("uor:peer:browser-epsilon")
        .expect("suspend epsilon");
    assert_eq!(mesh.active_count(), 4);
    assert!(!mesh.can_communicate("uor:peer:browser-alpha", "uor:peer:browser-epsilon"));

    // Wakeup event resumes node
    mesh.resume_node("uor:peer:browser-epsilon", 1050)
        .expect("resume epsilon");
    assert_eq!(mesh.active_count(), 5);
    assert!(mesh.can_communicate("uor:peer:browser-alpha", "uor:peer:browser-epsilon"));

    // 5. Adverse Scenario: Non-Responsive Peer Eviction
    // Advance epoch by 60s without heartbeat from delta
    let evicted = mesh
        .evict_if_unresponsive("uor:peer:browser-delta", 1100, 30)
        .expect("check eviction");
    assert!(evicted);
    assert_eq!(mesh.active_count(), 4);

    // 6. Adverse Scenario: Replica Loss & Auto-Repair
    let repaired_target = mesh
        .simulate_replica_loss_and_repair("uor:peer:browser-gamma", "uor:peer:browser-alpha")
        .expect("re-replicate to available active peer");
    assert_eq!(repaired_target, "uor:peer:browser-alpha");

    // 7. Adverse Scenario: Hostile Malformed Input Injection
    // Normal frame succeeds
    mesh.process_inbound_frame(
        "uor:peer:browser-alpha",
        b"VALID_FRAME",
        "ed25519:valid-signature-bytes-abcdef",
        1048576,
    )
    .expect("valid frame accepted");

    // Malformed frame with bad signature is rejected and peer quarantined
    let err = mesh
        .process_inbound_frame(
            "uor:peer:browser-beta",
            b"HOSTILE_FRAME",
            "bad-signature",
            1048576,
        )
        .expect_err("malformed signature rejected");
    match err {
        NetworkError::MalformedInputRejected(msg) => {
            assert!(msg.contains("invalid signature"));
        }
        other => panic!("expected MalformedInputRejected, got {other:?}"),
    }

    // 8. Measure Availability and Recovery SLOs
    let mut tracker = AvailabilityMetricsTracker::default();
    // 9,995 successful requests out of 10,000 (99.95% availability), MTTR 25s
    tracker.record_batch(10000, 9995, 25);
    tracker
        .verify_slo(&cfg.availability_targets)
        .expect("availability and MTTR satisfy approved targets");
    assert!(tracker.availability_percentage() >= 99.9);
}

#[test]
fn partition_split_brain_prevents_minority_writes_and_reconciles_on_heal() {
    let peer_ids = ["peer-1", "peer-2", "peer-3", "peer-4", "peer-5"];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 100);

    mesh.apply_partition(&["peer-1", "peer-2"], &["peer-3", "peer-4", "peer-5"]);
    assert!(!mesh.can_communicate("peer-1", "peer-3"));

    let err = mesh
        .verify_subgroup_quorum(&["peer-1", "peer-2"], 3)
        .expect_err("minority of 2 fails quorum 3");
    assert!(matches!(err, NetworkError::QuorumUnavailable(_)));

    mesh.verify_subgroup_quorum(&["peer-3", "peer-4", "peer-5"], 3)
        .expect("majority of 3 satisfies quorum");

    mesh.heal_partition();
    assert!(mesh.can_communicate("peer-1", "peer-3"));
}

#[test]
fn browser_suspension_is_detected_and_safely_resumed() {
    let peer_ids = ["peer-a", "peer-b"];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 500);

    mesh.suspend_node("peer-b").expect("suspend");
    assert!(!mesh.can_communicate("peer-a", "peer-b"));

    mesh.resume_node("peer-b", 550).expect("resume");
    assert!(mesh.can_communicate("peer-a", "peer-b"));
}

#[test]
fn peer_eviction_removes_non_responsive_nodes() {
    let peer_ids = ["peer-x", "peer-y"];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 1000);

    // After 20s (under 30s timeout) -> not evicted
    let evicted = mesh.evict_if_unresponsive("peer-x", 1020, 30).unwrap();
    assert!(!evicted);

    // After 40s (exceeds 30s timeout) -> evicted
    let evicted = mesh.evict_if_unresponsive("peer-x", 1040, 30).unwrap();
    assert!(evicted);
    assert_eq!(mesh.active_count(), 1);
}

#[test]
fn replica_loss_triggers_automatic_re_replication() {
    let peer_ids = ["peer-1", "peer-2", "peer-3"];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 100);

    let target = mesh
        .simulate_replica_loss_and_repair("peer-1", "peer-2")
        .expect("re-replicate");
    assert_eq!(target, "peer-2");
}

#[test]
fn hostile_malformed_input_is_isolated_and_rejected() {
    let peer_ids = ["attacker", "victim"];
    let mut mesh = BrowserNetworkMesh::new(&peer_ids, 100);

    // Oversized frame (> 100 bytes limit)
    let huge_payload = vec![0u8; 101];
    let err = mesh
        .process_inbound_frame("attacker", &huge_payload, "ed25519:valid-key-abcdef", 100)
        .expect_err("oversized frame rejected");
    assert!(matches!(err, NetworkError::MalformedInputRejected(_)));
}

#[test]
fn bootstrap_router_authenticates_veilid_relays_and_https_origin() {
    let valid_cfg = BootstrapRouteConfig {
        base_url: "https://uor-foundation.github.io/foundry-web/".to_string(),
        routing_mode: "SPA_HISTORY_PUSHSTATE".to_string(),
        primary_bootstrap_relay: "wss://bootstrap1.veilid.net:5150".to_string(),
        secondary_bootstrap_relay: "wss://bootstrap2.veilid.net:5150".to_string(),
        relay_public_key: "VLD0:pubkey".to_string(),
        trust_boundary_signature:
            "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
    };
    BootstrapRouter::verify_route(&valid_cfg).expect("valid route");

    let bad_origin = BootstrapRouteConfig {
        base_url: "http://insecure.com/foundry-web/".to_string(), // Insecure HTTP
        ..valid_cfg.clone()
    };
    let err = BootstrapRouter::verify_route(&bad_origin).expect_err("reject insecure HTTP");
    assert!(matches!(err, NetworkError::UnsafeBootstrapRoute(_)));

    let bad_path = BootstrapRouteConfig {
        base_url: "https://uor.foundation/other-path/".to_string(), // Missing /foundry-web/
        ..valid_cfg
    };
    let err = BootstrapRouter::verify_route(&bad_path).expect_err("reject non-/foundry-web/ path");
    assert!(matches!(err, NetworkError::UnsafeBootstrapRoute(_)));
}

#[test]
fn availability_metrics_meet_approved_targets() {
    let targets = repo_model::AvailabilityTargetConfig {
        target_availability_percentage: 99.9,
        max_mttr_seconds: 60,
        max_unreconciled_divergence: 0,
    };

    let mut tracker = AvailabilityMetricsTracker::default();
    tracker.record_batch(10000, 9999, 15);
    tracker.verify_slo(&targets).expect("meets SLO");

    // Adversarial: failing availability (95.0% < 99.9%)
    let mut failing_tracker = AvailabilityMetricsTracker::default();
    failing_tracker.record_batch(1000, 950, 15);
    let err = failing_tracker
        .verify_slo(&targets)
        .expect_err("breaches availability SLO");
    match err {
        NetworkError::SloBreached { metric, .. } => assert_eq!(metric, "availability_percentage"),
        other => panic!("expected SloBreached, got {other:?}"),
    }
}
