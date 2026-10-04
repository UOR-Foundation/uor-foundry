//! Conformance tests for organization communications, multi-channel messaging,
//! 25 MiB attachment boundaries, delivery states, and shared inbox notifications.

use repo_model::{
    AttachmentMetadata, ChannelRole, ChannelType, CommsError, CommsManager,
    DispatchNotificationRequest, MessageDeliveryState, NotificationEvent, NotificationSeverity,
    MAX_ATTACHMENT_BYTES, MAX_MESSAGE_PAYLOAD_BYTES,
};

#[test]
fn test_channel_creation_membership_and_cross_org_isolation() {
    let mut cm = CommsManager::new();
    let org_a = "uor:org:citizen-gardens-01";
    let org_b = "uor:org:metro-farms-02";

    // 1. Create public channel in org_a
    let c1 = cm
        .create_channel(
            "alice@uor.foundation",
            org_a,
            "uor:channel:cg-01:general",
            "#general",
            "General announcements and coordination",
            ChannelType::Public,
        )
        .expect("create general channel");

    assert_eq!(c1.members.len(), 1);
    assert_eq!(c1.members[0].role, ChannelRole::Owner);
    assert_eq!(c1.channel_type, ChannelType::Public);

    // 2. Add member
    cm.add_channel_member(
        "alice@uor.foundation",
        "uor:channel:cg-01:general",
        "bob@uor.foundation",
        ChannelRole::Member,
    )
    .expect("add bob to channel");

    // 3. Non-owner/non-moderator cannot add members
    let non_mod_err = cm
        .add_channel_member(
            "bob@uor.foundation",
            "uor:channel:cg-01:general",
            "charlie@uor.foundation",
            ChannelRole::Member,
        )
        .expect_err("non-mod cannot add members");
    assert!(matches!(
        non_mod_err,
        CommsError::UnauthorizedCommsAction { .. }
    ));

    // 4. Cross-org isolation
    let cross_err = cm
        .get_channel(org_b, "uor:channel:cg-01:general")
        .expect_err("org_b cannot read org_a channel");
    assert!(matches!(
        cross_err,
        CommsError::CrossOrgBoundaryViolation { .. }
    ));
}

#[test]
fn test_message_payload_bounds_and_delivery_lifecycle() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let chan_id = "uor:channel:cg-01:dev";

    cm.create_channel(
        "alice@uor.foundation",
        org_id,
        chan_id,
        "#dev",
        "Developer updates",
        ChannelType::Public,
    )
    .expect("create channel");

    // 1. Message within 64 KiB payload limit succeeds
    let valid_payload = "A".repeat(1024);
    let msg = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "uor:msg:dev:001",
            &valid_payload,
            None,
            None,
        )
        .expect("post message within bounds");

    assert_eq!(msg.delivery_state, MessageDeliveryState::Sent);
    assert!(msg.content_digest.starts_with("sha256:"));

    // 2. Delivery lifecycle transitions: Sent -> Delivered -> Acknowledged
    cm.mark_delivered("uor:msg:dev:001")
        .expect("mark delivered");
    let msgs = cm.list_messages_for_channel(chan_id);
    assert_eq!(msgs[0].delivery_state, MessageDeliveryState::Delivered);

    cm.acknowledge_receipt("uor:msg:dev:001")
        .expect("acknowledge receipt");
    let msgs = cm.list_messages_for_channel(chan_id);
    assert_eq!(msgs[0].delivery_state, MessageDeliveryState::Acknowledged);

    // 3. Exact boundary: exactly 65536 bytes succeeds
    let exact_payload = "B".repeat(MAX_MESSAGE_PAYLOAD_BYTES as usize);
    assert!(cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "uor:msg:dev:002",
            &exact_payload,
            None,
            None
        )
        .is_ok());

    // 4. One-over boundary: 65537 bytes rejected with MessageSizeExceeded
    let over_payload = "C".repeat((MAX_MESSAGE_PAYLOAD_BYTES + 1) as usize);
    let size_err = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "uor:msg:dev:003",
            &over_payload,
            None,
            None,
        )
        .expect_err("oversized message rejected");

    assert!(matches!(size_err, CommsError::MessageSizeExceeded { .. }));
}

#[test]
fn test_attachment_bounds_and_mime_validation() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let chan_id = "uor:channel:cg-01:media";

    cm.create_channel(
        "alice@uor.foundation",
        org_id,
        chan_id,
        "#media",
        "Media exchange",
        ChannelType::Public,
    )
    .expect("create media channel");

    // 1. Valid attachment metadata within 25 MiB
    let valid_meta = AttachmentMetadata::new(
        "garden_sensors.pdf",
        "application/pdf",
        5 * 1024 * 1024, // 5 MiB
        "Sensor telemetry specification manual",
    )
    .expect("valid attachment metadata");

    let blob_bytes = b"mock pdf content stream";
    let digest = format!("sha256:{}", CommsManager::sha256_bytes(blob_bytes));

    let msg = cm
        .post_message(
            "alice@uor.foundation",
            chan_id,
            "uor:msg:media:001",
            "Telemetry documentation attached",
            Some(digest),
            Some(valid_meta),
        )
        .expect("post message with attachment");

    assert!(msg.attachment_metadata.is_some());

    // 2. Exact 25 MiB boundary (26,214,400 bytes) succeeds
    let exact_meta = AttachmentMetadata::new(
        "large_bundle.mp4",
        "video/mp4",
        MAX_ATTACHMENT_BYTES,
        "High resolution garden camera footage",
    );
    assert!(exact_meta.is_ok());

    // 3. One-over 25 MiB boundary (26,214,401 bytes) rejected
    let over_meta_err = AttachmentMetadata::new(
        "oversized.mp4",
        "video/mp4",
        MAX_ATTACHMENT_BYTES + 1,
        "Over 25 MiB video",
    )
    .expect_err("oversized attachment rejected");

    assert!(matches!(
        over_meta_err,
        CommsError::AttachmentSizeExceeded { .. }
    ));

    // 4. Hostile / unapproved MIME types rejected
    let exe_err = AttachmentMetadata::new(
        "payload.exe",
        "application/x-msdownload",
        1024,
        "Executable",
    )
    .expect_err("exe MIME rejected");
    assert!(matches!(exe_err, CommsError::UnsupportedMimeType(_)));

    let html_err = AttachmentMetadata::new("phish.html", "text/html", 1024, "HTML")
        .expect_err("html rejected");
    assert!(matches!(html_err, CommsError::UnsupportedMimeType(_)));

    // 5. Unsafe filename path traversal rejected
    let path_err = AttachmentMetadata::new(
        "../../etc/passwd",
        "text/plain",
        100,
        "Path traversal attempt",
    )
    .expect_err("path traversal rejected");
    assert!(matches!(path_err, CommsError::InvalidFilename(_)));
}

#[test]
fn test_shared_inbox_and_security_notifications() {
    let mut cm = CommsManager::new();
    let org_id = "uor:org:citizen-gardens-01";
    let user = "alice@uor.foundation";

    // 1. Dispatch SecurityLogin notification
    cm.dispatch_notification(DispatchNotificationRequest {
        id: "notif-sec-01",
        org_id,
        recipient: user,
        event: NotificationEvent::SecurityLogin,
        severity: NotificationSeverity::Info,
        title: "New Session Authenticated",
        summary: "WebCrypto session established on Linux Firefox",
        route: "#panel-identity",
    })
    .expect("dispatch security login");

    // 2. Dispatch BackupCodeRedeemed notification
    cm.dispatch_notification(DispatchNotificationRequest {
        id: "notif-sec-02",
        org_id,
        recipient: user,
        event: NotificationEvent::BackupCodeRedeemed,
        severity: NotificationSeverity::Warning,
        title: "Backup Code Redeemed",
        summary: "Single-use recovery code redeemed. 7 codes remain.",
        route: "#panel-backup-codes",
    })
    .expect("dispatch backup code notification");

    // 3. Dispatch ProposalCreated notification
    cm.dispatch_notification(DispatchNotificationRequest {
        id: "notif-gov-03",
        org_id,
        recipient: user,
        event: NotificationEvent::ProposalCreated,
        severity: NotificationSeverity::Info,
        title: "New Governance Proposal",
        summary: "Quorum change proposal requires administrator review.",
        route: "#panel-governance",
    })
    .expect("dispatch proposal notification");

    // 4. Verify unread count and inbox contents
    assert_eq!(cm.unread_count(user), 3);
    let inbox = cm.list_inbox(user, true);
    assert_eq!(inbox.len(), 3);

    // 5. Mark individual notification read
    cm.mark_notification_read(user, "notif-sec-01")
        .expect("mark read");
    assert_eq!(cm.unread_count(user), 2);

    let unread_items = cm.list_inbox(user, true);
    assert_eq!(unread_items.len(), 2);
    assert_eq!(unread_items[0].id, "notif-sec-02");

    // 6. Mark all notifications read
    cm.mark_all_notifications_read(user);
    assert_eq!(cm.unread_count(user), 0);
    assert_eq!(cm.list_inbox(user, true).len(), 0);
    assert_eq!(cm.list_inbox(user, false).len(), 3);
}
