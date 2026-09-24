//! Conformance tests for Functional Core: identity, roles, shared workspaces,
//! persistence, messaging, and normal organization creation (FC-01).

use repo_model::{FunctionalCoreCoordinator, FunctionalCoreError, MessageDeliveryState, Model};

/// FC-01: The authorized first-release functional core implements identity, roles,
/// shared workspaces, persistence, and messaging as a single accepted stage without
/// preview substitutions or seeded privileges, proving normal account/organization creation,
/// multi-user interaction, permission enforcement, persisted state recovery, and
/// message delivery with resilient fault recovery.
#[test]
fn functional_core_implements_and_accepts_first_release_fc_01() {
    let root = repo_model::repo_root();
    let model = Model::load(&root.join("model")).expect("model loads");
    model
        .check()
        .expect("model checks and validates functional core");

    let cfg = &model.functional_core;
    assert_eq!(cfg.spec, "foundry/functional-core/1");
    assert_eq!(cfg.stage, "staged-core");
    assert!(cfg.policy.require_single_accepted_stage);
    assert!(cfg.policy.prohibit_preview_substitutions);
    assert!(cfg.policy.prohibit_seeded_privileged_accounts);
    assert!(cfg.policy.prohibit_seeded_privileged_orgs);
    assert!(cfg.policy.require_independent_user_interaction);
    assert!(cfg.policy.require_permission_enforcement);
    assert!(cfg.policy.require_persisted_state_recovery);
    assert!(cfg.policy.require_messaging_delivery_states);

    // 1. Enroll three independent users through standard workflow (no seeded accounts)
    let alice = FunctionalCoreCoordinator::enroll_user(
        "user-alice-01",
        "alice@uor.foundation",
        "Alice Admin",
    )
    .expect("alice enrolls");

    let bob =
        FunctionalCoreCoordinator::enroll_user("user-bob-01", "bob@uor.foundation", "Bob Editor")
            .expect("bob enrolls");

    let charlie = FunctionalCoreCoordinator::enroll_user(
        "user-charlie-01",
        "charlie@uor.foundation",
        "Charlie Viewer",
    )
    .expect("charlie enrolls");

    // 2. Create UOR Foundation through the same standard workflow as any other organization
    let uor_org = FunctionalCoreCoordinator::create_organization(
        &alice,
        "uor:org:uor-foundation",
        "UOR Foundation",
        true,
    )
    .expect("UOR Foundation created via standard workflow");
    assert_eq!(uor_org.display_name, "UOR Foundation");
    assert_eq!(uor_org.creator_mailbox, alice.mailbox);

    // 3. Create shared workspace
    let mut workspace = FunctionalCoreCoordinator::create_workspace(
        &alice,
        &uor_org,
        "ws-foundry-core",
        "Foundry Core Workspace",
    )
    .expect("workspace created");

    // 4. Enroll members with distinct roles (Alice: Admin, Bob: Editor, Charlie: Viewer)
    FunctionalCoreCoordinator::add_member(&mut workspace, &alice, &bob, "editor")
        .expect("bob added as editor");
    FunctionalCoreCoordinator::add_member(&mut workspace, &alice, &charlie, "viewer")
        .expect("charlie added as viewer");

    // 5. Shared workspace state mutations under role permissions
    FunctionalCoreCoordinator::mutate_workspace_state(
        &mut workspace,
        &bob,
        "project_status",
        "in_progress",
    )
    .expect("editor bob mutates shared workspace state");
    assert_eq!(
        workspace
            .shared_data
            .get("project_status")
            .map(String::as_str),
        Some("in_progress")
    );

    // Charlie (viewer) cannot mutate shared state
    let unauthorized_mutation = FunctionalCoreCoordinator::mutate_workspace_state(
        &mut workspace,
        &charlie,
        "project_status",
        "tampered",
    );
    assert!(matches!(
        unauthorized_mutation,
        Err(FunctionalCoreError::UnauthorizedAction { .. })
    ));

    // 6. Messaging with explicit delivery states
    let msg = FunctionalCoreCoordinator::send_workspace_message(
        &mut workspace,
        &alice,
        &bob,
        "msg-01",
        "Welcome to the shared workspace, Bob!",
    )
    .expect("message sent and delivered");
    assert_eq!(msg.delivery_state, MessageDeliveryState::Delivered);
    assert_eq!(workspace.messages.len(), 1);

    // 7. Persisted state recovery across simulated restart
    let recovered_workspace = FunctionalCoreCoordinator::simulate_persistence_restart(&workspace)
        .expect("persistence restart succeeds without data loss");
    assert_eq!(
        recovered_workspace.shared_data.get("project_status"),
        workspace.shared_data.get("project_status")
    );
    assert_eq!(recovered_workspace.messages.len(), 1);

    // 8. Resilient messaging recovery after transient failure
    let mut failed_msg = workspace.messages[0].clone();
    FunctionalCoreCoordinator::simulate_message_failure_and_recovery(&mut failed_msg)
        .expect("message failure and retransmission recovery succeeds");
    assert_eq!(failed_msg.delivery_state, MessageDeliveryState::Delivered);
    assert_eq!(failed_msg.retry_count, 1);
}

#[test]
fn non_admin_cannot_add_members() {
    let alice = FunctionalCoreCoordinator::enroll_user("u1", "a@test.org", "Alice").unwrap();
    let bob = FunctionalCoreCoordinator::enroll_user("u2", "b@test.org", "Bob").unwrap();
    let charlie = FunctionalCoreCoordinator::enroll_user("u3", "c@test.org", "Charlie").unwrap();
    let org = FunctionalCoreCoordinator::create_organization(&alice, "org-01", "Test Org", false)
        .unwrap();

    let mut ws =
        FunctionalCoreCoordinator::create_workspace(&alice, &org, "ws-01", "Workspace").unwrap();
    FunctionalCoreCoordinator::add_member(&mut ws, &alice, &bob, "editor").unwrap();

    // Bob (editor, not admin) cannot add Charlie
    let res = FunctionalCoreCoordinator::add_member(&mut ws, &bob, &charlie, "viewer");
    assert!(matches!(
        res,
        Err(FunctionalCoreError::UnauthorizedAction { .. })
    ));
}

#[test]
fn non_member_cannot_send_workspace_message() {
    let alice = FunctionalCoreCoordinator::enroll_user("u1", "a@test.org", "Alice").unwrap();
    let bob = FunctionalCoreCoordinator::enroll_user("u2", "b@test.org", "Bob").unwrap();
    let outsider =
        FunctionalCoreCoordinator::enroll_user("u4", "outsider@evil.com", "Attacker").unwrap();
    let org = FunctionalCoreCoordinator::create_organization(&alice, "org-01", "Test Org", false)
        .unwrap();

    let mut ws =
        FunctionalCoreCoordinator::create_workspace(&alice, &org, "ws-01", "Workspace").unwrap();
    FunctionalCoreCoordinator::add_member(&mut ws, &alice, &bob, "editor").unwrap();

    // Outsider cannot send message to Bob in this workspace
    let res = FunctionalCoreCoordinator::send_workspace_message(
        &mut ws, &outsider, &bob, "msg-evil", "Spam",
    );
    assert!(matches!(
        res,
        Err(FunctionalCoreError::UnauthorizedAction { .. })
    ));
}

#[test]
fn cross_org_isolation_is_maintained_for_identical_names() {
    let user_a = FunctionalCoreCoordinator::enroll_user("ua", "ua@gardens.org", "User A").unwrap();
    let user_b = FunctionalCoreCoordinator::enroll_user("ub", "ub@gardens.org", "User B").unwrap();

    // Two independent orgs created with same display name "Citizen Gardens"
    let org_a = FunctionalCoreCoordinator::create_organization(
        &user_a,
        "org:cg:boulder",
        "Citizen Gardens",
        false,
    )
    .unwrap();
    let org_b = FunctionalCoreCoordinator::create_organization(
        &user_b,
        "org:cg:denver",
        "Citizen Gardens",
        false,
    )
    .unwrap();

    let mut ws_a =
        FunctionalCoreCoordinator::create_workspace(&user_a, &org_a, "ws-cg-a", "Gardens A")
            .unwrap();
    let ws_b = FunctionalCoreCoordinator::create_workspace(&user_b, &org_b, "ws-cg-b", "Gardens B")
        .unwrap();

    FunctionalCoreCoordinator::mutate_workspace_state(&mut ws_a, &user_a, "secret", "boulder-data")
        .unwrap();

    assert_eq!(
        ws_a.shared_data.get("secret").map(String::as_str),
        Some("boulder-data")
    );
    assert_eq!(ws_b.shared_data.get("secret"), None);
}
