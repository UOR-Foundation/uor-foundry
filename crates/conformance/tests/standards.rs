//! Conformance tests for standards, OSCAL governance, and authenticated assessments (ST-01).

use repo_model::{Model, StandardsError};

/// ST-01: The standards boundary implements complete OSCAL catalogs, profile
/// resolution, component and system implementation records, inheritance tracking,
/// and authenticated assessment coverage across all adopted standards.
#[test]
fn standards_boundary_implements_oscal_and_authenticated_assessments_st_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates standards OSCAL boundary");

    let standards = &model.standards;
    let inputs = &model.owner_inputs;

    // 1. Resolve production profile
    let profile = standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("production profile resolves");

    assert_eq!(profile.profile_id, "PROF-FOUNDRY-PRODUCTION");
    assert!(
        !profile.controls.is_empty(),
        "resolved controls must not be empty"
    );

    // Ensure all mandatory base profile controls are resolved
    assert!(
        profile.controls.iter().any(|c| c.id == "PRISM-BASE-01"),
        "PRISM-BASE-01 must be present"
    );
    assert!(
        profile.controls.iter().any(|c| c.id == "PRISM-BASE-02"),
        "PRISM-BASE-02 must be present"
    );
    assert!(
        profile.controls.iter().any(|c| c.id == "PRISM-BASE-03"),
        "PRISM-BASE-03 must be present"
    );

    // 2. Validate system implementation records and inheritance
    standards
        .validate_system("SYS-UOR-FOUNDRY", &profile)
        .expect("system implementation validates");

    let sys = standards
        .systems
        .iter()
        .find(|s| s.system_id == "SYS-UOR-FOUNDRY")
        .expect("system record exists");

    assert!(
        !sys.inherited_controls.is_empty(),
        "inherited controls must be present"
    );
    for inh in &sys.inherited_controls {
        assert!(!inh.provider_component_id.is_empty());
        assert!(!inh.provider_organization.is_empty());
        assert!(!inh.provider_scope.is_empty());
        assert!(!inh.exact_subject_and_revision.is_empty());
        assert!(inh.evidence_digest.starts_with("sha256:"));
        assert!(!inh.validity_conditions.is_empty());
        assert!(!inh.consumer_responsibilities.is_empty());
    }

    // 3. Verify authenticated assessments
    let report = standards
        .verify_assessments(
            &profile,
            &inputs.assessment_authorities,
            &inputs.standards,
            &inputs.organization.id,
        )
        .expect("assessments verify");

    assert_eq!(report.total_controls, profile.controls.len());
    assert_eq!(report.conforming_controls, profile.controls.len());
    assert!(
        report
            .authorities_cited
            .contains(&"AUTH-UOR-SEC".to_string()),
        "UOR-SEC authority must be cited"
    );
    assert!(
        report
            .authorities_cited
            .contains(&"AUTH-ISO-IEC".to_string()),
        "ISO-IEC authority must be cited"
    );
    assert!(
        report.authorities_cited.contains(&"AUTH-NIST".to_string()),
        "NIST authority must be cited"
    );
}

#[test]
fn weakened_base_profile_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: attempt to exclude mandatory base profile control
    let prof = model
        .standards
        .profiles
        .iter_mut()
        .find(|p| p.profile_id == "PROF-FOUNDRY-PRODUCTION")
        .expect("profile exists");
    prof.excluded_controls.push("PRISM-BASE-01".to_string());

    let err = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect_err("must reject excluding base profile control");

    assert!(
        matches!(err, StandardsError::BaseProfileWeakened { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn unsatisfied_control_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: add a fictitious control to catalog and profile without implementation
    let cat = model
        .standards
        .catalogs
        .iter_mut()
        .find(|c| c.catalog_id == "CAT-PRISM-BASE")
        .expect("catalog exists");
    cat.controls.push(repo_model::OscalControl {
        id: "PRISM-BASE-99".to_string(),
        title: "Unimplemented Control".to_string(),
        statement: "Must fail satisfaction check.".to_string(),
    });

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .validate_system("SYS-UOR-FOUNDRY", &profile)
        .expect_err("must reject unsatisfied control");

    assert!(
        matches!(err, StandardsError::UnsatisfiedControl { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn incomplete_inheritance_contract_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: clear consumer responsibilities from inherited control
    let sys = model
        .standards
        .systems
        .iter_mut()
        .find(|s| s.system_id == "SYS-UOR-FOUNDRY")
        .expect("system exists");
    sys.inherited_controls[0].consumer_responsibilities.clear();

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .validate_system("SYS-UOR-FOUNDRY", &profile)
        .expect_err("must reject incomplete inheritance contract");

    assert!(
        matches!(err, StandardsError::IncompleteInheritance { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn unauthorized_assessor_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: forge assessor authority
    model.standards.assessments[0].assessor_authority_id = "AUTH-ROGUE-ENTITY".to_string();

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("must reject unauthorized assessor");

    assert!(
        matches!(err, StandardsError::AssessorNotAuthorized { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn assessor_key_mismatch_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: substitute verification key
    model.standards.assessments[0].assessor_signature_key =
        "SHA256:FORGED-KEY-00000000000".to_string();

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("must reject mismatched key");

    assert!(
        matches!(err, StandardsError::AssessorKeyMismatch { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn non_conforming_assessment_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: non-conforming verdict
    model.standards.assessments[0].verdict = "non-conforming".to_string();

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("must reject non-conforming verdict");

    assert!(
        matches!(err, StandardsError::NonConformingAssessment { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn unassessed_control_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: remove control from assessment coverage
    model.standards.assessments[0]
        .assessed_controls
        .retain(|c| c != "PRISM-BASE-01");

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("must reject unassessed control");

    assert!(
        matches!(err, StandardsError::UnassessedControl { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn cross_org_assessment_isolation_is_enforced() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: attribute assessment to another organization
    model.standards.assessments[0].organization_id = "uor:org:other-entity".to_string();

    let profile = model
        .standards
        .resolve_profile("PROF-FOUNDRY-PRODUCTION")
        .expect("profile resolves");

    let err = model
        .standards
        .verify_assessments(
            &profile,
            &model.owner_inputs.assessment_authorities,
            &model.owner_inputs.standards,
            &model.owner_inputs.organization.id,
        )
        .expect_err("must reject cross-org assessment");

    assert!(
        matches!(err, StandardsError::CrossOrgIsolationViolation { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn uncataloged_adopted_standard_is_rejected() {
    let root = repo_model::repo_root();
    let mut model = Model::load(&root.join("model")).expect("model loads");

    // Plant defect: remove catalog for an adopted standard
    model
        .standards
        .catalogs
        .retain(|c| c.standard_id != "ISO-27034-1-2011");

    let err = model
        .standards
        .check(&model.owner_inputs)
        .expect_err("must reject uncataloged adopted standard");

    assert!(
        err.to_string().contains("missing an OSCAL catalog"),
        "unexpected diagnostic: {err}"
    );
}
