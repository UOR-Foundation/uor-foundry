//! Adversarial stress test harness for Milestone 3 Communications lifecycle.
//! Probes boundary conditions, adversarial inputs, MIME whitelisting,
//! cross-org channel isolation, delivery progression, and shared inbox accounting.

use repo_model::{
    AttachmentMetadata, ChannelRole, ChannelType, CommsError, CommsManager,
    DispatchNotificationRequest, MessageDeliveryState, NotificationEvent, NotificationReadState,
    NotificationSeverity, MAX_ATTACHMENT_BYTES, MAX_MESSAGE_PAYLOAD_BYTES, MIME_WHITELIST,
};

#[test]
fn test_stress_message_payload_boundaries() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:adversarial-stress-01";
    let chan_id = "uor:channel:stress:payload";

    cm.create_channel(
        "alice@uor.foundation",
        org_id,
        chan_id,
        "#payload-stress",
        "Payload limits stress testing",
        ChannelType::Public,
    )
    .expect("create test channel");

    // 1. Empty message payload (0 bytes)
    let msg_empty = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-0-empty",
            "",
            None,
            None,
        )
        .expect("empty payload allowed");
    assert_eq!(msg_empty.content, "");
    assert_eq!(
        msg_empty.content_digest,
        format!("sha256:{}", CommsManager::sha256_hex(""))
    );

    // 2. Exact boundary minus one: 65,535 bytes
    let payload_65535 = "X".repeat((MAX_MESSAGE_PAYLOAD_BYTES - 1) as usize);
    let msg_65535 = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-65535",
            &payload_65535,
            None,
            None,
        )
        .expect("65535 bytes must succeed");
    assert_eq!(msg_65535.content.len(), 65535);

    // 3. Exact boundary: 65,536 bytes (64 KiB)
    let payload_65536 = "Y".repeat(MAX_MESSAGE_PAYLOAD_BYTES as usize);
    let msg_65536 = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-65536",
            &payload_65536,
            None,
            None,
        )
        .expect("65536 bytes must succeed");
    assert_eq!(msg_65536.content.len(), 65536);
    assert_eq!(
        msg_65536.content_digest,
        format!("sha256:{}", CommsManager::sha256_hex(&payload_65536))
    );

    // 4. Exact boundary plus one: 65,537 bytes -> MUST FAIL
    let payload_65537 = "Z".repeat((MAX_MESSAGE_PAYLOAD_BYTES + 1) as usize);
    let err_65537 = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-65537",
            &payload_65537,
            None,
            None,
        )
        .expect_err("65537 bytes must be rejected");
    assert_eq!(
        err_65537,
        CommsError::MessageSizeExceeded {
            size_bytes: 65537,
            limit_bytes: 65536,
        }
    );

    // 5. UTF-8 multi-byte boundary: 65534 ASCII bytes + 3-byte Euro sign (€ = 3 bytes in UTF-8)
    // Total byte length: 65534 + 3 = 65537 bytes -> MUST FAIL
    let mut payload_utf8 = "A".repeat(65534);
    payload_utf8.push('€');
    assert_eq!(payload_utf8.len(), 65537);
    let err_utf8 = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-utf8-overflow",
            &payload_utf8,
            None,
            None,
        )
        .expect_err("UTF-8 byte length overflow must be rejected");
    assert_eq!(
        err_utf8,
        CommsError::MessageSizeExceeded {
            size_bytes: 65537,
            limit_bytes: 65536,
        }
    );

    // 6. Massive payload: 1 MiB -> MUST FAIL
    let payload_1mb = "M".repeat(1024 * 1024);
    let err_1mb = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "msg-1mb",
            &payload_1mb,
            None,
            None,
        )
        .expect_err("1 MiB payload must be rejected");
    assert_eq!(
        err_1mb,
        CommsError::MessageSizeExceeded {
            size_bytes: 1024 * 1024,
            limit_bytes: 65536,
        }
    );
}

#[test]
fn test_stress_attachment_bounds_and_hostile_inputs() {
    // 1. Boundary tests on AttachmentMetadata
    // Exact 25 MiB: 26,214,400 bytes -> Ok
    let exact_meta = AttachmentMetadata::new(
        "archive.zip",
        "application/json",
        MAX_ATTACHMENT_BYTES,
        "JSON specification dataset",
    );
    assert!(exact_meta.is_ok());

    // Exact 25 MiB + 1: 26,214,401 bytes -> FAIL
    let overflow_meta = AttachmentMetadata::new(
        "archive.zip",
        "application/json",
        MAX_ATTACHMENT_BYTES + 1,
        "Oversized file",
    );
    assert_eq!(
        overflow_meta.unwrap_err(),
        CommsError::AttachmentSizeExceeded {
            size_bytes: MAX_ATTACHMENT_BYTES + 1,
            limit_bytes: MAX_ATTACHMENT_BYTES,
        }
    );

    // 50 MiB attachment -> FAIL
    let huge_meta =
        AttachmentMetadata::new("huge.pdf", "application/pdf", 50 * 1024 * 1024, "Huge PDF");
    assert_eq!(
        huge_meta.unwrap_err(),
        CommsError::AttachmentSizeExceeded {
            size_bytes: 50 * 1024 * 1024,
            limit_bytes: MAX_ATTACHMENT_BYTES,
        }
    );

    // 2. MIME Whitelist coverage verification
    for allowed_mime in MIME_WHITELIST {
        let meta = AttachmentMetadata::new("valid_file.bin", allowed_mime, 1024, "Valid");
        assert!(
            meta.is_ok(),
            "MIME '{}' must be accepted per whitelist",
            allowed_mime
        );
    }

    // Hostile and disallowed MIME types
    let disallowed_mimes = [
        "application/x-sh",
        "application/x-executable",
        "application/x-msdownload",
        "application/javascript",
        "text/html",
        "application/octet-stream",
        "application/wasm",
        "image/bmp",
        "image/gif",
        "",
        "unknown/format",
    ];

    for hostile_mime in disallowed_mimes {
        let err = AttachmentMetadata::new("payload.test", hostile_mime, 512, "Hostile")
            .expect_err(&format!("MIME '{}' should be rejected", hostile_mime));
        assert_eq!(
            err,
            CommsError::UnsupportedMimeType(hostile_mime.to_string())
        );
    }

    // 3. Filename safety & path traversal rejection
    let malicious_filenames = [
        "../../etc/shadow",
        "../../../root/.ssh/id_rsa",
        "C:\\Windows\\System32\\cmd.exe",
        "/etc/passwd",
        "sub/dir/file.pdf",
        "dir\\file.pdf",
        "..",
    ];

    for bad_name in malicious_filenames {
        let err = AttachmentMetadata::new(bad_name, "application/pdf", 1024, "Traversal")
            .expect_err(&format!("Filename '{}' must be rejected", bad_name));
        assert_eq!(err, CommsError::InvalidFilename(bad_name.to_string()));
    }

    // Empty or whitespace filenames
    assert_eq!(
        AttachmentMetadata::new("", "application/pdf", 1024, "Empty").unwrap_err(),
        CommsError::Validation("filename cannot be empty".to_string())
    );
    assert_eq!(
        AttachmentMetadata::new("   ", "application/pdf", 1024, "Whitespace").unwrap_err(),
        CommsError::Validation("filename cannot be empty".to_string())
    );

    // 4. End-to-end post_message with valid attachment metadata
    let mut cm = CommsManager::new();
    cm.create_channel(
        "alice@uor.foundation",
        "uor:org:attachments",
        "uor:channel:att:01",
        "#attachments",
        "Attachments",
        ChannelType::Public,
    )
    .expect("create channel");

    let valid_meta = AttachmentMetadata::new(
        "report.pdf",
        "application/pdf",
        2 * 1024 * 1024,
        "Quarterly Report PDF",
    )
    .expect("valid metadata");

    let sample_bytes = b"PDF-1.7 authoritative report content";
    let computed_digest = format!("sha256:{}", CommsManager::sha256_bytes(sample_bytes));

    let msg = cm
        .post_message(
            "alice@uor.foundation",
            "uor:channel:att:01",
            "msg-att-01",
            "Please find the quarterly report attached.",
            Some(computed_digest.clone()),
            Some(valid_meta),
        )
        .expect("post with attachment");

    assert_eq!(msg.attachment_digest, Some(computed_digest));
    assert_eq!(
        msg.attachment_metadata.as_ref().unwrap().filename,
        "report.pdf"
    );
}

#[test]
fn test_stress_channel_cross_org_isolation_and_permissions() {
    let mut cm = CommsManager::new();
    let org_a = "uor:org:citizen-gardens-01";
    let org_b = "uor:org:metro-farms-02";
    let org_c = "uor:org:solar-grid-03";

    // 1. Create channels in different orgs
    cm.create_channel(
        "alice@uor.foundation",
        org_a,
        "uor:channel:cg:finance",
        "#finance",
        "CG Finance",
        ChannelType::Private,
    )
    .expect("create chan A");

    cm.create_channel(
        "bob@uor.foundation",
        org_b,
        "uor:channel:mf:research",
        "#research",
        "MF Research",
        ChannelType::Public,
    )
    .expect("create chan B");

    // 2. Cross-org retrieval rejection
    let err_cross_1 = cm
        .get_channel(org_b, "uor:channel:cg:finance")
        .expect_err("org_b cannot read org_a's channel");
    assert_eq!(
        err_cross_1,
        CommsError::CrossOrgBoundaryViolation {
            org_id: org_b.to_string(),
            channel_id: "uor:channel:cg:finance".to_string(),
        }
    );

    let err_cross_2 = cm
        .get_channel(org_c, "uor:channel:mf:research")
        .expect_err("org_c cannot read org_b's channel");
    assert_eq!(
        err_cross_2,
        CommsError::CrossOrgBoundaryViolation {
            org_id: org_c.to_string(),
            channel_id: "uor:channel:mf:research".to_string(),
        }
    );

    // Non-existent channel
    let err_nonexistent = cm
        .get_channel(org_a, "uor:channel:does-not-exist")
        .expect_err("channel does not exist");
    assert_eq!(
        err_nonexistent,
        CommsError::ChannelNotFound("uor:channel:does-not-exist".to_string())
    );

    // 3. Organization listing isolation
    let org_a_channels = cm.list_channels_for_org(org_a);
    assert_eq!(org_a_channels.len(), 1);
    assert_eq!(org_a_channels[0].id, "uor:channel:cg:finance");

    let org_b_channels = cm.list_channels_for_org(org_b);
    assert_eq!(org_b_channels.len(), 1);
    assert_eq!(org_b_channels[0].id, "uor:channel:mf:research");

    let org_c_channels = cm.list_channels_for_org(org_c);
    assert_eq!(org_c_channels.len(), 0);

    // 4. Duplicate channel ID rejection
    let err_dup = cm
        .create_channel(
            "alice@uor.foundation",
            org_a,
            "uor:channel:cg:finance",
            "#finance-dup",
            "Duplicate ID",
            ChannelType::Public,
        )
        .expect_err("duplicate channel ID must fail");
    assert_eq!(
        err_dup,
        CommsError::DuplicateChannelId("uor:channel:cg:finance".to_string())
    );

    // Empty channel ID / name validation
    assert_eq!(
        cm.create_channel("a", org_a, "", "name", "top", ChannelType::Public)
            .unwrap_err(),
        CommsError::Validation("channel ID cannot be empty".to_string())
    );
    assert_eq!(
        cm.create_channel("a", org_a, "id", "   ", "top", ChannelType::Public)
            .unwrap_err(),
        CommsError::Validation("channel name cannot be empty".to_string())
    );

    // 5. Private channel membership enforcement
    // Non-member attempts to post in private channel -> Unauthorized
    let err_unauth_post = cm
        .post_message(
            "charlie@uor.foundation",
            "uor:channel:cg:finance",
            "msg-unauth-01",
            "Trying to spy on finance",
            None,
            None,
        )
        .expect_err("non-member cannot post to private channel");
    assert_eq!(
        err_unauth_post,
        CommsError::UnauthorizedCommsAction {
            actor: "charlie@uor.foundation".to_string(),
            action: "post_message".to_string(),
            required: "Channel Membership".to_string(),
        }
    );

    // Owner (Alice) adds Bob as Member
    cm.add_channel_member(
        "alice@uor.foundation",
        "uor:channel:cg:finance",
        "bob@uor.foundation",
        ChannelRole::Member,
    )
    .expect("alice can add bob");

    // Bob can now post
    let msg_bob = cm.post_message(
        "bob@uor.foundation",
        "uor:channel:cg:finance",
        "msg-bob-01",
        "Financial report received",
        None,
        None,
    );
    assert!(msg_bob.is_ok());

    // Bob (Member) attempts to add Charlie -> Unauthorized (requires Owner or Moderator)
    let err_bob_add = cm
        .add_channel_member(
            "bob@uor.foundation",
            "uor:channel:cg:finance",
            "charlie@uor.foundation",
            ChannelRole::Member,
        )
        .expect_err("member cannot add another member");
    assert_eq!(
        err_bob_add,
        CommsError::UnauthorizedCommsAction {
            actor: "bob@uor.foundation".to_string(),
            action: "add_channel_member".to_string(),
            required: "Owner or Moderator".to_string(),
        }
    );

    // Adding existing member -> Validation error
    let err_dup_member = cm
        .add_channel_member(
            "alice@uor.foundation",
            "uor:channel:cg:finance",
            "bob@uor.foundation",
            ChannelRole::Member,
        )
        .expect_err("cannot re-add bob");
    assert_eq!(
        err_dup_member,
        CommsError::Validation("user 'bob@uor.foundation' is already a channel member".to_string())
    );
}

#[test]
fn test_stress_delivery_progression_and_message_queries() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:delivery-01";
    let chan_1 = "uor:channel:del:01";
    let chan_2 = "uor:channel:del:02";

    cm.create_channel(
        "alice",
        org_id,
        chan_1,
        "#general",
        "Deliv",
        ChannelType::Public,
    )
    .unwrap();
    cm.create_channel(
        "alice",
        org_id,
        chan_2,
        "#random",
        "Deliv2",
        ChannelType::Public,
    )
    .unwrap();

    // Post to chan_1
    let m1 = cm
        .post_message("alice", chan_1, "msg-01", "Hello 1", None, None)
        .unwrap();
    assert_eq!(m1.delivery_state, MessageDeliveryState::Sent);

    // Advance m1 to Delivered
    cm.mark_delivered("msg-01").expect("mark delivered");
    let msgs = cm.list_messages_for_channel(chan_1);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].delivery_state, MessageDeliveryState::Delivered);

    // Advance m1 to Acknowledged
    cm.acknowledge_receipt("msg-01")
        .expect("acknowledge receipt");
    let msgs = cm.list_messages_for_channel(chan_1);
    assert_eq!(msgs[0].delivery_state, MessageDeliveryState::Acknowledged);

    // Advancing non-existent message returns MessageNotFound
    assert_eq!(
        cm.mark_delivered("non-existent-msg").unwrap_err(),
        CommsError::MessageNotFound("non-existent-msg".to_string())
    );
    assert_eq!(
        cm.acknowledge_receipt("non-existent-msg").unwrap_err(),
        CommsError::MessageNotFound("non-existent-msg".to_string())
    );

    // Post to chan_2 and verify channel separation
    cm.post_message("bob", chan_2, "msg-02", "Hello 2", None, None)
        .unwrap();
    let chan_1_msgs = cm.list_messages_for_channel(chan_1);
    let chan_2_msgs = cm.list_messages_for_channel(chan_2);
    assert_eq!(chan_1_msgs.len(), 1);
    assert_eq!(chan_1_msgs[0].message_id, "msg-01");
    assert_eq!(chan_2_msgs.len(), 1);
    assert_eq!(chan_2_msgs[0].message_id, "msg-02");
}

#[test]
fn test_stress_shared_inbox_and_notification_accounting() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:inbox-01";
    let user_a = "alice@uor.foundation";
    let user_b = "bob@uor.foundation";

    // 1. Dispatch all notification event types and severities to Alice
    let events = [
        (
            NotificationEvent::SecurityLogin,
            NotificationSeverity::Info,
            "Login",
        ),
        (
            NotificationEvent::BackupCodeRedeemed,
            NotificationSeverity::Warning,
            "Backup",
        ),
        (
            NotificationEvent::InvitationReceived,
            NotificationSeverity::Info,
            "Invite",
        ),
        (
            NotificationEvent::ProposalCreated,
            NotificationSeverity::Critical,
            "Proposal",
        ),
        (
            NotificationEvent::MilestoneCompleted,
            NotificationSeverity::Info,
            "Milestone",
        ),
    ];

    for (idx, (event, severity, label)) in events.iter().enumerate() {
        cm.dispatch_notification(DispatchNotificationRequest {
            id: &format!("notif-alice-{idx}"),
            org_id,
            recipient: user_a,
            event: *event,
            severity: *severity,
            title: &format!("Alice {label}"),
            summary: "Summary text",
            route: "#panel-identity",
        })
        .expect("dispatch to alice");
    }

    // Dispatch 2 notifications to Bob
    for idx in 0..2 {
        cm.dispatch_notification(DispatchNotificationRequest {
            id: &format!("notif-bob-{idx}"),
            org_id,
            recipient: user_b,
            event: NotificationEvent::SecurityLogin,
            severity: NotificationSeverity::Info,
            title: "Bob Login",
            summary: "Summary",
            route: "#panel-identity",
        })
        .expect("dispatch to bob");
    }

    // 2. Strict recipient inbox isolation and counter accounting
    assert_eq!(cm.unread_count(user_a), 5);
    assert_eq!(cm.unread_count(user_b), 2);
    assert_eq!(cm.unread_count("charlie@uor.foundation"), 0);

    let alice_unread = cm.list_inbox(user_a, true);
    assert_eq!(alice_unread.len(), 5);
    for n in &alice_unread {
        assert_eq!(n.recipient, user_a);
        assert_eq!(n.read_state, NotificationReadState::Unread);
    }

    // 3. Mark single notification read
    cm.mark_notification_read(user_a, "notif-alice-0")
        .expect("mark alice notif 0 read");
    assert_eq!(cm.unread_count(user_a), 4);
    assert_eq!(cm.unread_count(user_b), 2); // Bob untouched

    let alice_filtered = cm.list_inbox(user_a, true);
    assert_eq!(alice_filtered.len(), 4);
    assert!(!alice_filtered.iter().any(|n| n.id == "notif-alice-0"));

    let alice_all = cm.list_inbox(user_a, false);
    assert_eq!(alice_all.len(), 5);
    let read_item = alice_all
        .iter()
        .find(|n| n.id == "notif-alice-0")
        .expect("find notif 0");
    assert_eq!(read_item.read_state, NotificationReadState::Read);

    // 4. Marking non-existent or cross-user notification fails
    let err_cross_read = cm
        .mark_notification_read(user_b, "notif-alice-1")
        .expect_err("bob cannot mark alice's notification read");
    assert_eq!(
        err_cross_read,
        CommsError::NotificationNotFound("notif-alice-1".to_string())
    );

    // 5. Mark all read for Alice
    cm.mark_all_notifications_read(user_a);
    assert_eq!(cm.unread_count(user_a), 0);
    assert_eq!(cm.list_inbox(user_a, true).len(), 0);
    assert_eq!(cm.list_inbox(user_a, false).len(), 5);
    assert_eq!(cm.unread_count(user_b), 2); // Bob still has 2 unread
}
