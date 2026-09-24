//! Conformance tests for Publication SDK: source-free acquisition, readiness verification,
//! target authorization, confined atomic artifact export, live verification, and accepted-release
//! rollback semantics (PS-01).

use repo_model::{Model, PublicationError, PublicationSdkEngine, PublicationState};

/// PS-01: The publication SDK boundary enforces source-free acquisition, producer readiness
/// verification, target authorization, confined atomic artifact export, live post-deployment
/// verification, and accepted-release rollback without server proxies or byte substitution.
#[test]
fn publication_sdk_enforces_source_free_handoff_and_rollback_closure_ps_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates publication sdk");

    let cfg = &model.publication_sdk;
    assert_eq!(cfg.spec, "foundry/publication-sdk/1");
    assert_eq!(cfg.stage, "staged-core");
    assert!(cfg.policy.enforce_source_free_acquisition);
    assert!(cfg.policy.enforce_atomic_artifact_export);
    assert!(cfg.policy.require_exact_producer_binding);
    assert!(cfg.policy.require_target_authorization_decision);
    assert!(cfg.policy.prohibit_byte_substitution);
    assert!(cfg.policy.prohibit_stale_or_partial_evidence);
    assert!(cfg.policy.enable_accepted_release_rollback);

    // 1. Source-free acquisition from producer release
    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("source-free acquisition succeeds");
    assert!(pkg.source_free);
    assert_eq!(pkg.state, PublicationState::Acquired);
    assert_eq!(pkg.exported_assets.len(), 6);

    // 2. Readiness verification
    PublicationSdkEngine::verify_readiness(&mut pkg, cfg).expect("readiness verification succeeds");
    assert_eq!(pkg.state, PublicationState::ReadinessVerified);

    // 3. Target authorization
    PublicationSdkEngine::authorize_target(
        &mut pkg,
        &cfg.target_domain,
        "ed25519:council-deployment-authorization-signature",
    )
    .expect("target authorization succeeds");
    assert_eq!(pkg.state, PublicationState::TargetAuthorized);

    // 4. Confined atomic export
    let exported = PublicationSdkEngine::export_atomic_browser(
        &mut pkg,
        &model.producer_release.browser_artifacts,
        cfg,
    )
    .expect("atomic export succeeds");
    assert_eq!(pkg.state, PublicationState::Exported);
    assert_eq!(exported.len(), 6);

    // 5. Live deployment verification
    let live_assets = exported.clone();
    let state = PublicationSdkEngine::verify_live_deployment_and_rollback_on_failure(
        &mut pkg,
        &live_assets,
        &cfg.rollback_policy.retained_previous_release_digest,
    )
    .expect("live verification succeeds");
    assert_eq!(state, PublicationState::PublishedLive);
    assert_eq!(pkg.state, PublicationState::PublishedLive);
}

#[test]
fn stale_producer_binding_digest_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    pkg.binding_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();

    let res = PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk);
    assert!(matches!(
        res,
        Err(PublicationError::StaleEvidenceRejected(_))
    ));
}

#[test]
fn producer_identity_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    pkg.producer_identity = "foreign-unauthorized-producer".to_string();

    let res = PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk);
    assert!(matches!(
        res,
        Err(PublicationError::ReadinessVerificationFailed(_))
    ));
}

#[test]
fn non_https_or_empty_authorization_target_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk)
        .expect("readiness succeeds");

    // Insecure HTTP rejected
    let http_res = PublicationSdkEngine::authorize_target(
        &mut pkg,
        "http://insecure.target.domain/foundry-web/",
        "ed25519:auth-key",
    );
    assert!(matches!(
        http_res,
        Err(PublicationError::TargetNotAuthorized(_))
    ));

    // Empty authorization key rejected
    let no_key_res = PublicationSdkEngine::authorize_target(
        &mut pkg,
        "https://uor-foundation.github.io/foundry-web/",
        "",
    );
    assert!(matches!(
        no_key_res,
        Err(PublicationError::TargetNotAuthorized(_))
    ));
}

#[test]
fn byte_substitution_in_exported_artifact_is_detected_and_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk)
        .expect("readiness succeeds");
    PublicationSdkEngine::authorize_target(
        &mut pkg,
        &model.publication_sdk.target_domain,
        "ed25519:auth-key",
    )
    .expect("target authorization succeeds");

    // Tamper with foundry_bg.wasm digest in package
    if let Some(art) = pkg
        .exported_assets
        .iter_mut()
        .find(|a| a.path == "foundry_bg.wasm")
    {
        art.sha256 =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
    }

    let res = PublicationSdkEngine::export_atomic_browser(
        &mut pkg,
        &model.producer_release.browser_artifacts,
        &model.publication_sdk,
    );
    match res {
        Err(PublicationError::ByteSubstitutionDetected { file, .. }) => {
            assert_eq!(file, "foundry_bg.wasm");
        }
        other => panic!("expected ByteSubstitutionDetected for foundry_bg.wasm, got {other:?}"),
    }
}

#[test]
fn partial_export_missing_required_asset_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk)
        .expect("readiness succeeds");
    PublicationSdkEngine::authorize_target(
        &mut pkg,
        &model.publication_sdk.target_domain,
        "ed25519:auth-key",
    )
    .expect("target authorization succeeds");

    // Remove holo_runtime.holo from package
    pkg.exported_assets
        .retain(|a| a.path != "holo_runtime.holo");

    let res = PublicationSdkEngine::export_atomic_browser(
        &mut pkg,
        &model.producer_release.browser_artifacts,
        &model.publication_sdk,
    );
    assert!(matches!(
        res,
        Err(PublicationError::PartialExportRejected(_))
    ));
}

#[test]
fn live_deployment_verification_failure_triggers_automatic_rollback() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");

    let mut pkg = PublicationSdkEngine::acquire_source_free(&model.producer_release)
        .expect("acquisition succeeds");
    PublicationSdkEngine::verify_readiness(&mut pkg, &model.publication_sdk)
        .expect("readiness succeeds");
    PublicationSdkEngine::authorize_target(
        &mut pkg,
        &model.publication_sdk.target_domain,
        "ed25519:auth-key",
    )
    .expect("target authorization succeeds");
    let exported = PublicationSdkEngine::export_atomic_browser(
        &mut pkg,
        &model.producer_release.browser_artifacts,
        &model.publication_sdk,
    )
    .expect("export succeeds");

    // Corrupt one live deployed asset
    let mut corrupted_live = exported;
    if let Some(art) = corrupted_live.iter_mut().find(|a| a.path == "foundry.js") {
        art.sha256 =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();
    }

    let prev_digest = "sha256:prior-accepted-release-digest-00001";
    let res = PublicationSdkEngine::verify_live_deployment_and_rollback_on_failure(
        &mut pkg,
        &corrupted_live,
        prev_digest,
    );

    match res {
        Err(PublicationError::RollbackExecuted {
            reason,
            restored_digest,
        }) => {
            assert!(reason.contains("foundry.js"));
            assert_eq!(restored_digest, prev_digest);
            assert_eq!(pkg.state, PublicationState::RolledBack);
        }
        other => panic!("expected RollbackExecuted, got {other:?}"),
    }
}
