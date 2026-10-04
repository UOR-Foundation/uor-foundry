//! Adversarial stress test harness for Milestone 3 Projects lifecycle,
//! deliverables, tamper-evident SHA-256 chained activity journals, releases,
//! and dynamic organization retirement protection.

use repo_model::{
    ActivateOrganizationRequest, CreateOrganizationRequest, CreateProjectReleaseRequest,
    DeliverableStatus, MilestoneStatus, Model, OrgAdministrator, OrganizationError,
    OrganizationLifecycleState, OrganizationManager, ProjectError, ProjectLifecycleState,
    ProjectManager, ProjectRole, ProjectSettings, ProposalApproval,
};

#[test]
fn test_stress_project_creation_and_input_boundaries() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:adversarial-test-01";

    // 1. Empty project ID rejected
    let err_empty_id = pm
        .create_project(
            Some("admin"),
            org_id,
            "",
            "Valid Name",
            "Description",
            "alice@uor.foundation",
        )
        .expect_err("empty project ID must fail");
    assert!(matches!(err_empty_id, ProjectError::Validation(_)));

    // 2. Whitespace-only project ID rejected
    let err_ws_id = pm
        .create_project(
            Some("admin"),
            org_id,
            "   \t\n  ",
            "Valid Name",
            "Description",
            "alice@uor.foundation",
        )
        .expect_err("whitespace project ID must fail");
    assert!(matches!(err_ws_id, ProjectError::Validation(_)));

    // 3. Empty project name rejected
    let err_empty_name = pm
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv-01:valid-id",
            "",
            "Description",
            "alice@uor.foundation",
        )
        .expect_err("empty project name must fail");
    assert!(matches!(err_empty_name, ProjectError::Validation(_)));

    // 4. Whitespace project name rejected
    let err_ws_name = pm
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv-01:valid-id-2",
            "   ",
            "Description",
            "alice@uor.foundation",
        )
        .expect_err("whitespace project name must fail");
    assert!(matches!(err_ws_name, ProjectError::Validation(_)));

    // 5. Valid creation succeeds
    let valid_proj = pm
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv-01:valid-id-1",
            "Valid Name",
            "Description",
            "alice@uor.foundation",
        )
        .expect("valid project creation must succeed");
    assert_eq!(valid_proj.state, ProjectLifecycleState::Active);
    assert_eq!(valid_proj.members.len(), 1);
    assert_eq!(valid_proj.members[0].role, ProjectRole::Lead);

    // 6. Duplicate ID rejected even with different name and creator
    let err_dup = pm
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv-01:valid-id-1",
            "Completely Different Name",
            "Different description",
            "bob@uor.foundation",
        )
        .expect_err("duplicate project ID must fail");
    assert!(matches!(err_dup, ProjectError::DuplicateProjectId(_)));
}

#[test]
fn test_stress_cross_org_isolation_boundaries() {
    let mut pm = ProjectManager::new();
    let org_1 = "uor:org:alpha-01";
    let org_2 = "uor:org:beta-02";
    let prj_1 = "uor:project:alpha-01:core";
    let prj_2 = "uor:project:beta-02:core";

    pm.create_project(
        Some("admin"),
        org_1,
        prj_1,
        "Alpha Core",
        "Desc",
        "alice@alpha.org",
    )
    .expect("create alpha project");

    pm.create_project(
        Some("admin"),
        org_2,
        prj_2,
        "Beta Core",
        "Desc",
        "bob@beta.org",
    )
    .expect("create beta project");

    // Attempt to access alpha project using beta org ID
    let cross_get = pm
        .get_project(org_2, prj_1)
        .expect_err("cross-org get must fail");
    assert!(matches!(
        cross_get,
        ProjectError::CrossOrgBoundaryViolation { ref org_id, ref project_id }
        if org_id == org_2 && project_id == prj_1
    ));

    // Attempt to mutably access beta project using alpha org ID
    let cross_mut = pm
        .get_project_mut(org_1, prj_2)
        .expect_err("cross-org get_mut must fail");
    assert!(matches!(
        cross_mut,
        ProjectError::CrossOrgBoundaryViolation { ref org_id, ref project_id }
        if org_id == org_1 && project_id == prj_2
    ));

    // Verify list_projects_for_org partitions cleanly
    let list_alpha = pm.list_projects_for_org(org_1);
    assert_eq!(list_alpha.len(), 1);
    assert_eq!(list_alpha[0].id, prj_1);

    let list_beta = pm.list_projects_for_org(org_2);
    assert_eq!(list_beta.len(), 1);
    assert_eq!(list_beta[0].id, prj_2);

    let list_unknown = pm.list_projects_for_org("uor:org:non-existent");
    assert_eq!(list_unknown.len(), 0);
}

#[test]
fn test_stress_project_lifecycle_state_machine() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:lifecycle-01";
    let prj_id = "uor:project:lc-01:engine";
    let lead = "lead@uor.foundation";
    let non_lead = "random@uor.foundation";

    pm.create_project(Some("admin"), org_id, prj_id, "Engine", "Desc", lead)
        .expect("create project");

    // 1. Non-lead cannot archive project
    let err_unauth_archive = pm
        .archive_project(non_lead, prj_id)
        .expect_err("non-lead cannot archive");
    assert!(matches!(
        err_unauth_archive,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 2. Lead archives project -> state becomes Archived
    pm.archive_project(lead, prj_id).expect("lead archives");
    let prj = pm.get_project(org_id, prj_id).expect("fetch project");
    assert_eq!(prj.state, ProjectLifecycleState::Archived);
    assert_eq!(pm.active_project_count(org_id), 0);

    // 3. Cannot archive an already Archived project
    let err_rearchive = pm
        .archive_project(lead, prj_id)
        .expect_err("re-archive must fail");
    assert!(matches!(
        err_rearchive,
        ProjectError::InvalidProjectStateTransition { .. }
    ));

    // 4. Cannot update an Archived project
    let err_update_archived = pm
        .update_project(
            lead,
            prj_id,
            "New Name",
            ProjectSettings {
                description: "New desc".to_string(),
                ..Default::default()
            },
        )
        .expect_err("update archived project must fail");
    assert!(matches!(
        err_update_archived,
        ProjectError::InvalidProjectStateTransition { .. }
    ));

    // 5. Cannot create milestones on an Archived project
    let err_ms_archived = pm
        .create_milestone(lead, prj_id, "m-fail", "Title", "Desc", "2026-12-01")
        .expect_err("create milestone on archived project must fail");
    assert!(matches!(
        err_ms_archived,
        ProjectError::InvalidProjectStateTransition { .. }
    ));

    // 6. Lead restores project -> state becomes Active
    pm.restore_project(lead, prj_id).expect("lead restores");
    let prj_active = pm.get_project(org_id, prj_id).expect("fetch active");
    assert_eq!(prj_active.state, ProjectLifecycleState::Active);
    assert_eq!(pm.active_project_count(org_id), 1);

    // 7. Cannot restore an already Active project
    let err_rerestore = pm
        .restore_project(lead, prj_id)
        .expect_err("restore active project must fail");
    assert!(matches!(
        err_rerestore,
        ProjectError::InvalidProjectStateTransition { .. }
    ));

    // 8. Add milestone and deliverable to test active dependency safeguard on deletion
    pm.create_milestone(
        lead,
        prj_id,
        "m-blocker",
        "Blocker Milestone",
        "Must be finished",
        "2026-11-01",
    )
    .expect("create blocker milestone");

    pm.add_deliverable(
        lead,
        prj_id,
        "m-blocker",
        "d-blocker",
        "Blocker Deliverable",
    )
    .expect("add deliverable");

    // 9. Deletion without force_override must be rejected
    let err_delete_blocked = pm
        .delete_project(lead, prj_id, false)
        .expect_err("active dependencies must block deletion");
    assert!(matches!(
        err_delete_blocked,
        ProjectError::ActiveDependenciesPreventDeletion {
            ref project_id,
            active_milestones,
            active_deliverables,
        }
        if project_id == prj_id && active_milestones == 1 && active_deliverables == 1
    ));

    // 10. Deletion with force_override = true succeeds
    pm.delete_project(lead, prj_id, true)
        .expect("force deletion must succeed");
    let prj_deleted = pm.get_project(org_id, prj_id).expect("fetch deleted");
    assert_eq!(prj_deleted.state, ProjectLifecycleState::Deleted);
    assert_eq!(pm.active_project_count(org_id), 0);

    // 11. Cannot restore a Deleted project
    let err_restore_deleted = pm
        .restore_project(lead, prj_id)
        .expect_err("restore deleted project must fail");
    assert!(matches!(
        err_restore_deleted,
        ProjectError::InvalidProjectStateTransition { .. }
    ));
}

#[test]
fn test_stress_tamper_evident_activity_chain_deep() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:tamper-audit-01";
    let prj_id = "uor:project:ta-01:hash-chain";
    let lead = "lead@uor.foundation";

    // Genesis creation
    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Hash Chain Test",
        "Audit",
        lead,
    )
    .expect("create project");

    // Verify Genesis entry
    let prj = pm.get_project(org_id, prj_id).expect("get project");
    assert_eq!(prj.activity_log.len(), 1);
    let genesis = &prj.activity_log[0];
    assert_eq!(genesis.prev_entry_digest.len(), 64);
    assert_eq!(genesis.prev_entry_digest, "0".repeat(64));
    assert_eq!(genesis.prev_entry_digest, ProjectManager::GENESIS_DIGEST);
    assert!(pm
        .verify_activity_chain(prj_id)
        .expect("genesis chain valid"));

    // Build a multi-entry activity log
    pm.append_activity(prj_id, lead, "Action1", "Details 1")
        .expect("entry 1");
    pm.append_activity(prj_id, lead, "Action2", "Details 2")
        .expect("entry 2");
    pm.append_activity(prj_id, lead, "Action3", "Details 3")
        .expect("entry 3");
    pm.append_activity(prj_id, lead, "Action4", "Details 4")
        .expect("entry 4");
    pm.append_activity(prj_id, lead, "Action5", "Details 5")
        .expect("entry 5");

    assert_eq!(
        pm.get_project(org_id, prj_id).unwrap().activity_log.len(),
        6
    );
    assert!(pm
        .verify_activity_chain(prj_id)
        .expect("multi-entry chain valid"));

    // Adversarial Challenge 1: Tamper with Genesis predecessor digest
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[0].prev_entry_digest.clone();
        prj_mut.activity_log[0].prev_entry_digest = "1".repeat(64);
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered genesis prev must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[0].prev_entry_digest = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 2: Tamper with Genesis details
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[0].details.clone();
        prj_mut.activity_log[0].details = "Altered genesis details".to_string();
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered genesis details must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[0].details = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 3: Tamper with historical entry 2 timestamp
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[2].timestamp.clone();
        prj_mut.activity_log[2].timestamp = "1970-01-01T00:00:00Z".to_string();
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered timestamp must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[2].timestamp = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 4: Tamper with historical entry 3 actor
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[3].actor.clone();
        prj_mut.activity_log[3].actor = "impostor@evil.com".to_string();
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered actor must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[3].actor = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 5: Tamper with historical entry 4 action
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[4].action.clone();
        prj_mut.activity_log[4].action = "UnauthorizedAction".to_string();
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered action must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[4].action = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 6: Tamper with historical entry 4 details
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let orig = prj_mut.activity_log[4].details.clone();
        prj_mut.activity_log[4].details = "Malicious injection".to_string();
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("tampered details must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id).unwrap().activity_log[4].details = orig;
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }

    // Adversarial Challenge 7: Splicing / deleting an intermediate entry
    {
        let prj_mut = pm.get_project_mut(org_id, prj_id).unwrap();
        let removed_entry = prj_mut.activity_log.remove(2);
        let err = pm
            .verify_activity_chain(prj_id)
            .expect_err("spliced log must fail");
        assert!(matches!(err, ProjectError::ActivityLogTampered { .. }));
        pm.get_project_mut(org_id, prj_id)
            .unwrap()
            .activity_log
            .insert(2, removed_entry);
        assert!(pm.verify_activity_chain(prj_id).unwrap());
    }
}

#[test]
fn test_stress_releases_immutability_and_permissions() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:release-stress-01";
    let prj_id = "uor:project:rs-01:immutable";
    let lead = "lead@uor.foundation";
    let contributor = "dev@uor.foundation";

    pm.create_project(Some("admin"), org_id, prj_id, "Releases", "Desc", lead)
        .expect("create project");
    pm.add_or_update_member(
        lead,
        prj_id,
        contributor,
        "usr:dev",
        ProjectRole::Contributor,
    )
    .expect("add contributor");

    // 1. Contributor cannot create release
    let req_unauth = CreateProjectReleaseRequest {
        caller_mailbox: contributor,
        project_id: prj_id,
        release_id: "rel-1",
        version_tag: "v1.0.0",
        title: "Rel 1",
        notes: "Notes",
        model_revision: 1,
        commit_sha: "abcdef123456",
        artifact_root_digest:
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    };
    let err_unauth_create = pm
        .create_release(req_unauth)
        .expect_err("contributor cannot create release");
    assert!(matches!(
        err_unauth_create,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 2. Lead creates release draft
    let req_lead = CreateProjectReleaseRequest {
        caller_mailbox: lead,
        project_id: prj_id,
        release_id: "rel-1",
        version_tag: "v1.0.0",
        title: "Rel 1",
        notes: "Notes",
        model_revision: 1,
        commit_sha: "abcdef123456",
        artifact_root_digest:
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    };
    let release = pm
        .create_release(req_lead.clone())
        .expect("lead creates release");
    assert!(!release.is_published);

    // 3. Duplicate release ID rejected
    let err_dup_release = pm
        .create_release(req_lead)
        .expect_err("duplicate release ID must fail");
    assert!(matches!(err_dup_release, ProjectError::Validation(_)));

    // 4. Contributor cannot publish release
    let err_unauth_pub = pm
        .publish_release(contributor, prj_id, "rel-1")
        .expect_err("contributor cannot publish release");
    assert!(matches!(
        err_unauth_pub,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 5. Lead publishes release
    pm.publish_release(lead, prj_id, "rel-1")
        .expect("lead publishes release");
    let prj = pm.get_project(org_id, prj_id).expect("fetch project");
    assert!(prj.releases[0].is_published);

    // 6. Republishing is strictly rejected as immutable
    let err_repub = pm
        .publish_release(lead, prj_id, "rel-1")
        .expect_err("re-publishing immutable release must fail");
    assert!(matches!(
        err_repub,
        ProjectError::ReleaseAlreadyPublished(_)
    ));
}

#[test]
fn test_stress_organization_retirement_with_multi_state_projects() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut org_manager = OrganizationManager::new();
    let mut project_manager = ProjectManager::new();

    let org_id = "uor:org:adv-retire-01";

    // Setup active organization with 2 admins for quorum
    org_manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Retirement Test Org".to_string(),
            creator_user_id: "usr:admin1".to_string(),
            creator_mailbox: "admin1@uor.foundation".to_string(),
            creator_public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            creator_scopes: vec!["organization".to_string(), "security".to_string()],
        })
        .expect("create org");

    org_manager
        .activate_organization(
            ActivateOrganizationRequest {
                organization_id: org_id.to_string(),
                administrators: vec![
                    OrgAdministrator {
                        mailbox: "admin1@uor.foundation".to_string(),
                        user_id: "usr:admin1".to_string(),
                        public_key:
                            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec!["organization".to_string(), "security".to_string()],
                    },
                    OrgAdministrator {
                        mailbox: "admin2@uor.foundation".to_string(),
                        user_id: "usr:admin2".to_string(),
                        public_key:
                            "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec!["organization".to_string(), "security".to_string()],
                    },
                ],
                approving_mailboxes: vec![
                    "admin1@uor.foundation".to_string(),
                    "admin2@uor.foundation".to_string(),
                ],
            },
            rules,
        )
        .expect("activate org");

    let approvals = vec![
        ProposalApproval {
            mailbox: "admin1@uor.foundation".to_string(),
            user_id: "usr:admin1".to_string(),
            public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                .to_string(),
            timestamp: 1718000000,
            signature_hex: None,
        },
        ProposalApproval {
            mailbox: "admin2@uor.foundation".to_string(),
            user_id: "usr:admin2".to_string(),
            public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989"
                .to_string(),
            timestamp: 1718000001,
            signature_hex: None,
        },
    ];

    // Case 1: 0 projects -> retirement without force_override is ALLOWED
    assert_eq!(project_manager.active_project_count(org_id), 0);
    // (Don't retire yet, we need the org for next steps)

    // Case 2: Add 1 active project -> retirement without force_override MUST FAIL
    project_manager
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv:p1",
            "Project 1",
            "Desc",
            "admin1@uor.foundation",
        )
        .expect("create p1");

    assert_eq!(project_manager.active_project_count(org_id), 1);
    let err_blocked = org_manager
        .retire_organization_with_project_manager(org_id, &approvals, false, &project_manager)
        .expect_err("active project must block retirement");
    assert!(matches!(
        err_blocked,
        OrganizationError::DependentResourcesActive(_)
    ));

    // Case 3: Archive Project 1 -> active count becomes 0 -> retirement without force_override is ALLOWED
    project_manager
        .archive_project("admin1@uor.foundation", "uor:project:adv:p1")
        .expect("archive p1");
    assert_eq!(project_manager.active_project_count(org_id), 0);

    // Case 4: Add Project 2 (Active) and Project 3 (Deleted)
    project_manager
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv:p2",
            "Project 2",
            "Desc",
            "admin1@uor.foundation",
        )
        .expect("create p2");
    project_manager
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:adv:p3",
            "Project 3",
            "Desc",
            "admin1@uor.foundation",
        )
        .expect("create p3");
    project_manager
        .delete_project("admin1@uor.foundation", "uor:project:adv:p3", true)
        .expect("delete p3");

    // We have: p1 (Archived), p2 (Active), p3 (Deleted)
    // active_project_count should be exactly 1!
    assert_eq!(project_manager.active_project_count(org_id), 1);

    // Retirement without force_override still fails
    let err_still_blocked = org_manager
        .retire_organization_with_project_manager(org_id, &approvals, false, &project_manager)
        .expect_err("active project p2 must block retirement");
    assert!(matches!(
        err_still_blocked,
        OrganizationError::DependentResourcesActive(_)
    ));

    // Case 5: Retirement WITH force_override = true SUCCEEDS
    let retired_org = org_manager
        .retire_organization_with_project_manager(org_id, &approvals, true, &project_manager)
        .expect("force override retirement must succeed");
    assert_eq!(retired_org.state, OrganizationLifecycleState::Retired);

    // Case 6: Attempting to retire already Retired organization fails
    let err_already_retired = org_manager
        .retire_organization_with_project_manager(org_id, &approvals, true, &project_manager)
        .expect_err("already retired org cannot be retired again");
    assert!(matches!(
        err_already_retired,
        OrganizationError::InvalidStateTransition { .. }
    ));
}

#[test]
fn test_stress_milestones_and_deliverables_authorization() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:ms-del-stress-01";
    let prj_id = "uor:project:ms-del:p1";
    let lead = "lead@uor.foundation";
    let maintainer = "maintainer@uor.foundation";
    let contributor = "contrib@uor.foundation";
    let viewer = "viewer@uor.foundation";

    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Milestone Stress",
        "Desc",
        lead,
    )
    .expect("create project");

    pm.add_or_update_member(
        lead,
        prj_id,
        maintainer,
        "usr:maint",
        ProjectRole::Maintainer,
    )
    .expect("add maintainer");
    pm.add_or_update_member(
        lead,
        prj_id,
        contributor,
        "usr:contrib",
        ProjectRole::Contributor,
    )
    .expect("add contributor");
    pm.add_or_update_member(lead, prj_id, viewer, "usr:viewer", ProjectRole::Viewer)
        .expect("add viewer");

    // 1. Viewer cannot create milestone
    let err_viewer_ms = pm
        .create_milestone(viewer, prj_id, "ms-1", "Title", "Desc", "2026-11-01")
        .expect_err("viewer cannot create milestone");
    assert!(matches!(
        err_viewer_ms,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 2. Contributor cannot create milestone
    let err_contrib_ms = pm
        .create_milestone(contributor, prj_id, "ms-1", "Title", "Desc", "2026-11-01")
        .expect_err("contributor cannot create milestone");
    assert!(matches!(
        err_contrib_ms,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 3. Maintainer creates milestone
    pm.create_milestone(maintainer, prj_id, "ms-1", "Title", "Desc", "2026-11-01")
        .expect("maintainer can create milestone");

    // 4. Viewer cannot add deliverable
    let err_viewer_del = pm
        .add_deliverable(viewer, prj_id, "ms-1", "del-1", "Deliverable 1")
        .expect_err("viewer cannot add deliverable");
    assert!(matches!(
        err_viewer_del,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 5. Maintainer adds deliverable
    pm.add_deliverable(maintainer, prj_id, "ms-1", "del-1", "Deliverable 1")
        .expect("maintainer adds deliverable");

    // 6. Viewer cannot complete deliverable
    let err_viewer_comp = pm
        .complete_deliverable(viewer, prj_id, "ms-1", "del-1", "sha256:abc123")
        .expect_err("viewer cannot complete deliverable");
    assert!(matches!(
        err_viewer_comp,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 7. Contributor completes deliverable with SHA-256 digest proof
    let artifact_proof = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    pm.complete_deliverable(contributor, prj_id, "ms-1", "del-1", artifact_proof)
        .expect("contributor completes deliverable");

    let prj = pm.get_project(org_id, prj_id).expect("fetch project");
    let del = &prj.milestones[0].deliverables[0];
    assert_eq!(del.status, DeliverableStatus::Verified);
    assert_eq!(del.artifact_digest.as_deref(), Some(artifact_proof));

    // 8. Contributor cannot complete milestone
    let err_contrib_comp_ms = pm
        .complete_milestone(contributor, prj_id, "ms-1")
        .expect_err("contributor cannot complete milestone");
    assert!(matches!(
        err_contrib_comp_ms,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // 9. Maintainer completes milestone
    pm.complete_milestone(maintainer, prj_id, "ms-1")
        .expect("maintainer completes milestone");

    let prj_after = pm.get_project(org_id, prj_id).expect("fetch project");
    assert_eq!(prj_after.milestones[0].status, MilestoneStatus::Completed);
}
