//! Conformance tests for Holospaces boundary: threat model revalidation against
//! participant/faculty-only session realities, browser-only discovery and private-key
//! possession, rejection of native-relay assumptions and local-witness proxies, and
//! durability under participant session churn (HB-01).

use std::collections::HashSet;

use repo_model::{
    HolospacesBoundaryError, HolospacesMeshCoordinator, Model, ParticipantSessionRecord,
};

/// HB-01: The Holospaces external boundary revalidates threat-model assumptions against
/// participant/faculty session realities, requiring private-key possession proofs,
/// in-band authenticated discovery, and replica quorum durability under browser churn
/// while rejecting native-relay and local-witness proxy assumptions.
#[test]
fn holospaces_boundary_revalidates_threat_model_and_browser_durability_hb_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates holospaces boundary");

    let cfg = &model.holospaces_boundary;
    assert_eq!(cfg.spec, "foundry/holospaces-boundary/1");
    assert_eq!(cfg.stage, "staged-core");
    assert_eq!(
        cfg.pinned_upstream_commit,
        "96769f16be454ab1572fddff4613704ccfbebf5e"
    );

    assert!(cfg.threat_model.prohibit_native_relay_assumptions);
    assert!(cfg.threat_model.prohibit_local_witness_proxies);
    assert!(cfg.threat_model.prohibit_out_of_band_signaling);
    assert!(cfg.threat_model.require_private_key_possession);
    assert!(cfg.threat_model.require_participant_session_churn_model);
    assert!(cfg.threat_model.prohibit_all_offline_execution);

    assert_eq!(cfg.durability.min_participant_replicas, 3);
    assert_eq!(cfg.durability.max_acceptable_offline_hours, 72);
    assert_eq!(cfg.durability.anti_entropy_repair_frequency_seconds, 300);
    assert_eq!(cfg.durability.required_quorum_percentage, 60.0);

    assert_eq!(
        cfg.discovery.primary_discovery,
        "veilid-authenticated-relay"
    );
    assert_eq!(cfg.discovery.webrtc_signaling, "authenticated-in-band");
    assert!(cfg.discovery.prohibit_out_of_band_rendezvous);

    // 1. Verify valid participant session under browser-only threat model
    let valid_session = ParticipantSessionRecord {
        session_id: "faculty-session-01".to_string(),
        participant_role: "faculty".to_string(),
        is_browser_tab: true,
        assumes_native_daemon: false,
        proven_private_key_possession: true,
        is_online: true,
        held_object_keys: ["obj-curriculum-01".to_string()].into_iter().collect(),
    };
    HolospacesMeshCoordinator::validate_participant_session(&valid_session, cfg)
        .expect("valid participant session passes");

    // 2. Verify in-band production discovery claim
    HolospacesMeshCoordinator::evaluate_discovery_claim(
        "veilid-authenticated-relay",
        false,
        false,
        cfg,
    )
    .expect("production discovery claim succeeds");

    // 3. Verify mesh durability with quorum of 3 replicas
    let sessions = vec![
        valid_session,
        ParticipantSessionRecord {
            session_id: "student-session-01".to_string(),
            participant_role: "student".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: true,
            held_object_keys: ["obj-curriculum-01".to_string()].into_iter().collect(),
        },
        ParticipantSessionRecord {
            session_id: "student-session-02".to_string(),
            participant_role: "student".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: true,
            held_object_keys: ["obj-curriculum-01".to_string()].into_iter().collect(),
        },
    ];

    let replicas =
        HolospacesMeshCoordinator::evaluate_mesh_durability(&sessions, "obj-curriculum-01", cfg)
            .expect("quorum of 3 replicas satisfies durability");
    assert_eq!(replicas, 3);
}

#[test]
fn native_relay_assumptions_are_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.holospaces_boundary;

    let bad_session = ParticipantSessionRecord {
        session_id: "invalid-relay-session".to_string(),
        participant_role: "student".to_string(),
        is_browser_tab: true,
        assumes_native_daemon: true, // Native daemon assumed!
        proven_private_key_possession: true,
        is_online: true,
        held_object_keys: HashSet::new(),
    };

    let res = HolospacesMeshCoordinator::validate_participant_session(&bad_session, cfg);
    assert!(matches!(
        res,
        Err(HolospacesBoundaryError::NativeRelayAssumptionViolation(_))
    ));
}

#[test]
fn missing_private_key_possession_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.holospaces_boundary;

    let bad_session = ParticipantSessionRecord {
        session_id: "unproven-key-session".to_string(),
        participant_role: "faculty".to_string(),
        is_browser_tab: true,
        assumes_native_daemon: false,
        proven_private_key_possession: false, // Only has address/public key
        is_online: true,
        held_object_keys: HashSet::new(),
    };

    let res = HolospacesMeshCoordinator::validate_participant_session(&bad_session, cfg);
    assert!(matches!(
        res,
        Err(HolospacesBoundaryError::MissingPrivateKeyPossession(_))
    ));
}

#[test]
fn local_witness_proxies_and_out_of_band_signaling_are_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.holospaces_boundary;

    // Local witness proxy rejected
    let witness_res = HolospacesMeshCoordinator::evaluate_discovery_claim(
        "veilid-authenticated-relay",
        false,
        true, // Local witness proxy
        cfg,
    );
    assert!(matches!(
        witness_res,
        Err(HolospacesBoundaryError::LocalWitnessProxyViolation(_))
    ));

    // Out-of-band signaling rejected
    let oob_res = HolospacesMeshCoordinator::evaluate_discovery_claim(
        "veilid-authenticated-relay",
        true, // Out-of-band signaling
        false,
        cfg,
    );
    assert!(matches!(
        oob_res,
        Err(HolospacesBoundaryError::OutOfBandSignalingViolation(_))
    ));
}

#[test]
fn all_offline_participants_cannot_execute_services() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.holospaces_boundary;

    let offline_sessions = vec![
        ParticipantSessionRecord {
            session_id: "offline-1".to_string(),
            participant_role: "faculty".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: false, // Offline!
            held_object_keys: ["obj-test".to_string()].into_iter().collect(),
        },
        ParticipantSessionRecord {
            session_id: "offline-2".to_string(),
            participant_role: "student".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: false, // Offline!
            held_object_keys: ["obj-test".to_string()].into_iter().collect(),
        },
    ];

    let res =
        HolospacesMeshCoordinator::evaluate_mesh_durability(&offline_sessions, "obj-test", cfg);
    assert!(matches!(
        res,
        Err(HolospacesBoundaryError::AllParticipantsOffline(_))
    ));
}

#[test]
fn quorum_deficit_triggers_error() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.holospaces_boundary;

    let under_replicated = vec![
        ParticipantSessionRecord {
            session_id: "online-1".to_string(),
            participant_role: "faculty".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: true,
            held_object_keys: ["obj-test".to_string()].into_iter().collect(),
        },
        ParticipantSessionRecord {
            session_id: "online-2".to_string(),
            participant_role: "student".to_string(),
            is_browser_tab: true,
            assumes_native_daemon: false,
            proven_private_key_possession: true,
            is_online: true,
            held_object_keys: HashSet::new(), // Does NOT hold obj-test
        },
    ];

    // Only 1 active replica for obj-test, but min required is 3
    let res =
        HolospacesMeshCoordinator::evaluate_mesh_durability(&under_replicated, "obj-test", cfg);
    assert_eq!(
        res,
        Err(HolospacesBoundaryError::QuorumDeficit {
            active: 1,
            required: 3,
        })
    );
}
