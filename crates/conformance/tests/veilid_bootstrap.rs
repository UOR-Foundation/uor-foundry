//! Conformance tests for Veilid Bootstrap boundary: authenticated secure bootstrap routes,
//! browser transport integration, relay public-key verification, outbound-relay limitation
//! mitigations, and network discovery under adverse conditions (VB-01).

use repo_model::{
    BootstrapPeerConfig, Model, RelaySelectorConfig, VeilidBootstrapEngine, VeilidBootstrapError,
};

/// VB-01: The Veilid external boundary enforces authenticated secure bootstrap routing,
/// browser transport integration, relay public-key verification, outbound-relay limitation
/// mitigations, and network discovery under adverse conditions without feature-flag-only
/// or unauthenticated proxy substitutes.
#[test]
fn veilid_bootstrap_enforces_secure_routing_and_transport_integration_vb_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates veilid bootstrap boundary");

    let cfg = &model.veilid_bootstrap;
    assert_eq!(cfg.spec, "foundry/veilid-bootstrap/1");
    assert_eq!(cfg.stage, "staged-core");
    assert!(cfg.policy.require_authenticated_bootstrap_route);
    assert!(cfg.policy.require_https_origin_routing);
    assert!(cfg.policy.prohibit_feature_flag_only_claims);
    assert!(cfg.policy.prohibit_unauthenticated_relays);
    assert!(cfg.policy.require_outbound_relay_mitigation);
    assert!(cfg.policy.enforce_wss_transport_encryption);

    assert_eq!(cfg.routing.bootstrap_peers.len(), 2);
    let primary = &cfg.routing.bootstrap_peers[0];
    let secondary = &cfg.routing.bootstrap_peers[1];

    assert!(primary.endpoint.starts_with("wss://"));
    assert!(secondary.endpoint.starts_with("wss://"));

    // 1. Verify bootstrap peer trust roots
    VeilidBootstrapEngine::verify_bootstrap_peer(primary).expect("primary bootstrap peer verifies");
    VeilidBootstrapEngine::verify_bootstrap_peer(secondary)
        .expect("secondary bootstrap peer verifies");

    // 2. Establish authenticated transport session
    let mut session = VeilidBootstrapEngine::connect_bootstrap(primary, &cfg.routing.https_origin)
        .expect("connects to primary bootstrap peer");

    assert_eq!(session.connected_peer, primary.peer_id);
    assert!(session.is_encrypted_wss);
    assert!(!session.outbound_relay_allocated);

    // 3. Mitigate Veilid 0.5.7 outbound relay limitation
    let relay = VeilidBootstrapEngine::allocate_outbound_relay(&mut session, &cfg.relay_selector)
        .expect("allocates outbound relay via fallback");
    assert!(session.outbound_relay_allocated);
    assert_eq!(
        session.outbound_relay_address.as_deref(),
        Some(relay.as_str())
    );

    // 4. Test peer churn and failover reconnection
    VeilidBootstrapEngine::simulate_peer_churn_and_reconnect(
        &mut session,
        &cfg.routing.bootstrap_peers,
    )
    .expect("reconnects to secondary peer upon churn");
    assert_eq!(session.connected_peer, secondary.peer_id);
    assert_eq!(session.endpoint, secondary.endpoint);
}

#[test]
fn insecure_ws_endpoint_is_rejected() {
    let insecure_peer = BootstrapPeerConfig {
        peer_id: "insecure-node-01".to_string(),
        endpoint: "ws://insecure.node.net:5150".to_string(), // Plaintext WS!
        public_key: "ed25519:6c53579899aa125bcad88219c158913619082beae7236abdc011855e780769ba"
            .to_string(),
        trust_root_digest:
            "sha256:d8c6b75aeae8c4974fbc173b2c12217c4e5ff09ab683b5444fae9eb10a2bb194".to_string(),
        role: "untrusted".to_string(),
    };

    let res = VeilidBootstrapEngine::verify_bootstrap_peer(&insecure_peer);
    assert!(matches!(
        res,
        Err(VeilidBootstrapError::InsecureEndpointRejected(_))
    ));
}

#[test]
fn unauthenticated_relay_with_bad_key_is_rejected() {
    let bad_key_peer = BootstrapPeerConfig {
        peer_id: "bad-key-node".to_string(),
        endpoint: "wss://bootstrap1.veilid.net:5150".to_string(),
        public_key: "insecure-plain-key".to_string(), // Missing ed25519: prefix
        trust_root_digest:
            "sha256:d8c6b75aeae8c4974fbc173b2c12217c4e5ff09ab683b5444fae9eb10a2bb194".to_string(),
        role: "untrusted".to_string(),
    };

    let res = VeilidBootstrapEngine::verify_bootstrap_peer(&bad_key_peer);
    assert!(matches!(
        res,
        Err(VeilidBootstrapError::UnauthenticatedRelayRejected(_))
    ));
}

#[test]
fn origin_path_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let peer = &model.veilid_bootstrap.routing.bootstrap_peers[0];

    // HTTP origin rejected
    let http_res = VeilidBootstrapEngine::connect_bootstrap(
        peer,
        "http://uor-foundation.github.io/foundry-web/",
    );
    assert!(matches!(
        http_res,
        Err(VeilidBootstrapError::OriginPathMismatch(_))
    ));

    // Wrong path rejected
    let wrong_path_res = VeilidBootstrapEngine::connect_bootstrap(
        peer,
        "https://uor-foundation.github.io/other-app/",
    );
    assert!(matches!(
        wrong_path_res,
        Err(VeilidBootstrapError::OriginPathMismatch(_))
    ));
}

#[test]
fn veilid_057_outbound_relay_missing_limitation_is_mitigated() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let peer = &model.veilid_bootstrap.routing.bootstrap_peers[0];
    let mut session = VeilidBootstrapEngine::connect_bootstrap(
        peer,
        &model.veilid_bootstrap.routing.https_origin,
    )
    .expect("session created");

    // Disabled fallback fails due to native 0.5.7 limitation
    let unmitigated_cfg = RelaySelectorConfig {
        min_reputable_relays: 2,
        protocol_version: "0.5.7".to_string(),
        routing_table_capacity: 256,
        enable_outbound_relay_fallback: false,
        fallback_relay_endpoint: String::new(),
    };

    let fail_res = VeilidBootstrapEngine::allocate_outbound_relay(&mut session, &unmitigated_cfg);
    assert!(matches!(
        fail_res,
        Err(VeilidBootstrapError::OutboundRelayMissing(_))
    ));

    // Mitigated fallback succeeds
    let success_res = VeilidBootstrapEngine::allocate_outbound_relay(
        &mut session,
        &model.veilid_bootstrap.relay_selector,
    );
    assert!(success_res.is_ok());
    assert!(session.outbound_relay_allocated);
}

#[test]
fn feature_flag_only_claim_is_rejected_without_live_session() {
    // Unsupported assumption: claiming WSS support by build feature flag alone
    let res = VeilidBootstrapEngine::reject_feature_flag_only_claim(
        false,
        "cargo-feature:veilid-core/wss",
    );
    assert!(matches!(
        res,
        Err(VeilidBootstrapError::UnsupportedAssumption(_))
    ));

    // Supported when backed by live verified session
    let valid_res = VeilidBootstrapEngine::reject_feature_flag_only_claim(
        true,
        "cargo-feature:veilid-core/wss",
    );
    assert!(valid_res.is_ok());
}
