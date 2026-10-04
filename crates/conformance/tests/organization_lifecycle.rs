//! Conformance tests for organization lifecycle, activation, and scoped authority (OL-01).

use repo_model::{
    ActivateOrganizationRequest, CreateInvitationRequest, CreateOrganizationRequest,
    CrossOrgAccessRequest, InvitationStatus, Model, OrgAdministrator, OrganizationError,
    OrganizationLifecycleState, OrganizationManager, ProposalApproval, RetireFoundingGrantRequest,
    UpdateOrganizationRequest,
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

#[test]
fn organization_update_and_user_listing() {
    let mut manager = OrganizationManager::new();

    let create_req = CreateOrganizationRequest {
        id: "uor:org:rename-test".to_string(),
        display_name: "Original Name".to_string(),
        creator_mailbox: "admin@uor.foundation".to_string(),
        creator_user_id: "uor:user:admin".to_string(),
        creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
            .to_string(),
        creator_scopes: vec!["organization".to_string()],
    };
    manager.create_organization(create_req).unwrap();

    // 1. Rename organization
    let update_req = UpdateOrganizationRequest {
        organization_id: "uor:org:rename-test".to_string(),
        new_display_name: "Renamed Foundation".to_string(),
        requesting_mailbox: "admin@uor.foundation".to_string(),
    };
    let updated = manager
        .update_organization(update_req)
        .expect("authorized update must succeed");
    assert_eq!(updated.display_name, "Renamed Foundation");
    assert_eq!(updated.id, "uor:org:rename-test");

    // 2. Unauthorized update rejected
    let bad_update = UpdateOrganizationRequest {
        organization_id: "uor:org:rename-test".to_string(),
        new_display_name: "Hijacked Name".to_string(),
        requesting_mailbox: "stranger@external.io".to_string(),
    };
    assert!(manager.update_organization(bad_update).is_err());

    // 3. User organization listing
    let orgs = manager.list_organizations_for_user("admin@uor.foundation");
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0].id, "uor:org:rename-test");

    let empty = manager.list_organizations_for_user("other@uor.foundation");
    assert!(empty.is_empty());
}

#[test]
fn organization_suspend_reactivate_and_retire_lifecycle() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();

    // Setup active organization
    let org_id = "uor:org:lifecycle-state-test";
    manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Lifecycle State Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec!["organization".to_string(), "security".to_string()],
        })
        .unwrap();

    manager
        .activate_organization(
            ActivateOrganizationRequest {
                organization_id: org_id.to_string(),
                administrators: vec![
                    OrgAdministrator {
                        mailbox: "trinity@uor.foundation".to_string(),
                        user_id: "uor:user:trinity".to_string(),
                        public_key:
                            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec!["organization".to_string(), "security".to_string()],
                    },
                    OrgAdministrator {
                        mailbox: "morpheus@uor.foundation".to_string(),
                        user_id: "uor:user:morpheus".to_string(),
                        public_key:
                            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec!["organization".to_string(), "security".to_string()],
                    },
                ],
                approving_mailboxes: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            rules,
        )
        .unwrap();

    // 1. Suspend organization with authorized security admin
    let suspended = manager
        .suspend_organization(org_id, "trinity@uor.foundation")
        .expect("security admin can suspend org");
    assert_eq!(suspended.state, OrganizationLifecycleState::Suspended);

    let approval_trinity = ProposalApproval {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "uor:user:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 1718000000,
        signature_hex: None,
    };
    let approval_morpheus = ProposalApproval {
        mailbox: "morpheus@uor.foundation".to_string(),
        user_id: "uor:user:morpheus".to_string(),
        public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989".to_string(),
        timestamp: 1718000001,
        signature_hex: None,
    };

    // 2. Reactivate requires quorum of at least 2 distinct admins
    // Single admin attempt fails
    assert!(manager
        .reactivate_organization(org_id, std::slice::from_ref(&approval_trinity))
        .is_err());

    // Quorum of 2 succeeds
    let reactivated = manager
        .reactivate_organization(
            org_id,
            &[approval_trinity.clone(), approval_morpheus.clone()],
        )
        .expect("quorum reactivates organization");
    assert_eq!(reactivated.state, OrganizationLifecycleState::Activated);

    // 3. Retire with dependent resources check
    // Blocked when dependent resources active and force_override is false
    let err = manager
        .retire_organization(
            org_id,
            &[approval_trinity.clone(), approval_morpheus.clone()],
            false,
            3,
        )
        .expect_err("active dependent resources block retirement");
    assert!(matches!(
        err,
        OrganizationError::DependentResourcesActive(_)
    ));

    // Succeeds with force_override = true
    let retired = manager
        .retire_organization(org_id, &[approval_trinity, approval_morpheus], true, 3)
        .expect("forced retirement succeeds");
    assert_eq!(retired.state, OrganizationLifecycleState::Retired);

    // Further operations on retired organization are rejected
    assert!(manager
        .suspend_organization(org_id, "trinity@uor.foundation")
        .is_err());
}

#[test]
fn organization_invitation_lifecycle() {
    let mut manager = OrganizationManager::new();
    let org_id = "uor:org:invitation-test";

    manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Invite Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec!["organization".to_string(), "membership".to_string()],
        })
        .unwrap();

    // 1. Create invitation
    let invite_req = CreateInvitationRequest {
        id: "uor:invite:test-01".to_string(),
        organization_id: org_id.to_string(),
        target_mailbox: "neo@uor.foundation".to_string(),
        invited_scopes: vec!["security".to_string(), "operations".to_string()],
        inviter_mailbox: "trinity@uor.foundation".to_string(),
        inviter_user_id: "uor:user:trinity".to_string(),
        created_at: 1718000000,
        expires_at: 1718100000,
        revision_binding: 1,
    };
    let invite = manager
        .create_invitation(invite_req)
        .expect("creation of invitation succeeds");
    let invite_id = invite.id.clone();
    assert_eq!(invite.status, InvitationStatus::Pending);

    // 2. List invitations
    let org_invites = manager.list_invitations_for_org(org_id);
    assert_eq!(org_invites.len(), 1);
    let user_invites = manager.list_invitations_for_user("neo@uor.foundation");
    assert_eq!(user_invites.len(), 1);

    // 3. Accept invitation with recipient public key
    let neo_pk = "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4";
    let org = manager
        .accept_invitation(
            &invite_id,
            "neo@uor.foundation",
            "uor:user:neo",
            neo_pk,
            1718050000,
        )
        .expect("accepting invitation succeeds");

    // Verify recipient was added to org administrators
    let neo_admin = org
        .administrators
        .iter()
        .find(|a| a.mailbox == "neo@uor.foundation");
    assert!(neo_admin.is_some());
    assert_eq!(neo_admin.unwrap().public_key, neo_pk);

    // Verify invitation status changed to Accepted
    let updated_invite = manager.get_invitation(&invite_id).unwrap();
    assert_eq!(updated_invite.status, InvitationStatus::Accepted);

    // 4. Double accept is rejected
    assert!(manager
        .accept_invitation(
            &invite_id,
            "neo@uor.foundation",
            "uor:user:neo",
            neo_pk,
            1718050000
        )
        .is_err());

    // 5. Revocation and Decline test
    let invite_req2 = CreateInvitationRequest {
        id: "uor:invite:test-02".to_string(),
        organization_id: org_id.to_string(),
        target_mailbox: "cypher@uor.foundation".to_string(),
        invited_scopes: vec!["operations".to_string()],
        inviter_mailbox: "trinity@uor.foundation".to_string(),
        inviter_user_id: "uor:user:trinity".to_string(),
        created_at: 1718000000,
        expires_at: 1718100000,
        revision_binding: 1,
    };
    let invite2 = manager.create_invitation(invite_req2).unwrap();
    let invite2_id = invite2.id.clone();
    manager
        .decline_invitation(&invite2_id, "cypher@uor.foundation")
        .expect("declining invitation succeeds");
    let declined = manager.get_invitation(&invite2_id).unwrap();
    assert_eq!(declined.status, InvitationStatus::Declined);

    let invite_req3 = CreateInvitationRequest {
        id: "uor:invite:test-03".to_string(),
        organization_id: org_id.to_string(),
        target_mailbox: "smith@uor.foundation".to_string(),
        invited_scopes: vec!["operations".to_string()],
        inviter_mailbox: "trinity@uor.foundation".to_string(),
        inviter_user_id: "uor:user:trinity".to_string(),
        created_at: 1718000000,
        expires_at: 1718100000,
        revision_binding: 1,
    };
    let invite3 = manager.create_invitation(invite_req3).unwrap();
    let invite3_id = invite3.id.clone();
    manager
        .revoke_invitation(&invite3_id, "trinity@uor.foundation")
        .expect("revoking invitation succeeds");
    let revoked = manager.get_invitation(&invite3_id).unwrap();
    assert_eq!(revoked.status, InvitationStatus::Revoked);
}

#[test]
fn accept_invitation_rejected_for_suspended_or_retired_organization() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut manager = OrganizationManager::new();
    let org_id = "uor:org:invitation-lifecycle-guard";

    // 1. Create and activate organization with 2 admins
    manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Guard Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec!["organization".to_string(), "security".to_string()],
        })
        .unwrap();

    let activate_req = ActivateOrganizationRequest {
        organization_id: org_id.to_string(),
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
                scopes: vec!["organization".to_string(), "security".to_string()],
            },
        ],
        approving_mailboxes: vec![
            "trinity@uor.foundation".to_string(),
            "morpheus@uor.foundation".to_string(),
        ],
    };
    manager.activate_organization(activate_req, rules).unwrap();

    // 2. Issue invitation while active
    let invite = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:guard-01".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "neo@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718000000,
            expires_at: 1718100000,
            revision_binding: 1,
        })
        .unwrap();
    let invite_id = invite.id.clone();

    // 3. Suspend organization
    manager
        .suspend_organization(org_id, "trinity@uor.foundation")
        .unwrap();

    // 4. Attempt to accept invitation while suspended -> rejected
    let neo_pk = "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4";
    let err = manager
        .accept_invitation(
            &invite_id,
            "neo@uor.foundation",
            "uor:user:neo",
            neo_pk,
            1718050000,
        )
        .expect_err("accepting invitation to suspended org must fail");
    assert!(matches!(
        err,
        OrganizationError::InvalidStateTransition { .. }
    ));

    // 5. Reactivate organization with quorum
    manager
        .reactivate_organization(
            org_id,
            &[
                ProposalApproval::new(
                    "trinity@uor.foundation",
                    "uor:user:trinity",
                    "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
                    1718060000,
                ),
                ProposalApproval::new(
                    "morpheus@uor.foundation",
                    "uor:user:morpheus",
                    "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
                    1718060001,
                ),
            ],
        )
        .unwrap();

    // 6. Issue another invitation for retirement test
    let invite2 = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:guard-02".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "morpheus@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: 1718060010,
            expires_at: 1718160010,
            revision_binding: 1,
        })
        .unwrap();
    let invite2_id = invite2.id.clone();

    // 7. Retire organization
    manager
        .retire_organization(
            org_id,
            &[
                ProposalApproval::new(
                    "trinity@uor.foundation",
                    "uor:user:trinity",
                    "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be",
                    1718070000,
                ),
                ProposalApproval::new(
                    "morpheus@uor.foundation",
                    "uor:user:morpheus",
                    "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
                    1718070001,
                ),
            ],
            false,
            0,
        )
        .unwrap();

    // 8. Attempt to accept invitation while retired -> rejected
    let err2 = manager
        .accept_invitation(
            &invite2_id,
            "morpheus@uor.foundation",
            "uor:user:morpheus",
            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
            1718080000,
        )
        .expect_err("accepting invitation to retired org must fail");
    assert!(matches!(
        err2,
        OrganizationError::InvalidStateTransition { .. }
    ));
}
