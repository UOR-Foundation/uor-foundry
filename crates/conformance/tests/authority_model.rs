//! Conformance tests for AM-01: Scoped multi-administrator authority model.

use repo_model::authority::{
    AuthorityAction, AuthorityError, AuthorityManager, ChangeProposal, ProposalApproval,
    ProposalStatus,
};
use repo_model::organization::OrgAdministrator;
use repo_model::Model;

fn test_setup() -> (AuthorityManager, String) {
    let model = Model::load_from_repo_root().expect("model must load and check cleanly");
    model.check().expect("model consistency check must pass");

    let manager = AuthorityManager::new(model.authority.clone());
    let org_id = "uor:org:uor-foundation".to_string();
    (manager, org_id)
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

#[test]
fn single_owner_bypass_is_rejected() {
    let (mut manager, org_id) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    let proposal = ChangeProposal {
        proposal_id: "prop-01".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
            scope: "security".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![ProposalApproval {
            mailbox: "trinity@uor.foundation".to_string(),
            user_id: "uor:user:trinity".to_string(),
            public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            timestamp: 100,
        }],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(proposal).unwrap();
    let err = manager
        .execute_proposal("prop-01")
        .expect_err("single-owner execution must be rejected");
    assert!(matches!(err, AuthorityError::SingleOwnerBypassRejected(_)));
}

#[test]
fn duplicate_key_disguise_is_rejected() {
    let (mut manager, org_id) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    let proposal = ChangeProposal {
        proposal_id: "prop-disguise".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
            scope: "security".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![ProposalApproval {
            mailbox: "trinity@uor.foundation".to_string(),
            user_id: "uor:user:trinity".to_string(),
            public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            timestamp: 100,
        }],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(proposal.clone()).unwrap();

    // Attempt second approval using different mailbox but same public key
    let fake_approval = ProposalApproval {
        mailbox: "alias@uor.foundation".to_string(),
        user_id: "uor:user:alias".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 101,
    };

    let err = manager
        .add_approval("prop-disguise", fake_approval)
        .expect_err("approving with duplicate key must be rejected");
    assert!(matches!(
        err,
        AuthorityError::DuplicateKeyDisguise(_) | AuthorityError::UnauthorizedApprover(_)
    ));
}

#[test]
fn atomic_post_change_coverage_rejects_dropping_below_quorum() {
    let (mut manager, org_id) = test_setup();
    let admins = sample_admins(); // exactly 2 admins with "security" scope
    manager.initialize_organization(&org_id, &admins).unwrap();

    // Propose revoking "security" scope from morpheus
    let proposal = ChangeProposal {
        proposal_id: "prop-revoke-security".to_string(),
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
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 101,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(proposal).unwrap();
    let err = manager
        .execute_proposal("prop-revoke-security")
        .expect_err("revocation leaving 1 security admin must fail atomic coverage check");

    match err {
        AuthorityError::PostChangeCoverageDeficit {
            scope,
            remaining,
            required,
        } => {
            assert_eq!(scope, "security");
            assert_eq!(remaining, 1);
            assert_eq!(required, 2);
        }
        other => panic!("expected PostChangeCoverageDeficit, got {other:?}"),
    }
}

#[test]
fn concurrent_revision_conflict_prevents_lockout() {
    let (mut manager, org_id) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    // Prepare proposal 1 on base revision 1
    let prop1 = ChangeProposal {
        proposal_id: "prop-1".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
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
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 101,
            },
        ],
        status: ProposalStatus::Pending,
    };

    // Prepare proposal 2 concurrently on base revision 1
    let prop2 = ChangeProposal {
        proposal_id: "prop-2".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "cypher@uor.foundation".to_string(),
            user_id: "uor:user:cypher".to_string(),
            public_key: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                .to_string(),
            scope: "operations".to_string(),
        },
        proposer_mailbox: "morpheus@uor.foundation".to_string(),
        approvals: vec![
            ProposalApproval {
                mailbox: "trinity@uor.foundation".to_string(),
                user_id: "uor:user:trinity".to_string(),
                public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                    .to_string(),
                timestamp: 100,
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 101,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop1).unwrap();
    manager.submit_proposal(prop2).unwrap();

    // Execute prop1 -> revision advances to 2
    let new_rev = manager.execute_proposal("prop-1").unwrap();
    assert_eq!(new_rev, 2);

    // Now execute prop2 formulated on base revision 1 -> rejected with revision conflict
    let err = manager
        .execute_proposal("prop-2")
        .expect_err("stale proposal execution must fail optimistic concurrency fencing");
    assert!(matches!(
        err,
        AuthorityError::ConcurrentRevisionConflict {
            expected: 1,
            actual: 2
        }
    ));
}

#[test]
fn premature_bootstrap_retirement_is_rejected() {
    let (mut manager, org_id) = test_setup();
    // Sole founding administrator in provisional state
    let sole_admin = vec![OrgAdministrator {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "uor:user:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        status: "provisional".to_string(),
        scopes: vec![
            "organization".to_string(),
            "security".to_string(),
            "operations".to_string(),
        ],
    }];
    manager
        .initialize_organization(&org_id, &sole_admin)
        .unwrap();

    let proposal = ChangeProposal {
        proposal_id: "retire-bootstrap".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::RetireBootstrapGrant {
            founding_mailbox: "trinity@uor.foundation".to_string(),
        },
        proposer_mailbox: "trinity@uor.foundation".to_string(),
        approvals: vec![ProposalApproval {
            mailbox: "trinity@uor.foundation".to_string(),
            user_id: "uor:user:trinity".to_string(),
            public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            timestamp: 100,
        }],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(proposal).unwrap();
    let err = manager
        .execute_proposal("retire-bootstrap")
        .expect_err("retiring bootstrap without replacement coverage must be rejected");
    assert!(matches!(
        err,
        AuthorityError::SingleOwnerBypassRejected(_)
            | AuthorityError::PrematureBootstrapRetirement(_)
    ));
}

#[test]
fn cross_org_authority_access_is_rejected() {
    let (manager, _) = test_setup();
    let err = manager
        .verify_cross_org_authority("uor:org:org-alpha", "uor:org:org-beta")
        .expect_err("cross-org authority operations must be rejected");
    assert!(matches!(
        err,
        AuthorityError::CrossOrgAuthorityViolation { .. }
    ));
}

#[test]
fn authority_model_enforces_scoped_quorums_atomic_coverage_and_lockout_safety_am_01() {
    let (mut manager, org_id) = test_setup();
    let admins = sample_admins();
    manager.initialize_organization(&org_id, &admins).unwrap();

    // 1. Propose adding neo with security and operations scopes
    let prop_add = ChangeProposal {
        proposal_id: "prop-add-neo".to_string(),
        organization_id: org_id.clone(),
        base_revision: 1,
        action: AuthorityAction::GrantScope {
            mailbox: "neo@uor.foundation".to_string(),
            user_id: "uor:user:neo".to_string(),
            public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                .to_string(),
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
            },
            ProposalApproval {
                mailbox: "morpheus@uor.foundation".to_string(),
                user_id: "uor:user:morpheus".to_string(),
                public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                    .to_string(),
                timestamp: 101,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop_add).unwrap();
    let rev = manager
        .execute_proposal("prop-add-neo")
        .expect("multi-admin addition must execute cleanly");
    assert_eq!(rev, 2);

    // Verify distinct administrators for "security" is now 3
    let record = manager.records.get(&org_id).unwrap();
    let sec_admins = AuthorityManager::distinct_administrators_for_scope(record, "security");
    assert_eq!(sec_admins.len(), 3);
    assert!(sec_admins.contains("trinity@uor.foundation"));
    assert!(sec_admins.contains("morpheus@uor.foundation"));
    assert!(sec_admins.contains("neo@uor.foundation"));

    // 2. Now revoke morpheus from "security" - remaining distinct admins is 2 (trinity and neo), satisfying quorum >= 2
    let prop_revoke = ChangeProposal {
        proposal_id: "prop-revoke-morpheus".to_string(),
        organization_id: org_id.clone(),
        base_revision: 2,
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
                timestamp: 102,
            },
            ProposalApproval {
                mailbox: "neo@uor.foundation".to_string(),
                user_id: "uor:user:neo".to_string(),
                public_key: "cf122ae443d3fcad8b90fe30277d3c37e008c60d56c9658a44848bdb6f2078d4"
                    .to_string(),
                timestamp: 103,
            },
        ],
        status: ProposalStatus::Pending,
    };

    manager.submit_proposal(prop_revoke).unwrap();
    let rev = manager
        .execute_proposal("prop-revoke-morpheus")
        .expect("revocation with 2 retained distinct admins must succeed");
    assert_eq!(rev, 3);

    let record = manager.records.get(&org_id).unwrap();
    let sec_admins = AuthorityManager::distinct_administrators_for_scope(record, "security");
    assert_eq!(sec_admins.len(), 2);
    assert!(!sec_admins.contains("morpheus@uor.foundation"));
}
