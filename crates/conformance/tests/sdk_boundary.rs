//! Conformance tests for SDK Boundary: immutable multi-architecture OCI SDK verification,
//! complete offline dependency closure, digest-bound oracle inputs, and enforcement that
//! source integration is not consumer acceptance (SB-01).

use repo_model::{Model, SdkBoundaryEngine, SdkBoundaryError};

/// SB-01: The SDK boundary enforces immutable multi-architecture OCI SDK verification
/// across linux/amd64 and linux/arm64, complete offline dependency closure from Cargo.lock,
/// digest-bound authoritative oracle inputs, and the strict rule that source integration
/// is not consumer acceptance.
#[test]
fn sdk_boundary_verifies_multi_arch_and_offline_closure_sb_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates sdk boundary");

    let cfg = &model.sdk_boundary;
    assert_eq!(cfg.spec, "foundry/sdk-boundary/1");
    assert_eq!(cfg.stage, "staged-core");
    assert_eq!(
        cfg.sdk_image,
        "ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21"
    );
    assert_eq!(cfg.sdk_version, "0.3.0");
    assert_eq!(
        cfg.standards_lock_digest,
        "sha256:4396feba2f6ef28ed69bfccecba29398fd30c22861d30a6a2a9cde3d00b34426"
    );

    assert!(cfg.policy.require_multi_arch_manifest);
    assert!(cfg.policy.require_immutable_digest);
    assert!(cfg.policy.require_complete_offline_closure);
    assert!(cfg.policy.prohibit_wildcard_dependencies);
    assert!(cfg.policy.prohibit_unlocked_git_dependencies);
    assert!(cfg.policy.prohibit_path_dependencies_in_shipped);
    assert!(cfg.policy.require_digest_bound_oracles);
    assert!(cfg.policy.source_integration_is_not_consumer_acceptance);

    assert_eq!(cfg.architectures.len(), 2);
    let amd64_arch = cfg
        .architectures
        .iter()
        .find(|a| a.platform == "linux/amd64")
        .expect("amd64 architecture present");
    let arm64_arch = cfg
        .architectures
        .iter()
        .find(|a| a.platform == "linux/arm64")
        .expect("arm64 architecture present");

    // 1. Verify multi-architecture manifests and inventory digests
    SdkBoundaryEngine::verify_architecture_manifest(
        amd64_arch,
        "sha256:c2e0e50437e13d9b2e382d3af4ae7a962b469721b9b80f14f215d9254e8ed78f",
        "sha256:7bf8f597b532f49644e7a4c44c25eba6bdaacd6f00f15c5045abb8f0a8817d59",
    )
    .expect("amd64 architecture manifest verifies");

    SdkBoundaryEngine::verify_architecture_manifest(
        arm64_arch,
        "sha256:2f82a04e8f8141636e03a219d3960e30d98a6f304828341dd47b553b3c23baeb",
        "sha256:3b315ee041bd1d7ed2794a2aa13374b88c1fb9ada338f166fdfe743a56fed4f3",
    )
    .expect("arm64 architecture manifest verifies");

    // 2. Verify digest-bound authoritative oracles across architectures
    for oracle in &cfg.oracle_specifications {
        SdkBoundaryEngine::verify_oracle_digest(oracle, "linux/amd64", &oracle.digest_amd64)
            .expect("amd64 oracle digest matches");

        SdkBoundaryEngine::verify_oracle_digest(oracle, "linux/arm64", &oracle.digest_arm64)
            .expect("arm64 oracle digest matches");
    }

    // 3. Verify actual Cargo.lock dependency closure matches pinned model digest
    let cargo_lock_bytes =
        std::fs::read(root.join("Cargo.lock")).expect("Cargo.lock exists and is readable");
    let verified_digest =
        SdkBoundaryEngine::verify_dependency_closure(&cargo_lock_bytes, &cfg.cargo_lock_digest)
            .expect("Cargo.lock dependency closure is complete and verified");
    assert_eq!(verified_digest, cfg.cargo_lock_digest);

    // 4. Verify separation of source integration from consumer acceptance
    SdkBoundaryEngine::verify_consumer_acceptance_separation(false)
        .expect("consumer acceptance separation is preserved");
}

#[test]
fn architecture_manifest_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let amd64_arch = &model.sdk_boundary.architectures[0];

    let res = SdkBoundaryEngine::verify_architecture_manifest(
        amd64_arch,
        "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        &amd64_arch.inventory_digest,
    );
    assert!(matches!(
        res,
        Err(SdkBoundaryError::ManifestDigestMismatch { .. })
    ));
}

#[test]
fn architecture_inventory_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let amd64_arch = &model.sdk_boundary.architectures[0];

    let res = SdkBoundaryEngine::verify_architecture_manifest(
        amd64_arch,
        &amd64_arch.manifest_digest,
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );
    assert!(matches!(
        res,
        Err(SdkBoundaryError::InventoryDigestMismatch { .. })
    ));
}

#[test]
fn oracle_digest_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let oracle = &model.sdk_boundary.oracle_specifications[0];

    let res = SdkBoundaryEngine::verify_oracle_digest(
        oracle,
        "linux/amd64",
        "sha256:bad0000000000000000000000000000000000000000000000000000000000000",
    );
    assert!(matches!(
        res,
        Err(SdkBoundaryError::OracleDigestMismatch { .. })
    ));
}

#[test]
fn source_integration_alone_is_rejected_as_consumer_acceptance() {
    let res = SdkBoundaryEngine::verify_consumer_acceptance_separation(true);
    assert!(matches!(
        res,
        Err(SdkBoundaryError::SourceIntegrationNotAcceptance(_))
    ));
}

#[test]
fn cargo_lock_tampering_or_mismatch_is_rejected() {
    let tampered_bytes = b"# Tampered lockfile\n[[package]]\nname = \"insecure\"\n";
    let res = SdkBoundaryEngine::verify_dependency_closure(
        tampered_bytes,
        "sha256:21112a843436d09369282bbc8407025dd794d175bdec81d9d9ccb47733332987",
    );
    assert!(matches!(res, Err(SdkBoundaryError::Validation(_))));
}
