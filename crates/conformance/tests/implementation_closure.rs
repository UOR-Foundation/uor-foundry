//! Conformance tests for Implementation Closure: complete uor-foundry producer
//! model and acceptance evidence (IC-01).

use repo_model::{
    AcceptedBoundaryRecord, ImplementationClosureEngine, ImplementationClosureError, Model,
};

/// IC-01: The implementation closure model verifies complete remaining-work closure
/// across all product requirements and boundaries, zero deferred or narrowed scope,
/// exact producer release identity binding, and handoff readiness for foundry-web
/// consumption without draft preview or handwritten substitutes.
#[test]
fn implementation_closure_verifies_complete_acceptance_evidence_ic_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates implementation closure");

    let cfg = &model.implementation_closure;
    assert_eq!(cfg.spec, "foundry/implementation-closure/1");
    assert_eq!(cfg.stage, "staged-core");
    assert!(cfg.policy.enforce_complete_row_closure);
    assert!(cfg.policy.prohibit_deferred_scope);
    assert!(cfg.policy.prohibit_mock_substitutes);
    assert!(cfg.policy.require_replayable_evidence);
    assert!(cfg.policy.require_producer_handoff_readiness);

    // 1. Verify all 15 required boundary IDs are registered and accepted
    let required_boundaries = [
        "SB-01", "ST-01", "OL-01", "OS-01", "SV-01", "AM-01", "EC-01", "BC-01", "BO-01", "NA-01",
        "PR-01", "PS-01", "VB-01", "HB-01", "FC-01",
    ];

    ImplementationClosureEngine::verify_complete_closure(cfg, &required_boundaries)
        .expect("all 15 required boundaries are accepted");

    // 2. Verify release identity binding and handoff readiness
    let expected_url = "https://uor-foundation.github.io/foundry-web/";
    let expected_tree_digest =
        "sha256:d8c6b75aeae8c4974fbc173b2c12217c4e5ff09ab683b5444fae9eb10a2bb194";

    ImplementationClosureEngine::verify_release_handoff_readiness(
        cfg,
        expected_url,
        expected_tree_digest,
    )
    .expect("release handoff readiness verified");

    assert_eq!(cfg.release_identity.producer_name, "uor-foundry-producer");
    assert_eq!(cfg.release_identity.release_version, "0.1.0");
    assert_eq!(cfg.release_identity.release_stage, "staged-core");
    assert_eq!(
        cfg.release_identity.pre_publication_binding,
        "sha256:91bf34020a5664bead868fbfa89196b6e41bf1684fa6e3f8484196c342ebcb92"
    );
}

#[test]
fn missing_boundary_fails_closure_verification() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut cfg = model.implementation_closure.clone();

    // Remove FC-01
    cfg.accepted_boundaries.retain(|b| b.id != "FC-01");

    let required_boundaries = ["FC-01"];
    let res = ImplementationClosureEngine::verify_complete_closure(&cfg, &required_boundaries);
    assert!(matches!(
        res,
        Err(ImplementationClosureError::UnacceptedBoundary(ref msg)) if msg.contains("FC-01")
    ));
}

#[test]
fn unaccepted_boundary_fails_config_check() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut cfg = model.implementation_closure.clone();

    cfg.accepted_boundaries.push(AcceptedBoundaryRecord {
        id: "TEST-01".to_string(),
        name: "Test boundary".to_string(),
        status: "unaccepted".to_string(),
    });

    let res = cfg.check(&model.owner_inputs, &model.organization_lifecycle);
    assert!(res.is_err());
}

#[test]
fn target_url_mismatch_fails_handoff_readiness() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.implementation_closure;

    let res = ImplementationClosureEngine::verify_release_handoff_readiness(
        cfg,
        "https://evil.target.com/web/",
        &cfg.release_identity.artifact_tree_digest,
    );
    assert!(matches!(
        res,
        Err(ImplementationClosureError::ReleaseIdentityMismatch(ref msg)) if msg.contains("target deployment url mismatch")
    ));
}

#[test]
fn tree_digest_mismatch_fails_handoff_readiness() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let cfg = &model.implementation_closure;

    let res = ImplementationClosureEngine::verify_release_handoff_readiness(
        cfg,
        &cfg.release_identity.target_deployment_url,
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(matches!(
        res,
        Err(ImplementationClosureError::ReleaseIdentityMismatch(ref msg)) if msg.contains("artifact tree digest mismatch")
    ));
}
