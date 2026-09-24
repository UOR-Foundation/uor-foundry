//! Conformance tests for organization lifecycle, activation, and scoped authority (OL-01).

use repo_model::{
    ActivateOrganizationRequest, CreateOrganizationRequest, CrossOrgAccessRequest, Model,
    OrgAdministrator, OrganizationError, OrganizationLifecycleState, OrganizationManager,
    RetireFoundingGrantRequest,
};

/// OL-01: Organization lifecycle enforces open creation, provisional creator grants,
/// policy-compliant activation, distinct-user quorum coverage, and cross-organization isolation.
#[test]
fn organization_lifecycle_enforces_creation_activation_and_isolation_ol_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model.check().expect("model check passes");

    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    // 1. Open creation: any enrolled user can create an organization with any display name
    let create_req = CreateOrganizationRequest {
        id: "uor:org:uor-foundation-alpha".to_string(),
        display_name: "UOR Foundation".to_string(),
        creator_mailbox: "trinity@uor.foundation".to_string(),
        creator_user_id: "uor:user:trinity".to_string(),
        creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
            .to_string(),
        creator_scopes: vec!["organization".to_string(), "security".to_string()],
    };
    let org = manager
        .create_organization(create_req)
        .expect("provisional creation must succeed");

    let org1_id = org.id.clone();
    assert_eq!(org1_id, "uor:org:uor-foundation-alpha");
    assert_eq!(org.display_name, "UOR Foundation");
    assert_eq!(org.state, OrganizationLifecycleState::Provisional);
    assert_eq!(org.administrators.len(), 1);
    assert_eq!(org.administrators[0].mailbox, "trinity@uor.foundation");

    // 2. Names confer no special privileges; creating a second org with the exact same display name
    let create_req2 = CreateOrganizationRequest {
        id: "uor:org:uor-foundation-beta".to_string(),
        display_name: "UOR Foundation".to_string(),
        creator_mailbox: "cypher@external.io".to_string(),
        creator_user_id: "uor:user:cypher".to_string(),
        creator_public_key: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            .to_string(),
        creator_scopes: vec!["organization".to_string()],
    };
    let org2 = manager
        .create_organization(create_req2)
        .expect("second org with same display name must succeed");
    assert_eq!(org2.display_name, "UOR Foundation");
    assert_ne!(org1_id, org2.id);

    // 3. Multi-administrator activation: requires at least 2 distinct active administrators
    // covering all required scopes ("organization", "security")
    let activate_req = ActivateOrganizationRequest {
        organization_id: "uor:org:uor-foundation-alpha".to_string(),
        administrators: vec![
            OrgAdministrator {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
            OrgAdministrator {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec![
                    "organization".to_string(),
                    "security".to_string(),
                    "operations".to_string(),
                ],
            },
            OrgAdministrator {
                mailbox: "neo@uor.foundation".to_string(),
                user_id: "uor:user:neo".to_string(),
                public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
        ],
        approving_mailboxes: vec![
            "trinity@uor.foundation".to_string(),
            "morpheus@uor.foundation".to_string(),
        ],
    };

    let activated_org = manager
        .activate_organization(activate_req, rules)
        .expect("activation with distinct administrators and scope coverage must succeed");

    assert_eq!(activated_org.state, OrganizationLifecycleState::Activated);
    assert_eq!(activated_org.administrators.len(), 3);

    // 4. Founding grant retirement: trinity can retire because morpheus and neo
    // continue to cover both "organization" and "security" scopes (2 distinct admins remaining)
    let retire_req = RetireFoundingGrantRequest {
        organization_id: "uor:org:uor-foundation-alpha".to_string(),
        founding_mailbox: "trinity@uor.foundation".to_string(),
    };
    let post_retirement = manager
        .retire_founding_grant(retire_req)
        .expect("retirement must succeed when full scope coverage is preserved");
    assert_eq!(post_retirement.administrators.len(), 2);
    assert!(!post_retirement
        .administrators
        .iter()
        .any(|a| a.mailbox == "trinity@uor.foundation"));

    // 5. Cross-organization isolation: queries and state are strictly partitioned
    let allowed_access = CrossOrgAccessRequest {
        caller_org_id: "uor:org:uor-foundation-alpha".to_string(),
        caller_mailbox: "morpheus@uor.foundation".to_string(),
        target_org_id: "uor:org:uor-foundation-alpha".to_string(),
        operation: "query_workspace".to_string(),
    };
    manager
        .verify_cross_org_access(&allowed_access)
        .expect("intra-organization request must be permitted");

    let cross_access = CrossOrgAccessRequest {
        caller_org_id: "uor:org:uor-foundation-alpha".to_string(),
        caller_mailbox: "morpheus@uor.foundation".to_string(),
        target_org_id: "uor:org:uor-foundation-beta".to_string(),
        operation: "query_workspace".to_string(),
    };
    let err = manager
        .verify_cross_org_access(&cross_access)
        .expect_err("cross-organization access must be rejected");
    assert!(matches!(
        err,
        OrganizationError::CrossOrgIsolationViolation { .. }
    ));
}

#[test]
fn activation_rejects_single_owner_bypass() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    let create_req = CreateOrganizationRequest {
        id: "uor:org:single-owner-test".to_string(),
        display_name: "Single Owner Test".to_string(),
        creator_mailbox: "alice@example.org".to_string(),
        creator_user_id: "uor:user:alice".to_string(),
        creator_public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        creator_scopes: vec!["organization".to_string(), "security".to_string()],
    };
    manager.create_organization(create_req).unwrap();

    // Attempt to activate with only 1 administrator
    let activate_req = ActivateOrganizationRequest {
        organization_id: "uor:org:single-owner-test".to_string(),
        administrators: vec![OrgAdministrator {
            mailbox: "alice@example.org".to_string(),
            user_id: "uor:user:alice".to_string(),
            public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            status: "authenticated".to_string(),
            scopes: vec!["organization".to_string(), "security".to_string()],
        }],
        approving_mailboxes: vec!["alice@example.org".to_string()],
    };

    let err = manager
        .activate_organization(activate_req, rules)
        .expect_err("must reject single-owner activation");
    assert!(matches!(
        err,
        OrganizationError::InsufficientAdministrators { .. }
    ));
}

#[test]
fn activation_rejects_duplicate_key_disguise() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    let create_req = CreateOrganizationRequest {
        id: "uor:org:dup-key-test".to_string(),
        display_name: "Dup Key Test".to_string(),
        creator_mailbox: "alice@example.org".to_string(),
        creator_user_id: "uor:user:alice".to_string(),
        creator_public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        creator_scopes: vec!["organization".to_string(), "security".to_string()],
    };
    manager.create_organization(create_req).unwrap();

    // Attempt activation with 2 mailboxes but the same public key
    let activate_req = ActivateOrganizationRequest {
        organization_id: "uor:org:dup-key-test".to_string(),
        administrators: vec![
            OrgAdministrator {
                mailbox: "alice@example.org".to_string(),
                user_id: "uor:user:alice".to_string(),
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
            OrgAdministrator {
                mailbox: "bob-alias@example.org".to_string(),
                user_id: "uor:user:bob".to_string(),
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(), // duplicate key!
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
        ],
        approving_mailboxes: vec![
            "alice@example.org".to_string(),
            "bob-alias@example.org".to_string(),
        ],
    };

    let err = manager
        .activate_organization(activate_req, rules)
        .expect_err("must reject duplicate key disguise");
    assert!(matches!(
        err,
        OrganizationError::DuplicateIdentityDisguise(_)
    ));
}

#[test]
fn activation_rejects_inadequate_scope_coverage() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    let create_req = CreateOrganizationRequest {
        id: "uor:org:scope-test".to_string(),
        display_name: "Scope Test".to_string(),
        creator_mailbox: "alice@example.org".to_string(),
        creator_user_id: "uor:user:alice".to_string(),
        creator_public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        creator_scopes: vec!["organization".to_string()],
    };
    manager.create_organization(create_req).unwrap();

    // Two distinct admins, but only one has "security" scope
    let activate_req = ActivateOrganizationRequest {
        organization_id: "uor:org:scope-test".to_string(),
        administrators: vec![
            OrgAdministrator {
                mailbox: "alice@example.org".to_string(),
                user_id: "uor:user:alice".to_string(),
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
            OrgAdministrator {
                mailbox: "bob@example.org".to_string(),
                user_id: "uor:user:bob".to_string(),
                public_key: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string()], // missing "security"
            },
        ],
        approving_mailboxes: vec![
            "alice@example.org".to_string(),
            "bob@example.org".to_string(),
        ],
    };

    let err = manager
        .activate_organization(activate_req, rules)
        .expect_err("must reject inadequate scope coverage");
    assert!(matches!(
        err,
        OrganizationError::InadequateScopeCoverage { .. }
    ));
}

#[test]
fn retirement_rejects_dropping_scope_below_quorum() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    let create_req = CreateOrganizationRequest {
        id: "uor:org:retire-fail-test".to_string(),
        display_name: "Retire Fail Test".to_string(),
        creator_mailbox: "alice@example.org".to_string(),
        creator_user_id: "uor:user:alice".to_string(),
        creator_public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_string(),
        creator_scopes: vec!["organization".to_string(), "security".to_string()],
    };
    manager.create_organization(create_req).unwrap();

    // Activate with exactly 2 admins
    let activate_req = ActivateOrganizationRequest {
        organization_id: "uor:org:retire-fail-test".to_string(),
        administrators: vec![
            OrgAdministrator {
                mailbox: "alice@example.org".to_string(),
                user_id: "uor:user:alice".to_string(),
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
            OrgAdministrator {
                mailbox: "bob@example.org".to_string(),
                user_id: "uor:user:bob".to_string(),
                public_key: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
                status: "authenticated".to_string(),
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
        ],
        approving_mailboxes: vec![
            "alice@example.org".to_string(),
            "bob@example.org".to_string(),
        ],
    };
    manager.activate_organization(activate_req, rules).unwrap();

    // Attempting to retire alice would leave only bob (1 admin), violating minimum 2
    let retire_req = RetireFoundingGrantRequest {
        organization_id: "uor:org:retire-fail-test".to_string(),
        founding_mailbox: "alice@example.org".to_string(),
    };
    let err = manager
        .retire_founding_grant(retire_req)
        .expect_err("must reject retirement that leaves fewer than 2 admins");
    assert!(matches!(
        err,
        OrganizationError::RetirementCoverageLacking(_)
    ));
}

#[test]
fn cross_org_isolation_enforces_boundaries_for_same_display_names() {
    let mut manager = OrganizationManager::new();

    // Two distinct organizations created with the SAME display name "Citizen Garden"
    let org1 = CreateOrganizationRequest {
        id: "uor:org:citizen-garden-boulder".to_string(),
        display_name: "Citizen Garden".to_string(),
        creator_mailbox: "boulder-lead@garden.org".to_string(),
        creator_user_id: "uor:user:boulder-lead".to_string(),
        creator_public_key: "1111111111111111111111111111111111111111111111111111111111111111"
            .to_string(),
        creator_scopes: vec!["organization".to_string()],
    };
    let org2 = CreateOrganizationRequest {
        id: "uor:org:citizen-garden-austin".to_string(),
        display_name: "Citizen Garden".to_string(),
        creator_mailbox: "austin-lead@garden.org".to_string(),
        creator_user_id: "uor:user:austin-lead".to_string(),
        creator_public_key: "2222222222222222222222222222222222222222222222222222222222222222"
            .to_string(),
        creator_scopes: vec!["organization".to_string()],
    };
    manager.create_organization(org1).unwrap();
    manager.create_organization(org2).unwrap();

    // Cross-organization call must fail
    let cross_req = CrossOrgAccessRequest {
        caller_org_id: "uor:org:citizen-garden-boulder".to_string(),
        caller_mailbox: "boulder-lead@garden.org".to_string(),
        target_org_id: "uor:org:citizen-garden-austin".to_string(),
        operation: "read_private_records".to_string(),
    };
    let err = manager
        .verify_cross_org_access(&cross_req)
        .expect_err("cross-org access must fail despite identical display names");
    assert!(matches!(
        err,
        OrganizationError::CrossOrgIsolationViolation { .. }
    ));
}
