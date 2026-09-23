//! Conformance tests for owner-controlled acceptance inputs (OI-01).

use repo_model::Model;

/// OI-01: Owner-controlled acceptance inputs are approved, bound to cryptographic
/// evidence, and validated across identity, authority quorums, standards,
/// operational approvals, and availability bounds.
#[test]
fn owner_inputs_are_bound_and_validated_oi_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates owner inputs");

    let inputs = &model.owner_inputs;

    // 1. Identity, Legal Entity, Sites, Jurisdictions, and Assessments
    assert_eq!(inputs.organization.id, "uor:org:uor-foundation");
    assert_eq!(inputs.organization.display_name, "UOR Foundation");
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
        inputs
            .administrators
            .iter()
            .any(|a| a.mailbox == "trinity@uor.foundation"),
        "initial designated admin mailbox must be present"
    );
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
