//! Synthetic OI-01 configuration unit checks, not authenticated authority evidence.
//! Full product and organization-admission acceptance remain required.

use repo_conformance::fixtures::SyntheticModel as Model;

/// OI-01 prerequisite: synthetic fields can exercise configuration validation.
/// Digest strings and asserted approvals do not authenticate the modeled facts.
#[test]
fn synthetic_owner_configuration_shape_is_not_authority_evidence_oi_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates owner inputs");

    let inputs = &model.owner_inputs;

    // 1. Identity, Legal Entity, Sites, Jurisdictions, and Assessments
    assert!(!inputs.organization.id.is_empty());
    assert!(!inputs.organization.display_name.is_empty());
    assert_eq!(inputs.legal_entity.statutory_filings_status, "approved");
    assert!(inputs.legal_entity.charter_digest.starts_with("sha256:"));
    assert!(!inputs.sites.is_empty(), "sites must not be empty");
    assert!(
        !inputs.site_assessments.is_empty(),
        "site assessments must not be empty"
    );
    for assessment in &inputs.site_assessments {
        assert_eq!(assessment.result, "conforming");
        assert!(assessment.evidence_digest.starts_with("sha256:"));
    }

    // 2. Authenticated Admin Keys, Membership Admission, Activation Policy, Quorums, Recovery Rules
    assert!(
        inputs.administrators.len() >= inputs.activation_policy.minimum_active_administrators,
        "must have at least minimum active administrators for redundancy"
    );
    assert!(inputs.activation_policy.prohibit_single_owner_bypass);
    assert_eq!(
        inputs.membership_policy.faculty_admission_approver,
        "enrolled-organization-administrator"
    );
    assert!(inputs.membership_policy.prohibit_self_appointment);
    assert!(inputs.recovery_rules.email_challenge_replay_protection);
    assert!(inputs.recovery_rules.backup_code_single_use);
    assert!(inputs.recovery_rules.session_invalidation_on_recovery);
    assert!(inputs.recovery_rules.reject_revoked_grant_recovery);

    // 3. Adopted Standards, Editions, Normative-Source Rights, Assessment Authorities
    assert!(!inputs.standards.is_empty(), "standards must not be empty");
    assert!(
        !inputs.assessment_authorities.is_empty(),
        "assessment authorities must not be empty"
    );

    // 4. Business, Operational, and Publication Approvals; Payment and Certification Scope Rules
    assert_eq!(inputs.business_operations.status, "approved");
    assert!(inputs.publication_approvals.staged_core_authorized);
    assert!(
        !inputs.publication_approvals.preview_publication_authorized,
        "preview must NOT be authorized"
    );
    assert!(inputs
        .publication_approvals
        .approved_targets
        .contains(&"https://uor-foundation.github.io/foundry-web/".to_string()));
    assert!(
        inputs
            .payment_certification_rules
            .settlement_verification_required
    );
    assert!(
        inputs
            .payment_certification_rules
            .prohibit_speculative_trading
    );

    // 5. Availability, Workload, and Fault Bounds; RPO/RTO; Retention and Replica Obligations
    assert_eq!(inputs.resilience_bounds.rpo_local_committed_seconds, 0);
    assert_eq!(inputs.resilience_bounds.rpo_replicated_state_seconds, 0);
    assert!(inputs.resilience_bounds.rto_local_session_seconds <= 5);
    assert!(inputs.resilience_bounds.rto_peer_reconciliation_seconds <= 30);
    assert!(inputs.resilience_bounds.min_independent_peer_replicas >= 2);
    assert!(inputs.resilience_bounds.audit_retention_years >= 7);
}

#[test]
fn owner_inputs_reject_single_owner_bypass() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: retain only one administrator
    model.owner_inputs.administrators.truncate(1);
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject single administrator");
    assert!(
        err.to_string()
            .contains("less than minimum active administrators"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_reject_unauthorized_preview_publication() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: authorize preview publication
    model
        .owner_inputs
        .publication_approvals
        .preview_publication_authorized = true;
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject unauthorized preview publication");
    assert!(
        err.to_string()
            .contains("preview publication is not authorized"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_reject_missing_site_assessment() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: clear site assessments
    model.owner_inputs.site_assessments.clear();
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject empty site assessments");
    assert!(
        err.to_string()
            .contains("must define at least one site assessment"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_reject_insufficient_replicas() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: min independent peer replicas < 2
    model
        .owner_inputs
        .resilience_bounds
        .min_independent_peer_replicas = 1;
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject insufficient peer replicas");
    assert!(
        err.to_string()
            .contains("min_independent_peer_replicas must be at least 2"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_reject_unapproved_statutory_filings() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: set statutory filings status to provisional
    model.owner_inputs.legal_entity.statutory_filings_status = "provisional".to_string();
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject unapproved statutory filings");
    assert!(
        err.to_string()
            .contains("legal_entity statutory filings must be approved"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_reject_insecure_backup_code_entropy() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: set entropy bits < 128
    model.owner_inputs.recovery_rules.backup_code_entropy_bits = 64;
    let err = model
        .owner_inputs
        .check()
        .expect_err("must reject weak backup code entropy");
    assert!(
        err.to_string()
            .contains("backup_code_entropy_bits must be at least 128 bits"),
        "unexpected diagnostic: {err}"
    );
}

#[test]
fn owner_inputs_signed_attestation_verification_and_integrity() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let inputs = &model.owner_inputs;

    // 1. Valid charter attestation signed by trinity (has organization scope)
    let valid_charter = repo_model::SignedOwnerAttestation {
        attestation_id: "att-charter-01".to_string(),
        organization_id: "uor:org:uor-foundation".to_string(),
        attestation_type: "charter".to_string(),
        document_digest: "sha256:7379c7314f6e635202e221f787737531eefe1ac28cbbf034d04e490e769e6d41".to_string(),
        signer_mailbox: "trinity@uor.foundation".to_string(),
        signer_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        signature_hex: "304402202b8d00938f45a6c4ef76921319c5932560ef7b8ec5d1b7d5ca4c1e4c7304f29102200259b64ea32313d3957eb64ea32313d3957eb64ea32313d3957eb64ea32313d3".to_string(),
        timestamp: 1718000000,
        valid_until: "2027-09-01".to_string(),
    };
    inputs
        .verify_attestation(&valid_charter)
        .expect("valid charter attestation must verify");

    // 2. Valid site assessment attestation signed by morpheus (has security scope)
    let valid_site = repo_model::SignedOwnerAttestation {
        attestation_id: "att-site-01".to_string(),
        organization_id: "uor:org:uor-foundation".to_string(),
        attestation_type: "site-assessment".to_string(),
        document_digest: "sha256:b2dffcef8d86cb92b457f6d4c284c018f126b3b3289c76270f6b23e362aee081".to_string(),
        signer_mailbox: "morpheus@uor.foundation".to_string(),
        signer_public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989".to_string(),
        signature_hex: "0fc51da010ba370ad7cc9f62bb501189f93b41519bd2845ccfcbe1015d6b3ef09f5c8659c8909de26b67dcf7e07d798b30f36b4273944383d1aa8615722652da".to_string(),
        timestamp: 1718000000,
        valid_until: "2027-09-01".to_string(),
    };
    inputs
        .verify_attestation(&valid_site)
        .expect("valid site assessment attestation must verify");

    // 3. Organization ID mismatch rejected
    let mut bad_org = valid_charter.clone();
    bad_org.organization_id = "uor:org:other-foundation".to_string();
    assert!(inputs.verify_attestation(&bad_org).is_err());

    // 4. Digest mismatch rejected
    let mut bad_digest = valid_charter.clone();
    bad_digest.document_digest =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string();
    assert!(inputs.verify_attestation(&bad_digest).is_err());

    // 5. Unenrolled signer rejected
    let mut bad_signer = valid_charter.clone();
    bad_signer.signer_mailbox = "intruder@evil.org".to_string();
    assert!(inputs.verify_attestation(&bad_signer).is_err());

    // 6. Signer key mismatch rejected
    let mut bad_key = valid_charter.clone();
    bad_key.signer_public_key =
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
    assert!(inputs.verify_attestation(&bad_key).is_err());

    // 7. Missing required scope rejected (neo has no security scope for site-assessment)
    let mut bad_scope = valid_site.clone();
    bad_scope.signer_mailbox = "neo@uor.foundation".to_string();
    bad_scope.signer_public_key =
        "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4".to_string();
    assert!(inputs.verify_attestation(&bad_scope).is_err());

    // 8. Zero / malformed signature rejected
    let mut bad_sig = valid_charter.clone();
    bad_sig.signature_hex = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string();
    assert!(inputs.verify_attestation(&bad_sig).is_err());

    // 9. Dummy repetitive signature rejected
    let mut dummy_rep_sig = valid_site.clone();
    dummy_rep_sig.signature_hex = "112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00".to_string();
    let err = inputs
        .verify_attestation(&dummy_rep_sig)
        .expect_err("dummy repetitive signature must be rejected");
    assert!(err.to_string().contains("dummy repetitive pattern"));

    // 10. Scalar exceeding secp256r1 curve order rejected
    let mut out_of_bounds_sig = valid_site.clone();
    // Use r >= n: FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632552
    out_of_bounds_sig.signature_hex = "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc6325520fc51da010ba370ad7cc9f62bb501189f93b41519bd2845ccfcbe1015d6b3ef0".to_string();
    let err = inputs
        .verify_attestation(&out_of_bounds_sig)
        .expect_err("out of bounds curve order scalar must be rejected");
    assert!(err.to_string().contains("exceeds secp256r1 curve order"));

    // 11. Identical r and s scalars rejected
    let mut identical_rs_sig = valid_site.clone();
    identical_rs_sig.signature_hex = "0fc51da010ba370ad7cc9f62bb501189f93b41519bd2845ccfcbe1015d6b3ef00fc51da010ba370ad7cc9f62bb501189f93b41519bd2845ccfcbe1015d6b3ef0".to_string();
    let err = inputs
        .verify_attestation(&identical_rs_sig)
        .expect_err("identical r and s scalars must be rejected");
    assert!(err.to_string().contains("identical r and s"));
}
