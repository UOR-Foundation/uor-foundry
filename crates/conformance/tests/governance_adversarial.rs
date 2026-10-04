//! Adversarial stress tests for Milestone 2: Governance, Invitation Lifecycle, Proposal Approvals, and Atomic Continuity.

use repo_conformance::fixtures::SyntheticModel;
use repo_model::authority::{
    AuthorityAction, AuthorityError, AuthorityManager, ChangeProposal, ProposalApproval,
    ProposalStatus,
};
use repo_model::organization::{
    ActivateOrganizationRequest, CreateInvitationRequest, CreateOrganizationRequest,
    InvitationStatus, OrgAdministrator, OrganizationError, OrganizationLifecycleState,
    OrganizationManager,
};
use repo_model::{Model, SignedOwnerAttestation};

fn test_setup() -> (AuthorityManager, String, Model) {
    let model = Model::load_from_repo_root().expect("model must load and check cleanly");
    model.check().expect("model consistency check must pass");

    let manager = AuthorityManager::new(model.authority.clone());
    let org_id = "uor:org:uor-foundation".to_string();
    (manager, org_id, model)
}

fn sample_admins() -> Vec<OrgAdministrator> {
    vec![
        OrgAdministrator {
            mailbox: "trinity@uor.foundation".to_string(),
            user_id: "uor:user:trinity".to_string(),
            public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            status: "authenticated".to_string(),
            scopes: vec![
                "organization".to_string(),
                "security".to_string(),
                "operations".to_string(),
                "releases".to_string(),
                "certification".to_string(),
                "membership".to_string(),
            ],
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
                "releases".to_string(),
                "certification".to_string(),
                "membership".to_string(),
            ],
        },
    ]
}

// ============================================================================
// PART 1: REACTIVATION QUORUM ENFORCEMENT
// ============================================================================

#[test]
fn adversarial_reactivation_requires_distinct_admin_quorum() {
    let (_manager, org_id, model) = test_setup();
    let rules = &model.organization_lifecycle.rules;
    let mut org_manager = OrganizationManager::new();

    // 1. Create and activate organization
    org_manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.clone(),
            display_name: "Adversarial Quorum Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec!["organization".to_string(), "security".to_string()],
        })
        .unwrap();

    let admins = sample_admins();
    org_manager
        .activate_organization(
            ActivateOrganizationRequest {
                organization_id: org_id.clone(),
                administrators: admins.clone(),
                approving_mailboxes: vec![
                    "trinity@uor.foundation".to_string(),
                    "morpheus@uor.foundation".to_string(),
                ],
            },
            rules,
        )
        .unwrap();

    // Suspend organization
    let suspended = org_manager
        .suspend_organization(&org_id, "trinity@uor.foundation")
        .expect("authorized suspension must succeed");
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

    // ATTACK 1.1: Single-admin reactivation attempt
    let err = org_manager
        .reactivate_organization(&org_id, std::slice::from_ref(&approval_trinity))
        .expect_err("single-admin reactivation must fail");
    assert!(matches!(
        err,
        OrganizationError::SingleOwnerBypassRejected(_)
    ));

    // ATTACK 1.2: Duplicate mailbox reactivation attempt (same admin twice)
    let err = org_manager
        .reactivate_organization(
            &org_id,
            &[approval_trinity.clone(), approval_trinity.clone()],
        )
        .expect_err("duplicate mailbox approval must fail");
    assert!(matches!(
        err,
        OrganizationError::DuplicateIdentityDisguise(_)
    ));

    // ATTACK 1.3: Duplicate public key with different mailbox (sybil disguise)
    let fake_sybil_approval = ProposalApproval {
        mailbox: "sybil@uor.foundation".to_string(),
        user_id: "uor:user:sybil".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 1718000002,
        signature_hex: None,
    };
    let err = org_manager
        .reactivate_organization(&org_id, &[approval_trinity.clone(), fake_sybil_approval])
        .expect_err("sybil key disguise or unenrolled approver must fail");
    assert!(matches!(
        err,
        OrganizationError::UnauthorizedRoleElevation(_)
            | OrganizationError::DuplicateIdentityDisguise(_)
    ));

    // ATTACK 1.4: Unenrolled administrator approval
    let stranger_approval = ProposalApproval {
        mailbox: "stranger@external.io".to_string(),
        user_id: "uor:user:stranger".to_string(),
        public_key: "1122334455667788990011223344556677889900112233445566778899001122".to_string(),
        timestamp: 1718000003,
        signature_hex: None,
    };
    let err = org_manager
        .reactivate_organization(&org_id, &[approval_trinity.clone(), stranger_approval])
        .expect_err("stranger approval must fail");
    assert!(matches!(
        err,
        OrganizationError::UnauthorizedRoleElevation(_)
    ));

    // ATTACK 1.5: Zero approvers
    let err = org_manager
        .reactivate_organization(&org_id, &[])
        .expect_err("empty approvers list must fail");
    assert!(matches!(
        err,
        OrganizationError::SingleOwnerBypassRejected(_)
    ));

    // VERIFY: Organization remains strictly Suspended after all failed attacks
    let org = org_manager.get(&org_id).unwrap();
    assert_eq!(org.state, OrganizationLifecycleState::Suspended);

    // LEGITIMATE: Distinct-admin quorum (trinity + morpheus) succeeds
    let reactivated = org_manager
        .reactivate_organization(&org_id, &[approval_trinity, approval_morpheus])
        .expect("distinct quorum must reactivate org");
    assert_eq!(reactivated.state, OrganizationLifecycleState::Activated);

    // ATTACK 1.6: Reactivating an already Activated organization
    let err = org_manager
        .reactivate_organization(
            &org_id,
            &[
                ProposalApproval {
                    mailbox: "trinity@uor.foundation".to_string(),
                    user_id: "uor:user:trinity".to_string(),
                    public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                        .to_string(),
                    timestamp: 1718000010,
                    signature_hex: None,
                },
                ProposalApproval {
                    mailbox: "morpheus@uor.foundation".to_string(),
                    user_id: "uor:user:morpheus".to_string(),
                    public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                        .to_string(),
                    timestamp: 1718000011,
                    signature_hex: None,
                },
            ],
        )
        .expect_err("reactivating already active org must fail");
    assert!(matches!(
        err,
        OrganizationError::InvalidStateTransition { .. }
    ));
}

// ============================================================================
// PART 2: ATOMIC CONTINUITY VIOLATION ATTACKS
// ============================================================================

#[test]
fn adversarial_atomic_continuity_rejects_dropping_below_quorum() {
    let (mut manager, org_id, _) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    let initial_rev = manager.records.get(&org_id).unwrap().revision;
    assert_eq!(initial_rev, 1);

    // Currently: 2 admins (trinity, morpheus) covering "organization" and "security".
    // Quorum rule requires minimum 2 distinct administrators for both scopes.

    // ATTACK 2.1: Revoking "security" scope from morpheus would leave only trinity (1 admin) in "security"
    let prop_revoke_sec = ChangeProposal {
        proposal_id: "prop-attack-revoke-sec".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::RevokeScope {
            mailbox: "morpheus@uor.foundation".to_string(),
            scope: "security".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 100,
                signature_hex: None,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 101,
                signature_hex: None,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop_revoke_sec).unwrap();
    let err = manager
        .execute_proposal("prop-attack-revoke-sec")
        .expect_err("proposal leaving < 2 admins in 'security' must be atomically rejected");
    assert!(matches!(
        err,
        AuthorityError::PostChangeCoverageDeficit { ref scope, remaining: 1, required: 2 } if scope == "security"
    ));

    // VERIFY: State is completely unchanged and revision is strictly preserved
    let record = manager.records.get(&org_id).unwrap();
    assert_eq!(record.revision, initial_rev);
    let sec_admins = AuthorityManager::distinct_administrators_for_scope(record, "security");
    assert_eq!(
        sec_admins.len(),
        2,
        "security must still retain both admins"
    );

    // ATTACK 2.2: Revoking "organization" scope from trinity
    let prop_revoke_org = ChangeProposal {
        proposal_id: "prop-attack-revoke-org".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::RevokeScope {
            mailbox: "trinity@uor.foundation".to_string(),
            scope: "organization".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 102,
                signature_hex: None,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 103,
                signature_hex: None,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop_revoke_org).unwrap();
    let err = manager
        .execute_proposal("prop-attack-revoke-org")
        .expect_err("proposal leaving < 2 admins in 'organization' must be atomically rejected");
    assert!(matches!(
        err,
        AuthorityError::PostChangeCoverageDeficit { ref scope, remaining: 1, required: 2 } if scope == "organization"
    ));
    assert_eq!(manager.records.get(&org_id).unwrap().revision, initial_rev);

    // ATTACK 2.3: Completely retiring an administrator (trinity)
    let prop_retire_admin = ChangeProposal {
        proposal_id: "prop-attack-retire-admin".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::RetireAdministrator {
            mailbox: "trinity@uor.foundation".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 104,
                signature_hex: None,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 105,
                signature_hex: None,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop_retire_admin).unwrap();
    let err = manager
        .execute_proposal("prop-attack-retire-admin")
        .expect_err("retiring admin leaving single admin must be atomically rejected");
    assert!(matches!(
        err,
        AuthorityError::PostChangeCoverageDeficit { .. }
    ));
    assert_eq!(manager.records.get(&org_id).unwrap().revision, initial_rev);

    // ATTACK 2.4a: Optimistic concurrency revision conflict at proposal submission
    // Formulate proposal against stale revision 0
    let prop_stale = ChangeProposal {
        proposal_id: "prop-stale-rev".to_string(),
        organization_id: org_id.clone(),
        base_revision: 0,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
            scope: "security".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![],
        status: ProposalStatus::Pending,
    };

    let err = manager
        .submit_proposal(prop_stale)
        .expect_err("stale revision proposal submission must be rejected immediately");
    assert!(matches!(
        err,
        AuthorityError::ConcurrentRevisionConflict {
            expected: 0,
            actual: 1
        }
    ));

    // ATTACK 2.4b: Optimistic concurrency race: proposal formulated at base_revision 1,
    // but another change commits first, advancing revision to 2.
    // Propose adding neo with "operations" scope (valid multi-admin addition)
    let prop_legit = ChangeProposal {
        proposal_id: "prop-legit-advance".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
            scope: "operations".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 110,
                signature_hex: None,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 111,
                signature_hex: None,
            },
        ],
        status: ProposalStatus::Pending,
    };
    manager.submit_proposal(prop_legit).unwrap();

    // Also formulate a concurrent proposal at revision 1 before execution
    let prop_concurrent = ChangeProposal {
        proposal_id: "prop-concurrent-race".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1, // Formulated against revision 1
        action: AuthorityAction::GrantScope {
            mailbox: "cypher@uor.foundation".to_string(),
            user_id: "uor:user:cypher".to_string(),
            public_key: "9988776655443322110099887766554433221100998877665544332211009988"
                .to_string(),
            scope: "operations".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 112,
                signature_hex: None,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 113,
                signature_hex: None,
            },
        ],
        status: ProposalStatus::Pending,
    };
    manager.submit_proposal(prop_concurrent).unwrap();

    // Execute first proposal: succeeds and advances revision from 1 to 2
    let new_rev = manager.execute_proposal("prop-legit-advance").unwrap();
    assert_eq!(new_rev, 2);

    // Attempt to execute concurrent proposal (stale base_revision 1 vs current revision 2)
    let err = manager
        .execute_proposal("prop-concurrent-race")
        .expect_err("concurrent race execution with outdated base_revision must be rejected");
    assert!(matches!(
        err,
        AuthorityError::ConcurrentRevisionConflict {
            expected: 1,
            actual: 2
        }
    ));

    // ATTACK 2.5: Cross-organization boundary violation
    let err = manager
        .verify_cross_org_authority(&org_id, "uor:org:adversarial-other")
        .expect_err("cross-org authority access must fail");
    assert!(matches!(
        err,
        AuthorityError::CrossOrgAuthorityViolation { .. }
    ));
}

// ============================================================================
// PART 3: INVITATION ATTACK VECTORS
// ============================================================================

#[test]
fn adversarial_invitation_lifecycle_attacks() {
    let mut manager = OrganizationManager::new();
    let org_id = "uor:org:invite-adversarial-test";

    // Setup active organization
    manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Invite Adversarial Org".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
            creator_user_id: "uor:user:trinity".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec![
                "organization".to_string(),
                "membership".to_string(),
                "security".to_string(),
            ],
        })
        .unwrap();

    let now = 1718000000;
    let expires = now + 86400;

    // ATTACK 3.1: Create invitation with non-existent inviter or unauthorized inviter
    let err = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:bad-inviter".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "stranger@external.io".to_string(),
            inviter_user_id: "uor:user:stranger".to_string(),
            created_at: now,
            expires_at: expires,
            revision_binding: 1,
        })
        .expect_err("unenrolled inviter must be rejected");
    assert!(matches!(
        err,
        OrganizationError::UnauthorizedRoleElevation(_)
    ));

    // ATTACK 3.2: Create invitation with expires_at <= created_at
    let err = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:bad-expiry".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "candidate@uor.foundation".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: now,
            expires_at: now, // Not strictly after created_at
            revision_binding: 1,
        })
        .expect_err("expires_at <= created_at must be rejected");
    assert!(matches!(err, OrganizationError::InvalidInvitationState(_)));

    // ATTACK 3.3: Create invitation with malformed recipient email
    let err = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:bad-email".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "not-an-email".to_string(),
            invited_scopes: vec!["operations".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: now,
            expires_at: expires,
            revision_binding: 1,
        })
        .expect_err("malformed email must be rejected");
    assert!(matches!(
        err,
        OrganizationError::SingleOwnerBypassRejected(_)
    ));

    // Create a legitimate pending invitation
    let valid_invite_id = "uor:invite:valid-01";
    manager
        .create_invitation(CreateInvitationRequest {
            id: valid_invite_id.to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "neo@uor.foundation".to_string(),
            invited_scopes: vec!["security".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: now,
            expires_at: expires,
            revision_binding: 1,
        })
        .unwrap();

    // ATTACK 3.4: Duplicate pending invitation for same recipient in same org
    let err = manager
        .create_invitation(CreateInvitationRequest {
            id: "uor:invite:duplicate-01".to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "neo@uor.foundation".to_string(),
            invited_scopes: vec!["security".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: now,
            expires_at: expires,
            revision_binding: 1,
        })
        .expect_err("duplicate pending invitation must be rejected");
    assert!(matches!(err, OrganizationError::InvalidInvitationState(_)));

    // ATTACK 3.5: Attempt acceptance of EXPIRED invitation
    let expired_timestamp = expires + 100;
    let err = manager
        .accept_invitation(
            valid_invite_id,
            "neo@uor.foundation",
            "uor:user:neo",
            "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4",
            expired_timestamp,
        )
        .expect_err("expired invitation acceptance must be rejected");
    assert!(matches!(err, OrganizationError::InvitationExpired(_)));
    assert_eq!(
        manager.get_invitation(valid_invite_id).unwrap().status,
        InvitationStatus::Expired
    );

    // Create fresh invitation for remaining tests
    let invite_id_2 = "uor:invite:valid-02";
    manager
        .create_invitation(CreateInvitationRequest {
            id: invite_id_2.to_string(),
            organization_id: org_id.to_string(),
            target_mailbox: "morpheus@uor.foundation".to_string(),
            invited_scopes: vec!["security".to_string()],
            inviter_mailbox: "trinity@uor.foundation".to_string(),
            inviter_user_id: "uor:user:trinity".to_string(),
            created_at: now,
            expires_at: expires,
            revision_binding: 1,
        })
        .unwrap();

    // ATTACK 3.6: Attempt acceptance with MISMATCHED recipient email
    let err = manager
        .accept_invitation(
            invite_id_2,
            "impostor@uor.foundation", // Mismatched!
            "uor:user:impostor",
            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989",
            now + 1000,
        )
        .expect_err("mismatched email acceptance must be rejected");
    assert!(matches!(
        err,
        OrganizationError::InvitationRecipientMismatch(_)
    ));

    // ATTACK 3.7: Attempt acceptance with DUPLICATE public key already enrolled by trinity
    let err = manager
        .accept_invitation(
            invite_id_2,
            "morpheus@uor.foundation",
            "uor:user:morpheus",
            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be", // Trinity's key!
            now + 1000,
        )
        .expect_err("duplicate public key under different mailbox must be rejected");
    assert!(matches!(
        err,
        OrganizationError::DuplicateIdentityDisguise(_)
    ));

    // ATTACK 3.8: Unauthorized revocation (by non-admin or unauthorized admin)
    let err = manager
        .revoke_invitation(invite_id_2, "stranger@external.io")
        .expect_err("unauthorized revocation must be rejected");
    assert!(matches!(
        err,
        OrganizationError::UnauthorizedRoleElevation(_)
    ));

    // ATTACK 3.9: Unauthorized decline (by wrong user)
    let err = manager
        .decline_invitation(invite_id_2, "wrong_user@uor.foundation")
        .expect_err("decline by wrong recipient must be rejected");
    assert!(matches!(
        err,
        OrganizationError::InvitationRecipientMismatch(_)
    ));

    // Successful acceptance
    let morpheus_pk = "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989";
    manager
        .accept_invitation(
            invite_id_2,
            "morpheus@uor.foundation",
            "uor:user:morpheus",
            morpheus_pk,
            now + 2000,
        )
        .expect("legitimate acceptance must succeed");
    assert_eq!(
        manager.get_invitation(invite_id_2).unwrap().status,
        InvitationStatus::Accepted
    );

    // ATTACK 3.10: Double acceptance of already accepted invitation
    let err = manager
        .accept_invitation(
            invite_id_2,
            "morpheus@uor.foundation",
            "uor:user:morpheus",
            morpheus_pk,
            now + 2500,
        )
        .expect_err("double acceptance must be rejected");
    assert!(matches!(err, OrganizationError::InvalidInvitationState(_)));

    // ATTACK 3.11: Revoking an already Accepted invitation
    let err = manager
        .revoke_invitation(invite_id_2, "trinity@uor.foundation")
        .expect_err("revoking accepted invitation must be rejected");
    assert!(matches!(err, OrganizationError::InvalidInvitationState(_)));
}

// ============================================================================
// PART 4: OWNER ATTESTATION & SIGNATURE TAMPERING ATTACKS
// ============================================================================

#[test]
fn adversarial_owner_attestation_attacks() {
    let root = repo_model::repo_root();
    let syn_model = SyntheticModel::load(&root.join("model")).expect("model loads");
    let inputs = &syn_model.owner_inputs;

    // Baseline valid charter attestation signed by trinity
    let valid_charter = SignedOwnerAttestation {
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
        .expect("baseline valid charter attestation must verify cleanly");

    // ATTACK 4.1: Tamper with document digest (1 bit flipped)
    let mut tampered_digest = valid_charter.clone();
    tampered_digest.document_digest =
        "sha256:7379c7314f6e635202e221f787737531eefe1ac28cbbf034d04e490e769e6d42".to_string();
    let err = inputs
        .verify_attestation(&tampered_digest)
        .expect_err("tampered document digest must be rejected");
    assert!(err.to_string().contains("does not match"));

    // ATTACK 4.2: Malformed document digest format (wrong prefix or length)
    let mut bad_digest_format = valid_charter.clone();
    bad_digest_format.document_digest = "md5:7379c7314f6e635202e221f787737531".to_string();
    let err = inputs
        .verify_attestation(&bad_digest_format)
        .expect_err("non-sha256 digest format must be rejected");
    assert!(err.to_string().contains("invalid document digest"));

    // ATTACK 4.3: Unenrolled administrator mailbox
    let mut unenrolled_signer = valid_charter.clone();
    unenrolled_signer.signer_mailbox = "mallory@evil.io".to_string();
    let err = inputs
        .verify_attestation(&unenrolled_signer)
        .expect_err("unenrolled administrator signer must be rejected");
    assert!(err.to_string().contains("not an enrolled administrator"));

    // ATTACK 4.4: Signer public key mismatch against enrolled key
    let mut key_mismatch = valid_charter.clone();
    key_mismatch.signer_public_key =
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string();
    let err = inputs
        .verify_attestation(&key_mismatch)
        .expect_err("mismatched signer public key must be rejected");
    assert!(err.to_string().contains("does not match enrolled key"));

    // ATTACK 4.5: Signer lacks required scope for attestation type
    // Site-assessment requires "security" scope; test with an admin lacking security scope
    let mut scope_attack = valid_charter.clone();
    scope_attack.attestation_type = "site-assessment".to_string();
    scope_attack.document_digest =
        "sha256:b2dffcef8d86cb92b457f6d4c284c018f126b3b3289c76270f6b23e362aee081".to_string();
    // Neo lacks security scope for site-assessment in synthetic owner inputs
    scope_attack.signer_mailbox = "neo@uor.foundation".to_string();
    scope_attack.signer_public_key =
        "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4".to_string();
    let err = inputs
        .verify_attestation(&scope_attack)
        .expect_err("signer lacking required security scope must be rejected");
    assert!(err.to_string().contains("lacks required scope"));

    // ATTACK 4.6: Organization ID mismatch
    let mut org_mismatch = valid_charter.clone();
    org_mismatch.organization_id = "uor:org:counterfeit-foundation".to_string();
    let err = inputs
        .verify_attestation(&org_mismatch)
        .expect_err("organization ID mismatch must be rejected");
    assert!(err
        .to_string()
        .contains("does not match owner input organization"));

    // ATTACK 4.7: Unknown attestation type
    let mut bad_type = valid_charter.clone();
    bad_type.attestation_type = "arbitrary-privilege-grant".to_string();
    let err = inputs
        .verify_attestation(&bad_type)
        .expect_err("unknown attestation type must be rejected");
    assert!(err.to_string().contains("unknown attestation type"));

    // ATTACK 4.8: Malformed ECDSA signature: non-hex characters
    let mut non_hex_sig = valid_charter.clone();
    non_hex_sig.signature_hex = "zzzz".repeat(32);
    let err = inputs
        .verify_attestation(&non_hex_sig)
        .expect_err("non-hex signature must be rejected");
    assert!(err.to_string().contains("invalid signature format"));

    // ATTACK 4.9: Truncated signature length
    let mut truncated_sig = valid_charter.clone();
    truncated_sig.signature_hex = "304402202b8d00938f".to_string();
    let err = inputs
        .verify_attestation(&truncated_sig)
        .expect_err("truncated signature must be rejected");
    assert!(err.to_string().contains("invalid signature format"));

    // ATTACK 4.10: Zero-scalar signature components
    let mut zero_sig = valid_charter.clone();
    zero_sig.signature_hex = "0".repeat(128);
    let err = inputs
        .verify_attestation(&zero_sig)
        .expect_err("zero signature must be rejected");
    assert!(err.to_string().contains("invalid zero signature"));
}

// ============================================================================
// PART 5: PROPOSAL APPROVAL SIGNATURE INTEGRITY ATTACKS
// ============================================================================

#[test]
fn adversarial_proposal_approval_signature_integrity() {
    let (mut manager, org_id, _) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    let proposal = ChangeProposal {
        proposal_id: "prop-sig-test".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
            scope: "operations".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![],
        status: ProposalStatus::Pending,
    };
    manager.submit_proposal(proposal).unwrap();

    // ATTACK 5.1: Non-hex signature in ProposalApproval
    let bad_hex_approval = ProposalApproval {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "uor:user:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 100,
        signature_hex: Some("NOT_A_HEX_STRING_INVALID_SIGNATURE".to_string()),
    };
    let err = manager
        .add_approval("prop-sig-test", bad_hex_approval)
        .expect_err("non-hex proposal signature must be rejected");
    assert!(matches!(err, AuthorityError::InvalidSignature(_)));

    // ATTACK 5.2: Invalid length signature in ProposalApproval (e.g. 100 chars, not 128 or DER 136-144)
    let bad_len_approval = ProposalApproval {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "uor:user:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 101,
        signature_hex: Some("11".repeat(50)), // 100 chars
    };
    let err = manager
        .add_approval("prop-sig-test", bad_len_approval)
        .expect_err("invalid length signature must be rejected");
    assert!(matches!(err, AuthorityError::InvalidSignature(_)));

    // ATTACK 5.3: All-zeros raw P-256 signature (128 hex zeros)
    let zero_sig_approval = ProposalApproval {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "uor:user:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 102,
        signature_hex: Some("0".repeat(128)),
    };
    let err = manager
        .add_approval("prop-sig-test", zero_sig_approval)
        .expect_err("all-zeros signature components must be rejected");
    assert!(matches!(err, AuthorityError::InvalidSignature(_)));

    // ATTACK 5.4: Unauthorized approver (approver lacking affected scope)
    let stranger_approval = ProposalApproval {
        mailbox: "intruder@external.io".to_string(),
        user_id: "uor:user:intruder".to_string(),
        public_key: "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef"
            .to_string(),
        timestamp: 103,
        signature_hex: None,
    };
    let err = manager
        .add_approval("prop-sig-test", stranger_approval)
        .expect_err("unauthorized approver must be rejected");
    assert!(matches!(err, AuthorityError::UnauthorizedApprover(_)));
}
