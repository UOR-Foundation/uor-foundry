//! Conformance tests for organization-scoped projects, milestones, deliverables,
//! SHA-256 chained activity journals, releases, and organization retirement integration.

use repo_model::{
    ActivateOrganizationRequest, CreateOrganizationRequest, CreateProjectReleaseRequest,
    DeliverableStatus, MilestoneStatus, Model, OrgAdministrator, OrganizationError,
    OrganizationManager, ProjectError, ProjectLifecycleState, ProjectManager, ProjectRole,
    ProposalApproval,
};

#[test]
fn test_project_creation_and_duplicate_display_names() {
    let mut pm = ProjectManager::new();

    // 1. Create Project A in org-01
    let p1_id = {
        let p1 = pm
            .create_project(
                Some("admin"),
                "uor:org:citizen-gardens-01",
                "uor:project:citizen-gardens-01:community-platform",
                "Community Platform",
                "Urban community gardening platform",
                "alice@uor.foundation",
            )
            .expect("create project 1");

        assert_eq!(p1.state, ProjectLifecycleState::Active);
        assert_eq!(p1.name, "Community Platform");
        assert_eq!(p1.members.len(), 1);
        assert_eq!(p1.members[0].role, ProjectRole::Lead);
        assert_eq!(p1.activity_log.len(), 1);
        assert_eq!(p1.activity_log[0].action, "ProjectCreated");
        p1.id.clone()
    };

    // 2. Duplicate display name in SAME organization is allowed with unique ID
    let p2_id = {
        let p2 = pm
            .create_project(
                Some("admin"),
                "uor:org:citizen-gardens-01",
                "uor:project:citizen-gardens-01:community-platform-v2",
                "Community Platform", // Identical display name
                "V2 Urban community gardening platform",
                "bob@uor.foundation",
            )
            .expect("create project 2 with duplicate display name");

        assert_eq!(p2.name, "Community Platform");
        p2.id.clone()
    };

    assert_ne!(p1_id, p2_id);

    // 3. Duplicate display name in DIFFERENT organization is allowed
    let p3 = pm
        .create_project(
            Some("admin"),
            "uor:org:metro-farms-02",
            "uor:project:metro-farms-02:community-platform",
            "Community Platform",
            "Metro farm platform",
            "charlie@uor.foundation",
        )
        .expect("create project 3 in different org");

    assert_eq!(p3.org_id, "uor:org:metro-farms-02");

    // 4. Duplicate project ID is strictly rejected
    let dup_err = pm
        .create_project(
            Some("admin"),
            "uor:org:citizen-gardens-01",
            "uor:project:citizen-gardens-01:community-platform",
            "Another Name",
            "Desc",
            "alice@uor.foundation",
        )
        .expect_err("duplicate id rejected");

    assert!(matches!(dup_err, ProjectError::DuplicateProjectId(_)));
}

#[test]
fn test_cross_org_isolation() {
    let mut pm = ProjectManager::new();

    let org_a = "uor:org:citizen-gardens-01";
    let org_b = "uor:org:metro-farms-02";

    pm.create_project(
        Some("admin"),
        org_a,
        "uor:project:cg-01:core",
        "Core Project",
        "Garden core",
        "alice@uor.foundation",
    )
    .expect("create org_a project");

    pm.create_project(
        Some("admin"),
        org_b,
        "uor:project:mf-02:core",
        "Core Project",
        "Metro core",
        "bob@uor.foundation",
    )
    .expect("create org_b project");

    // Querying org_a project under org_a succeeds
    assert!(pm.get_project(org_a, "uor:project:cg-01:core").is_ok());

    // Querying org_a project under org_b fails with CrossOrgBoundaryViolation
    let cross_err = pm
        .get_project(org_b, "uor:project:cg-01:core")
        .expect_err("cross-org read must fail");

    assert!(matches!(
        cross_err,
        ProjectError::CrossOrgBoundaryViolation { .. }
    ));

    // Listing projects for org_a returns only org_a projects
    let list_a = pm.list_projects_for_org(org_a);
    assert_eq!(list_a.len(), 1);
    assert_eq!(list_a[0].id, "uor:project:cg-01:core");
}

#[test]
fn test_project_lifecycle_archive_restore_and_delete() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let prj_id = "uor:project:cg-01:telemetry";
    let lead = "alice@uor.foundation";

    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Telemetry",
        "Soil sensors",
        lead,
    )
    .expect("create telemetry project");

    // Add milestone
    pm.create_milestone(
        lead,
        prj_id,
        "m-01",
        "Hardware Integration",
        "Assemble sensors",
        "2026-11-01",
    )
    .expect("create milestone");

    // Archive project
    let archived = pm.archive_project(lead, prj_id).expect("archive project");
    assert_eq!(archived.state, ProjectLifecycleState::Archived);

    // Non-lead cannot restore
    let non_lead_err = pm
        .restore_project("mallory@evil.com", prj_id)
        .expect_err("non-lead cannot restore");
    assert!(matches!(
        non_lead_err,
        ProjectError::UnauthorizedProjectAction { .. }
    ));

    // Lead restores project
    let restored = pm.restore_project(lead, prj_id).expect("restore project");
    assert_eq!(restored.state, ProjectLifecycleState::Active);

    // Deletion blocked by active milestone when force_override is false
    let del_err = pm
        .delete_project(lead, prj_id, false)
        .expect_err("active milestones block delete");
    assert!(matches!(
        del_err,
        ProjectError::ActiveDependenciesPreventDeletion { .. }
    ));

    // Force deletion succeeds
    pm.delete_project(lead, prj_id, true)
        .expect("forced delete succeeds");
    let prj = pm.get_project(org_id, prj_id).expect("get project");
    assert_eq!(prj.state, ProjectLifecycleState::Deleted);
}

#[test]
fn test_milestones_and_deliverables_lifecycle() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let prj_id = "uor:project:cg-01:hydrology";
    let lead = "alice@uor.foundation";
    let contributor = "bob@uor.foundation";

    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Hydrology",
        "Watering automation",
        lead,
    )
    .expect("create hydrology project");

    // Add contributor
    pm.add_or_update_member(
        lead,
        prj_id,
        contributor,
        "usr:bob",
        ProjectRole::Contributor,
    )
    .expect("add contributor");

    // Create milestone
    let milestone = pm
        .create_milestone(
            lead,
            prj_id,
            "m-valve-01",
            "Smart Valves",
            "Deploy IoT valves",
            "2026-10-31",
        )
        .expect("create milestone");
    assert_eq!(milestone.status, MilestoneStatus::Planned);

    // Add deliverable
    let del = pm
        .add_deliverable(
            lead,
            prj_id,
            "m-valve-01",
            "d-fw-01",
            "Embedded firmware binary",
        )
        .expect("add deliverable");
    assert_eq!(del.status, DeliverableStatus::Pending);

    // Contributor completes deliverable with cryptographic artifact digest proof
    let artifact_digest = "sha256:490ce59f1388f8d68ef2d3ef08234399e46a164a66a1a89c8a984fe750c4ebba";
    pm.complete_deliverable(
        contributor,
        prj_id,
        "m-valve-01",
        "d-fw-01",
        artifact_digest,
    )
    .expect("complete deliverable");

    // Complete milestone
    pm.complete_milestone(lead, prj_id, "m-valve-01")
        .expect("complete milestone");

    let prj = pm.get_project(org_id, prj_id).expect("fetch project");
    let completed_m = &prj.milestones[0];
    assert_eq!(completed_m.status, MilestoneStatus::Completed);
    assert_eq!(
        completed_m.deliverables[0].status,
        DeliverableStatus::Verified
    );
    assert_eq!(
        completed_m.deliverables[0].artifact_digest.as_deref(),
        Some(artifact_digest)
    );
}

#[test]
fn test_tamper_evident_activity_chain_and_tamper_detection() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let prj_id = "uor:project:cg-01:journal-audit";
    let lead = "alice@uor.foundation";

    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Journal Audit",
        "Tamper proof logging",
        lead,
    )
    .expect("create project");

    // Genesis verification
    assert!(
        pm.verify_activity_chain(prj_id).expect("verify chain"),
        "genesis activity chain must verify"
    );

    // Add activity entries
    pm.append_activity(prj_id, lead, "ConfigUpdated", "Changed default retry=5")
        .expect("append 1");
    pm.append_activity(prj_id, lead, "SecurityScan", "Zero CVEs reported")
        .expect("append 2");

    assert!(
        pm.verify_activity_chain(prj_id).expect("verify chain"),
        "activity chain with multiple entries must verify"
    );

    // Simulate tampering with an entry in the log
    let prj_mut = pm.get_project_mut(org_id, prj_id).expect("fetch mut");
    // Mutate the details of the second entry without recomputing its hash
    prj_mut.activity_log[1].details = "Malicious unauthorized modification".to_string();

    let tamper_err = pm
        .verify_activity_chain(prj_id)
        .expect_err("tampered log must be detected");
    assert!(matches!(
        tamper_err,
        ProjectError::ActivityLogTampered { .. }
    ));
}

#[test]
fn test_project_releases_draft_and_publication_immutability() {
    let mut pm = ProjectManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let prj_id = "uor:project:cg-01:releases";
    let lead = "alice@uor.foundation";

    pm.create_project(
        Some("admin"),
        org_id,
        prj_id,
        "Release Test",
        "Releases tracking",
        lead,
    )
    .expect("create project");

    let req = CreateProjectReleaseRequest {
        caller_mailbox: lead,
        project_id: prj_id,
        release_id: "rel-v0.1.0",
        version_tag: "v0.1.0",
        title: "Beta Release",
        notes: "First functional prototype",
        model_revision: 42,
        commit_sha: "7f8b9c0d1e2f3a4b",
        artifact_root_digest:
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    };

    let release = pm.create_release(req).expect("create release");
    assert!(!release.is_published);
    assert_eq!(release.model_revision, 42);

    // Publish release
    pm.publish_release(lead, prj_id, "rel-v0.1.0")
        .expect("publish release");

    // Second publish must be rejected as immutable
    let repub_err = pm
        .publish_release(lead, prj_id, "rel-v0.1.0")
        .expect_err("already published release cannot be republished");
    assert!(matches!(
        repub_err,
        ProjectError::ReleaseAlreadyPublished(_)
    ));

    let prj = pm.get_project(org_id, prj_id).expect("fetch project");
    assert!(prj.releases[0].is_published);
}

#[test]
fn test_organization_retirement_safeguard_dynamic_link() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    let rules = &model.organization_lifecycle.rules;
    let mut org_manager = OrganizationManager::new();
    let mut project_manager = ProjectManager::new();

    let org_id = "uor:org:citizen-gardens-01";

    // 1. Create and activate organization
    org_manager
        .create_organization(CreateOrganizationRequest {
            id: org_id.to_string(),
            display_name: "Citizen Gardens".to_string(),
            creator_user_id: "usr:trinity".to_string(),
            creator_mailbox: "trinity@uor.foundation".to_string(),
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
                        mailbox: "trinity@uor.foundation".to_string(),
                        user_id: "usr:trinity".to_string(),
                        public_key:
                            "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be"
                                .to_string(),
                        status: "authenticated".to_string(),
                        scopes: vec!["organization".to_string(), "security".to_string()],
                    },
                    OrgAdministrator {
                        mailbox: "morpheus@uor.foundation".to_string(),
                        user_id: "usr:morpheus".to_string(),
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
        .expect("activate org");

    // 2. Add an active project to this organization
    project_manager
        .create_project(
            Some("admin"),
            org_id,
            "uor:project:cg-01:active-hub",
            "Active Hub",
            "Central communication hub",
            "trinity@uor.foundation",
        )
        .expect("create active project");

    assert_eq!(project_manager.active_project_count(org_id), 1);

    let approval_trinity = ProposalApproval {
        mailbox: "trinity@uor.foundation".to_string(),
        user_id: "usr:trinity".to_string(),
        public_key: "b41b52a4cd1c77d96ad8f1c16c11e8b4edb522fb3296bdb27c1eb0048bf057be".to_string(),
        timestamp: 1718000000,
        signature_hex: None,
    };
    let approval_morpheus = ProposalApproval {
        mailbox: "morpheus@uor.foundation".to_string(),
        user_id: "usr:morpheus".to_string(),
        public_key: "7b9de4debb0f6050bf5c4d6284694876c0cf6ec6bcd72fb250d2b58d66538989".to_string(),
        timestamp: 1718000001,
        signature_hex: None,
    };

    // 3. Attempting to retire organization with active project fails when force_override = false
    let retire_err = org_manager
        .retire_organization_with_project_manager(
            org_id,
            &[approval_trinity.clone(), approval_morpheus.clone()],
            false,
            &project_manager,
        )
        .expect_err("active project must block retirement");

    assert!(matches!(
        retire_err,
        OrganizationError::DependentResourcesActive(_)
    ));

    // 4. Force override allows retirement even with active project
    let forced_retire = org_manager
        .retire_organization_with_project_manager(
            org_id,
            &[approval_trinity, approval_morpheus],
            true,
            &project_manager,
        )
        .expect("forced retirement succeeds");

    assert_eq!(
        forced_retire.state,
        repo_model::OrganizationLifecycleState::Retired
    );
}
