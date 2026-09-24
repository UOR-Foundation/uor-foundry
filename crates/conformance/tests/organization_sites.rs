//! Conformance tests for organization sites lifecycle, activation quorums,
//! assessments, and cross-organization isolation (OS-01).

use repo_model::{
    AccessSiteRequest, ActivateSiteRequest, CreateSiteRequest, Model, OrganizationLifecycleState,
    SiteError, SiteLifecycleState, SiteManager, TransitionSiteRequest,
};

/// OS-01: The organization and sites boundary enforces complete lifecycle management,
/// multi-admin activation quorums, physical and accessibility assessments, and
/// cross-organization isolation without seeded privileges.
#[test]
fn organization_sites_enforce_lifecycle_quorums_assessments_and_isolation_os_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates organization sites");

    let cfg = &model.organization_sites;
    let mut mgr = SiteManager::new(cfg);

    // 1. Verify Foundation HQ and Citizen Gardens Boulder are active
    let hq = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: "uor:site:hq-foundry-01".to_string(),
        })
        .expect("access HQ site");
    assert_eq!(hq.state, SiteLifecycleState::Active);
    assert_eq!(hq.organization_id, "uor:org:uor-foundation");

    let boulder = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: "uor:site:citizen-gardens-boulder".to_string(),
        })
        .expect("access Citizen Gardens Boulder site");
    assert_eq!(boulder.state, SiteLifecycleState::Active);

    // 2. Citizen Garden Community Foundry Beta is initially Provisional; activate it through normal workflow
    let cg_beta = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: "uor:site:citizen-garden-02".to_string(),
        })
        .expect("access Citizen Garden Beta site");
    assert_eq!(cg_beta.state, SiteLifecycleState::Provisional);

    // Assign a second administrator to satisfy 2-of-N quorum
    mgr.assign_administrator("uor:site:citizen-garden-02", "morpheus@uor.foundation")
        .expect("assign second admin");

    // Activate Citizen Garden Beta with distinct admin approvals
    mgr.activate_site(
        ActivateSiteRequest {
            site_id: "uor:site:citizen-garden-02".to_string(),
            organization_id: "uor:org:uor-foundation".to_string(),
            approving_administrators: vec![
                "trinity@uor.foundation".to_string(),
                "morpheus@uor.foundation".to_string(),
            ],
        },
        OrganizationLifecycleState::Activated,
    )
    .expect("activate Citizen Garden Beta site");

    let cg_beta_active = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: "uor:site:citizen-garden-02".to_string(),
        })
        .expect("access activated Citizen Garden Beta");
    assert_eq!(cg_beta_active.state, SiteLifecycleState::Active);
}

#[test]
fn site_activation_rejects_unactivated_parent_organization() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut mgr = SiteManager::new(&model.organization_sites);

    mgr.assign_administrator("uor:site:citizen-garden-02", "morpheus@uor.foundation")
        .expect("assign second admin");

    // Attempt activation while parent org is Provisional
    let err = mgr
        .activate_site(
            ActivateSiteRequest {
                site_id: "uor:site:citizen-garden-02".to_string(),
                organization_id: "uor:org:uor-foundation".to_string(),
                approving_administrators: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            OrganizationLifecycleState::Provisional,
        )
        .expect_err("must reject activating site under provisional organization");

    assert!(
        matches!(err, SiteError::ParentOrganizationNotActive { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn site_activation_rejects_single_administrator_approval() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut mgr = SiteManager::new(&model.organization_sites);

    // Attempt activation with only 1 administrator
    let err = mgr
        .activate_site(
            ActivateSiteRequest {
                site_id: "uor:site:citizen-garden-02".to_string(),
                organization_id: "uor:org:uor-foundation".to_string(),
                approving_administrators: vec!["trinity@uor.foundation".to_string()],
            },
            OrganizationLifecycleState::Activated,
        )
        .expect_err("must reject single administrator approval");

    assert!(
        matches!(err, SiteError::InsufficientApprovers { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn site_activation_rejects_duplicate_approving_administrator() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut mgr = SiteManager::new(&model.organization_sites);

    // Attempt activation with duplicate admin disguised as quorum
    let err = mgr
        .activate_site(
            ActivateSiteRequest {
                site_id: "uor:site:citizen-garden-02".to_string(),
                organization_id: "uor:org:uor-foundation".to_string(),
                approving_administrators: vec![
                    "trinity@uor.foundation".to_string(),
                    "trinity@uor.foundation".to_string(),
                ],
            },
            OrganizationLifecycleState::Activated,
        )
        .expect_err("must reject duplicate administrator approval");

    assert!(
        matches!(err, SiteError::DuplicateApprover { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn site_activation_rejects_missing_physical_security_assessment() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut cfg = model.organization_sites.clone();

    // Remove physical security assessment for Beta
    cfg.assessments
        .retain(|a| a.id != "uor:assessment:site:cg-austin-phys-sec");

    let mut mgr = SiteManager::new(&cfg);
    mgr.assign_administrator("uor:site:citizen-garden-02", "morpheus@uor.foundation")
        .expect("assign second admin");

    let err = mgr
        .activate_site(
            ActivateSiteRequest {
                site_id: "uor:site:citizen-garden-02".to_string(),
                organization_id: "uor:org:uor-foundation".to_string(),
                approving_administrators: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            OrganizationLifecycleState::Activated,
        )
        .expect_err("must reject missing physical assessment");

    assert!(
        matches!(err, SiteError::MissingPhysicalSecurityAssessment(..)),
        "unexpected error: {err}"
    );
}

#[test]
fn site_activation_rejects_missing_accessibility_assessment() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut cfg = model.organization_sites.clone();

    // Remove accessibility assessment for Beta
    cfg.assessments
        .retain(|a| a.id != "uor:assessment:site:cg-austin-accessibility");

    let mut mgr = SiteManager::new(&cfg);
    mgr.assign_administrator("uor:site:citizen-garden-02", "morpheus@uor.foundation")
        .expect("assign second admin");

    let err = mgr
        .activate_site(
            ActivateSiteRequest {
                site_id: "uor:site:citizen-garden-02".to_string(),
                organization_id: "uor:org:uor-foundation".to_string(),
                approving_administrators: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            OrganizationLifecycleState::Activated,
        )
        .expect_err("must reject missing accessibility assessment");

    assert!(
        matches!(err, SiteError::MissingAccessibilityAssessment(..)),
        "unexpected error: {err}"
    );
}

#[test]
fn cross_org_site_access_is_rejected() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mgr = SiteManager::new(&model.organization_sites);

    // Alien org attempting to access UOR Foundation HQ
    let err = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:unauthorized-third-party".to_string(),
            site_id: "uor:site:hq-foundry-01".to_string(),
        })
        .expect_err("must reject foreign organization access");

    assert!(
        matches!(err, SiteError::CrossOrgIsolationViolation { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn duplicate_site_names_in_different_orgs_are_isolated() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut mgr = SiteManager::new(&model.organization_sites);

    // Create an independent site with the SAME name "Foundation HQ & First Foundry" under a different org
    let alien_site_id = mgr
        .create_site(CreateSiteRequest {
            id: "uor:site:alien-hq".to_string(),
            organization_id: "uor:org:independent-entity".to_string(),
            name: "Foundation HQ & First Foundry".to_string(),
            site_type: "headquarters".to_string(),
            address: "999 Alien St, New York, NY".to_string(),
            jurisdiction: "United States (New York)".to_string(),
            creator_mailbox: "admin@independent.org".to_string(),
        })
        .expect("independent site creates successfully");

    // Foreign org cannot access UOR's HQ
    let err = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:independent-entity".to_string(),
            site_id: "uor:site:hq-foundry-01".to_string(),
        })
        .expect_err("foreign org cannot access real UOR HQ");
    assert!(matches!(err, SiteError::CrossOrgIsolationViolation { .. }));

    // Real UOR Foundation cannot access alien site
    let err2 = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: alien_site_id.clone(),
        })
        .expect_err("UOR org cannot access alien site");
    assert!(matches!(err2, SiteError::CrossOrgIsolationViolation { .. }));
}

#[test]
fn site_transitions_enforce_valid_state_machine_and_quorums() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let mut mgr = SiteManager::new(&model.organization_sites);

    // 1. Transition Active -> Maintenance requires 2 distinct admins
    mgr.transition_site(TransitionSiteRequest {
        site_id: "uor:site:hq-foundry-01".to_string(),
        organization_id: "uor:org:uor-foundation".to_string(),
        target_state: SiteLifecycleState::Maintenance,
        approving_administrators: vec![
            "trinity@uor.foundation".to_string(),
            "morpheus@uor.foundation".to_string(),
        ],
    })
    .expect("transition to maintenance");

    let hq = mgr
        .access_site_records(AccessSiteRequest {
            caller_org_id: "uor:org:uor-foundation".to_string(),
            site_id: "uor:site:hq-foundry-01".to_string(),
        })
        .expect("access HQ");
    assert_eq!(hq.state, SiteLifecycleState::Maintenance);

    // 2. Single admin cannot transition Maintenance -> Active
    let err = mgr
        .transition_site(TransitionSiteRequest {
            site_id: "uor:site:hq-foundry-01".to_string(),
            organization_id: "uor:org:uor-foundation".to_string(),
            target_state: SiteLifecycleState::Active,
            approving_administrators: vec!["trinity@uor.foundation".to_string()],
        })
        .expect_err("must reject single admin transition");
    assert!(matches!(err, SiteError::InsufficientApprovers { .. }));
}
