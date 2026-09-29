//! Rejection of unsupported production authority, not product acceptance.

use repo_model::Model;

#[test]
fn empty_platform_is_valid_without_synthetic_authority() {
    let model = Model::load_from_repo_root().expect("empty platform registry parses");
    model
        .check()
        .expect("empty platform is valid configuration");
    assert!(model.owner_inputs.organizations.is_empty());
    assert!(model
        .implementation_closure
        .accepted_boundaries
        .iter()
        .all(|boundary| boundary.status == "unaccepted"));
}

#[test]
fn empty_registry_does_not_disable_email_recovery_safeguards() {
    let mut model = Model::load_from_repo_root().expect("model loads");
    model.email_continuity.protocol.replay_protection = false;
    assert!(model.check().is_err());
    model.email_continuity.protocol.replay_protection = true;
    model.email_continuity.protocol.challenge_ttl_seconds = 0;
    assert!(model.check().is_err());
}

#[test]
fn empty_registry_does_not_disable_backup_recovery_safeguards() {
    let mut model = Model::load_from_repo_root().expect("model loads");
    model.backup_codes.standards.minimum_entropy_bits = 0;
    assert!(model.check().is_err());
    model.backup_codes.standards.minimum_entropy_bits = 128;
    model.backup_codes.lifecycle.single_use = false;
    assert!(model.check().is_err());
}

#[test]
fn synthetic_configuration_never_populates_the_production_registry() {
    let fixture =
        repo_conformance::fixtures::SyntheticModel::load(&repo_model::repo_root().join("model"))
            .expect("explicit synthetic fixture parses");
    fixture
        .check()
        .expect("synthetic configuration can be unit tested");
    let platform = Model::load_from_repo_root().expect("production platform loads");
    assert!(platform.owner_inputs.organizations.is_empty());
}

#[test]
fn supplied_registry_rejects_duplicate_organization_identities() {
    let fixture =
        repo_conformance::fixtures::SyntheticModel::load(&repo_model::repo_root().join("model"))
            .expect("explicit synthetic fixture parses");
    let mut model = Model::load_from_repo_root().expect("platform loads");
    model.owner_inputs.organizations =
        vec![fixture.owner_inputs.clone(), fixture.owner_inputs.clone()];
    assert!(model.owner_inputs.check().is_err());
}
