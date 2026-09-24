//! Conformance tests for Producer Release: complete reproducibility verification,
//! full service/control/dependency/assessment coverage, exact producer identity,
//! artifact tree, and pre-publication evidence closure (PR-01).

use repo_model::{
    BrowserArtifactRecord, Model, ProducerReleaseConfig, ProducerReleaseEngine,
    ProducerReleaseError, ReleaseState,
};
use std::collections::HashMap;

/// PR-01: Producer release generates all artifacts twice reproducibly, verifies
/// complete service, control, dependency, and assessment coverage, and binds exact
/// producer identity, artifact tree, and pre-publication evidence with only
/// deployment-dependent checks outstanding.
#[test]
fn producer_release_generates_reproducibly_and_binds_pre_publication_evidence_pr_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates producer release");

    let cfg = &model.producer_release;
    assert_eq!(cfg.spec, "foundry/producer-release/1");
    assert_eq!(cfg.stage, "staged-core");
    assert!(cfg.policy.enforce_twice_reproducible);
    assert!(cfg.policy.require_full_service_coverage);
    assert!(cfg.policy.require_full_control_coverage);
    assert!(cfg.policy.require_full_assessment_coverage);
    assert!(cfg.policy.prohibit_draft_preview_substitutes);
    assert!(cfg.policy.prohibit_handwritten_ui_substitutes);
    assert!(cfg.policy.prohibit_empty_registers);
    assert!(cfg.policy.require_signed_pre_publication_evidence);

    // Producer identity checks
    assert_eq!(cfg.producer_identity.name, "uor-foundry-producer");
    assert_eq!(cfg.producer_identity.stage, "staged-core");
    assert!(cfg.producer_identity.commit.len() >= 7);
    assert!(cfg.producer_identity.locked_sdk_image.contains("@sha256:"));

    // Services coverage
    assert_eq!(cfg.services.len(), 7);
    let service_ids: Vec<&str> = cfg.services.iter().map(|s| s.id.as_str()).collect();
    assert!(service_ids.contains(&"uor:service:workflows"));
    assert!(service_ids.contains(&"uor:service:ai-inference"));
    assert!(service_ids.contains(&"uor:service:messaging-collaboration"));
    assert!(service_ids.contains(&"uor:service:admin-governance"));
    assert!(service_ids.contains(&"uor:service:business-finance"));
    assert!(service_ids.contains(&"uor:service:learning-certification"));
    assert!(service_ids.contains(&"uor:service:brand-presentation"));

    // Assessments coverage
    assert_eq!(cfg.covered_assessments.len(), 8);
    for assess in &cfg.covered_assessments {
        assert_eq!(assess.verdict, "conforming");
    }

    // Artifacts tree and reproducibility
    assert!(!cfg.browser_artifacts.is_empty());
    assert_eq!(
        cfg.reproducible_build.run_1_tree_digest,
        cfg.reproducible_build.run_2_tree_digest
    );
    assert_eq!(
        cfg.reproducible_build.equality_status,
        "BIT_FOR_BIT_IDENTICAL"
    );

    // Outstanding deployment checks
    assert!(!cfg.outstanding_deployment_checks.is_empty());
    for chk in &cfg.outstanding_deployment_checks {
        assert!(chk.requires_live_deployment);
        assert!(chk.target.starts_with("https://"));
    }

    // Pre-publication evidence
    assert_eq!(cfg.pre_publication_evidence.release_state, "PRODUCER_READY");
    assert!(cfg
        .pre_publication_evidence
        .binding_digest
        .starts_with("sha256:"));
    assert!(cfg
        .pre_publication_evidence
        .signature
        .starts_with("ed25519:"));
}

#[test]
fn reproducible_build_detects_digest_mismatch() {
    let run1 = vec![
        BrowserArtifactRecord {
            path: "index.html".to_string(),
            mime_type: "text/html".to_string(),
            size_bytes: 4096,
            sha256: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
        },
        BrowserArtifactRecord {
            path: "foundry.js".to_string(),
            mime_type: "application/javascript".to_string(),
            size_bytes: 81920,
            sha256: "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
        },
    ];

    let run2 = vec![
        BrowserArtifactRecord {
            path: "index.html".to_string(),
            mime_type: "text/html".to_string(),
            size_bytes: 4096,
            sha256: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
        },
        BrowserArtifactRecord {
            path: "foundry.js".to_string(),
            mime_type: "application/javascript".to_string(),
            size_bytes: 81920,
            sha256: "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                .to_string(), // Mismatched!
        },
    ];

    let res = ProducerReleaseEngine::verify_reproducibility(&run1, &run2);
    match res {
        Err(ProducerReleaseError::ReproducibilityMismatch { file, .. }) => {
            assert_eq!(file, "foundry.js");
        }
        other => panic!("expected ReproducibilityMismatch for foundry.js, got {other:?}"),
    }
}

#[test]
fn reproducible_build_detects_artifact_count_mismatch() {
    let run1 = vec![BrowserArtifactRecord {
        path: "index.html".to_string(),
        mime_type: "text/html".to_string(),
        size_bytes: 4096,
        sha256: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
    }];

    let run2 = vec![
        BrowserArtifactRecord {
            path: "index.html".to_string(),
            mime_type: "text/html".to_string(),
            size_bytes: 4096,
            sha256: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_string(),
        },
        BrowserArtifactRecord {
            path: "extra.js".to_string(),
            mime_type: "application/javascript".to_string(),
            size_bytes: 1024,
            sha256: "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                .to_string(),
        },
    ];

    let res = ProducerReleaseEngine::verify_reproducibility(&run1, &run2);
    assert!(matches!(
        res,
        Err(ProducerReleaseError::ReproducibilityMismatch { .. })
    ));
}

#[test]
fn missing_service_coverage_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut bad_release: ProducerReleaseConfig = model.producer_release.clone();
    bad_release
        .services
        .retain(|s| s.id != "uor:service:workflows");

    let res = bad_release.check(
        &model.owner_inputs,
        &model.organization_lifecycle,
        &model.services,
        &model.standards,
    );
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("uor:service:workflows"));
    assert!(err_msg.contains("missing from producer release coverage"));
}

#[test]
fn non_conforming_assessment_in_release_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut bad_release: ProducerReleaseConfig = model.producer_release.clone();
    if let Some(assess) = bad_release.covered_assessments.first_mut() {
        assess.verdict = "non-conforming".to_string();
    }

    let res = bad_release.check(
        &model.owner_inputs,
        &model.organization_lifecycle,
        &model.services,
        &model.standards,
    );
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("verdict must be 'conforming'"));
}

#[test]
fn pre_publication_check_cannot_be_deferred_to_deployment() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut bad_release: ProducerReleaseConfig = model.producer_release.clone();
    if let Some(chk) = bad_release.outstanding_deployment_checks.first_mut() {
        chk.requires_live_deployment = false; // Deferred offline check!
    }

    let res = bad_release.check(
        &model.owner_inputs,
        &model.organization_lifecycle,
        &model.services,
        &model.standards,
    );
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("pre-publication checks cannot be deferred"));
}

#[test]
fn draft_preview_cannot_enter_deployment_authorization() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut draft_release = model.producer_release.clone();
    draft_release.stage = "draft-preview".to_string();

    let res = ProducerReleaseEngine::authorize_deployment(
        &draft_release,
        "https://uor-foundation.github.io/foundry-web/",
        "sig:auth:admin-board",
    );
    assert!(matches!(
        res,
        Err(ProducerReleaseError::DraftPreviewNotAuthorized(_))
    ));
}

#[test]
fn deployment_authorization_transitions_producer_ready_to_authorized() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let auth = ProducerReleaseEngine::authorize_deployment(
        &model.producer_release,
        "https://uor-foundation.github.io/foundry-web/",
        "ed25519:auth-key-council-approval-01",
    )
    .expect("authorization succeeds");

    assert_eq!(auth.state, ReleaseState::DeploymentAuthorized);
    assert_eq!(
        auth.target_url,
        "https://uor-foundation.github.io/foundry-web/"
    );
    assert_eq!(
        auth.release_binding_digest,
        model
            .producer_release
            .pre_publication_evidence
            .binding_digest
    );
}

#[test]
fn final_acceptance_verifies_all_deployment_checks_passed() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let auth = ProducerReleaseEngine::authorize_deployment(
        &model.producer_release,
        "https://uor-foundation.github.io/foundry-web/",
        "ed25519:auth-key-council-approval-01",
    )
    .expect("authorization succeeds");

    // Partial / failing checks must be rejected
    let mut incomplete_results = HashMap::new();
    incomplete_results.insert("DEP-CHK-01".to_string(), true);
    incomplete_results.insert("DEP-CHK-02".to_string(), false); // Failed check

    let reject_res = ProducerReleaseEngine::accept_release(
        &auth,
        &model.producer_release,
        &incomplete_results,
        "sha256:live-deployment-attestation-digest",
    );
    assert!(matches!(
        reject_res,
        Err(ProducerReleaseError::DeploymentCheckFailed { .. })
    ));

    // Complete passing results
    let mut complete_results = HashMap::new();
    for chk in &model.producer_release.outstanding_deployment_checks {
        complete_results.insert(chk.id.clone(), true);
    }

    let accepted = ProducerReleaseEngine::accept_release(
        &auth,
        &model.producer_release,
        &complete_results,
        "sha256:live-deployment-attestation-digest",
    )
    .expect("acceptance succeeds");

    assert_eq!(accepted.state, ReleaseState::Accepted);
    assert_eq!(
        accepted.completed_checks.len(),
        model.producer_release.outstanding_deployment_checks.len()
    );
}
