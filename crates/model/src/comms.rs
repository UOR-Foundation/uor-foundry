//! Domain model for organization communications, multi-channel messaging,
//! message delivery progression, shared inboxes, and multimedia attachments.

use crate::services::MessageDeliveryState;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Maximum allowable text message payload in bytes (64 KiB).
pub const MAX_MESSAGE_PAYLOAD_BYTES: u64 = 65536;

/// Maximum allowable multimedia attachment size in bytes (25 MiB = 26,214,400 bytes).
pub const MAX_ATTACHMENT_BYTES: u64 = 26214400;

/// Permitted MIME types for secure attachments.
pub const MIME_WHITELIST: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/svg+xml",
    "audio/mpeg",
    "audio/ogg",
    "audio/wav",
    "video/mp4",
    "video/webm",
    "application/pdf",
    "text/plain",
    "application/json",
];

/// Visibility and access type of a communications channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    /// Public channel open to all members of the parent organization.
    Public,
    /// Private channel restricted to explicitly enrolled members.
    Private,
    /// Direct 1-on-1 messaging channel.
    DirectMessage,
}

impl ChannelType {
    /// String representation of the channel type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Private => "Private",
            Self::DirectMessage => "DirectMessage",
        }
    }
}

impl fmt::Display for ChannelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Role of a participant within a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelRole {
    /// Channel owner with administrative permissions.
    Owner,
    /// Channel moderator with moderation and membership management privileges.
    Moderator,
    /// Standard channel member with post and read privileges.
    Member,
}

impl ChannelRole {
    /// String representation of channel role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Moderator => "Moderator",
            Self::Member => "Member",
        }
    }
}

impl fmt::Display for ChannelRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Enrolled member in a communications channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelMember {
    /// Unique user identity of the participant.
    pub user_id: String,
    /// Role assigned within this channel.
    pub role: ChannelRole,
    /// ISO-8601 timestamp of enrollment.
    pub joined_at_iso: String,
}

/// Record representing a multi-user communications channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelRecord {
    /// Canonical channel identifier (e.g. `uor:channel:<org_id>:<slug>`).
    pub id: String,
    /// Parent organization identifier.
    pub org_id: String,
    /// Channel display name.
    pub name: String,
    /// Channel topic or purpose description.
    pub topic: String,
    /// Access control type.
    pub channel_type: ChannelType,
    /// User identifier of channel creator.
    pub created_by: String,
    /// Enrolled members.
    pub members: Vec<ChannelMember>,
    /// Creation timestamp (ISO-8601).
    pub created_at_iso: String,
    /// Whether the channel has been archived.
    pub is_archived: bool,
}

/// Metadata describing a multimedia attachment, enforcing the 25 MiB boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentMetadata {
    /// Original or sanitized filename.
    pub filename: String,
    /// Verified MIME type.
    pub mime_type: String,
    /// Size of the attachment in bytes. Must not exceed 26,214,400 bytes.
    pub size_bytes: u64,
    /// Accessible alt text or media description for WCAG 2.2 AA conformance.
    pub alt_text: String,
}

impl AttachmentMetadata {
    /// Create and validate attachment metadata against size bounds and allowed MIME types.
    pub fn new(
        filename: &str,
        mime_type: &str,
        size_bytes: u64,
        alt_text: &str,
    ) -> Result<Self, CommsError> {
        if size_bytes > MAX_ATTACHMENT_BYTES {
            return Err(CommsError::AttachmentSizeExceeded {
                size_bytes,
                limit_bytes: MAX_ATTACHMENT_BYTES,
            });
        }

        if !MIME_WHITELIST.contains(&mime_type) {
            return Err(CommsError::UnsupportedMimeType(mime_type.to_string()));
        }

        if filename.trim().is_empty() {
            return Err(CommsError::Validation(
                "filename cannot be empty".to_string(),
            ));
        }

        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(CommsError::InvalidFilename(filename.to_string()));
        }

        Ok(Self {
            filename: filename.to_string(),
            mime_type: mime_type.to_string(),
            size_bytes,
            alt_text: alt_text.to_string(),
        })
    }
}

/// Comprehensive record representing a collaborative text message with optional multimedia attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommsMessageRecord {
    /// Unique message identifier (e.g. `uor:msg:<channel_id>:<uuid>`).
    pub message_id: String,
    /// Parent channel identifier.
    pub channel_id: String,
    /// Parent organization identifier.
    pub org_id: String,
    /// Sender user identifier.
    pub sender_id: String,
    /// Message text payload.
    pub content: String,
    /// Cryptographic SHA-256 digest of the payload content.
    pub content_digest: String,
    /// Message delivery lifecycle state.
    pub delivery_state: MessageDeliveryState,
    /// ISO-8601 creation timestamp.
    pub timestamp: String,
    /// Optional Kappa content-addressed blob digest of attachment (e.g. `sha256:<hex>`).
    pub attachment_digest: Option<String>,
    /// Optional metadata for the attached media.
    pub attachment_metadata: Option<AttachmentMetadata>,
    /// Number of delivery transmission attempts.
    pub retry_count: u32,
    /// Whether the message has been soft-deleted.
    pub is_deleted: bool,
}

/// Severity classification of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationSeverity {
    /// Informational message.
    Info,
    /// Warning requiring user attention.
    Warning,
    /// Critical alert or security notification.
    Critical,
}

impl NotificationSeverity {
    /// String representation of notification severity.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
        }
    }
}

impl fmt::Display for NotificationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Security and workflow events triggering inbox notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationEvent {
    /// User authenticated a new session.
    SecurityLogin,
    /// Single-use backup code was redeemed.
    BackupCodeRedeemed,
    /// Organization invitation was received.
    InvitationReceived,
    /// Governance policy or quorum proposal created.
    ProposalCreated,
    /// Project milestone was completed and verified.
    MilestoneCompleted,
}

impl NotificationEvent {
    /// String representation of notification event.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SecurityLogin => "SecurityLogin",
            Self::BackupCodeRedeemed => "BackupCodeRedeemed",
            Self::InvitationReceived => "InvitationReceived",
            Self::ProposalCreated => "ProposalCreated",
            Self::MilestoneCompleted => "MilestoneCompleted",
        }
    }
}

impl fmt::Display for NotificationEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Read state of an inbox notification item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationReadState {
    /// Notification is unread.
    Unread,
    /// Notification has been read.
    Read,
    /// Notification has been dismissed.
    Dismissed,
}

impl NotificationReadState {
    /// String representation of read state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unread => "Unread",
            Self::Read => "Read",
            Self::Dismissed => "Dismissed",
        }
    }
}

impl fmt::Display for NotificationReadState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Notification item dispatched to a user's shared inbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Unique notification identifier.
    pub id: String,
    /// Parent organization identifier.
    pub org_id: String,
    /// Recipient user identifier.
    pub recipient: String,
    /// Event category.
    pub event: NotificationEvent,
    /// Severity level.
    pub severity: NotificationSeverity,
    /// Notification title.
    pub title: String,
    /// Notification summary text.
    pub summary: String,
    /// Interactive application route target (e.g. "#panel-identity", "#panel-governance").
    pub route: String,
    /// Current read state.
    pub read_state: NotificationReadState,
    /// ISO-8601 timestamp when created.
    pub timestamp: String,
}

/// Request parameters for dispatching a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchNotificationRequest<'a> {
    /// Notification ID.
    pub id: &'a str,
    /// Parent organization identifier.
    pub org_id: &'a str,
    /// Recipient user identifier.
    pub recipient: &'a str,
    /// Event category.
    pub event: NotificationEvent,
    /// Severity level.
    pub severity: NotificationSeverity,
    /// Notification title.
    pub title: &'a str,
    /// Notification summary text.
    pub summary: &'a str,
    /// Interactive application route target.
    pub route: &'a str,
}

/// Errors returned by communications operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommsError {
    /// Channel was not found.
    ChannelNotFound(String),
    /// Duplicate channel identifier.
    DuplicateChannelId(String),
    /// Cross-organization boundary isolation violation.
    CrossOrgBoundaryViolation {
        /// Target organization.
        org_id: String,
        /// Channel attempted to be accessed.
        channel_id: String,
    },
    /// Unauthorized action by an actor in a channel or inbox.
    UnauthorizedCommsAction {
        /// Actor identity.
        actor: String,
        /// Action attempted.
        action: String,
        /// Required role or permission.
        required: String,
    },
    /// Message payload exceeded 64 KiB limit.
    MessageSizeExceeded {
        /// Measured payload size in bytes.
        size_bytes: u64,
        /// Configured limit in bytes.
        limit_bytes: u64,
    },
    /// Attachment size exceeded 25 MiB limit.
    AttachmentSizeExceeded {
        /// Measured attachment size in bytes.
        size_bytes: u64,
        /// Configured limit in bytes.
        limit_bytes: u64,
    },
    /// MIME type is not allowed under security whitelist.
    UnsupportedMimeType(String),
    /// Filename contains illegal characters or path traversal.
    InvalidFilename(String),
    /// Message was not found.
    MessageNotFound(String),
    /// Notification was not found.
    NotificationNotFound(String),
    /// General validation failure.
    Validation(String),
}

impl fmt::Display for CommsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChannelNotFound(id) => write!(f, "channel '{id}' not found"),
            Self::DuplicateChannelId(id) => write!(f, "duplicate channel id '{id}'"),
            Self::CrossOrgBoundaryViolation { org_id, channel_id } => write!(
                f,
                "cross-org boundary violation: channel '{channel_id}' does not belong to '{org_id}'"
            ),
            Self::UnauthorizedCommsAction {
                actor,
                action,
                required,
            } => write!(
                f,
                "actor '{actor}' unauthorized for '{action}': requires '{required}'"
            ),
            Self::MessageSizeExceeded {
                size_bytes,
                limit_bytes,
            } => write!(
                f,
                "message payload {size_bytes} bytes exceeds maximum limit {limit_bytes} bytes"
            ),
            Self::AttachmentSizeExceeded {
                size_bytes,
                limit_bytes,
            } => write!(
                f,
                "attachment {size_bytes} bytes exceeds maximum limit {limit_bytes} bytes (25 MiB)"
            ),
            Self::UnsupportedMimeType(mime) => write!(f, "unsupported MIME type '{mime}'"),
            Self::InvalidFilename(name) => write!(f, "invalid or unsafe filename '{name}'"),
            Self::MessageNotFound(id) => write!(f, "message '{id}' not found"),
            Self::NotificationNotFound(id) => write!(f, "notification '{id}' not found"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for CommsError {}

/// Manager governing multi-channel messaging, message delivery state transitions,
/// attachment boundary validation, and notification inboxes.
#[derive(Debug, Default)]
pub struct CommsManager {
    channels: Vec<ChannelRecord>,
    messages: Vec<CommsMessageRecord>,
    notifications: Vec<NotificationRecord>,
}

impl CommsManager {
    /// Create a new, empty communications manager.
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            messages: Vec::new(),
            notifications: Vec::new(),
        }
    }

    /// Compute genuine SHA-256 hex digest of string content.
    pub fn sha256_hex(data: &str) -> String {
        Self::sha256_bytes(data.as_bytes())
    }

    /// Compute genuine SHA-256 hex digest of binary bytes.
    pub fn sha256_bytes(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for byte in result {
            use std::fmt::Write;
            let _ = write!(hex, "{:02x}", byte);
        }
        hex
    }

    /// Create a new channel within an organization.
    pub fn create_channel(
        &mut self,
        caller_id: &str,
        org_id: &str,
        channel_id: &str,
        name: &str,
        topic: &str,
        channel_type: ChannelType,
    ) -> Result<&ChannelRecord, CommsError> {
        if channel_id.trim().is_empty() {
            return Err(CommsError::Validation(
                "channel ID cannot be empty".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(CommsError::Validation(
                "channel name cannot be empty".to_string(),
            ));
        }
        if self.channels.iter().any(|c| c.id == channel_id) {
            return Err(CommsError::DuplicateChannelId(channel_id.to_string()));
        }

        let now_iso = "2026-10-03T07:00:00Z".to_string();
        let initial_member = ChannelMember {
            user_id: caller_id.to_string(),
            role: ChannelRole::Owner,
            joined_at_iso: now_iso.clone(),
        };

        let channel = ChannelRecord {
            id: channel_id.to_string(),
            org_id: org_id.to_string(),
            name: name.to_string(),
            topic: topic.to_string(),
            channel_type,
            created_by: caller_id.to_string(),
            members: vec![initial_member],
            created_at_iso: now_iso,
            is_archived: false,
        };

        self.channels.push(channel);
        Ok(self.channels.last().expect("channel just pushed"))
    }

    /// Retrieve a channel by ID, checking organization isolation.
    pub fn get_channel(
        &self,
        caller_org_id: &str,
        channel_id: &str,
    ) -> Result<&ChannelRecord, CommsError> {
        let channel = self
            .channels
            .iter()
            .find(|c| c.id == channel_id)
            .ok_or_else(|| CommsError::ChannelNotFound(channel_id.to_string()))?;

        if channel.org_id != caller_org_id {
            return Err(CommsError::CrossOrgBoundaryViolation {
                org_id: caller_org_id.to_string(),
                channel_id: channel_id.to_string(),
            });
        }

        Ok(channel)
    }

    /// List all channels for an organization.
    pub fn list_channels_for_org(&self, org_id: &str) -> Vec<&ChannelRecord> {
        self.channels
            .iter()
            .filter(|c| c.org_id == org_id)
            .collect()
    }

    /// Enroll a user in a channel.
    pub fn add_channel_member(
        &mut self,
        caller_id: &str,
        channel_id: &str,
        new_user_id: &str,
        role: ChannelRole,
    ) -> Result<(), CommsError> {
        let channel = self
            .channels
            .iter_mut()
            .find(|c| c.id == channel_id)
            .ok_or_else(|| CommsError::ChannelNotFound(channel_id.to_string()))?;

        let can_manage = channel.members.iter().any(|m| {
            m.user_id == caller_id && matches!(m.role, ChannelRole::Owner | ChannelRole::Moderator)
        });
        if !can_manage {
            return Err(CommsError::UnauthorizedCommsAction {
                actor: caller_id.to_string(),
                action: "add_channel_member".to_string(),
                required: "Owner or Moderator".to_string(),
            });
        }

        if channel.members.iter().any(|m| m.user_id == new_user_id) {
            return Err(CommsError::Validation(format!(
                "user '{new_user_id}' is already a channel member"
            )));
        }

        channel.members.push(ChannelMember {
            user_id: new_user_id.to_string(),
            role,
            joined_at_iso: "2026-10-03T07:15:00Z".to_string(),
        });

        Ok(())
    }

    /// Post a new collaborative message to a channel with payload and attachment validation.
    pub fn post_message(
        &mut self,
        sender_id: &str,
        channel_id: &str,
        message_id: &str,
        content: &str,
        attachment_digest: Option<String>,
        attachment_metadata: Option<AttachmentMetadata>,
    ) -> Result<&CommsMessageRecord, CommsError> {
        let channel = self
            .channels
            .iter()
            .find(|c| c.id == channel_id)
            .ok_or_else(|| CommsError::ChannelNotFound(channel_id.to_string()))?;

        if channel.is_archived {
            return Err(CommsError::Validation(
                "cannot post to an archived channel".to_string(),
            ));
        }

        // For private channels, verify sender is an enrolled member
        if channel.channel_type == ChannelType::Private
            && !channel.members.iter().any(|m| m.user_id == sender_id)
        {
            return Err(CommsError::UnauthorizedCommsAction {
                actor: sender_id.to_string(),
                action: "post_message".to_string(),
                required: "Channel Membership".to_string(),
            });
        }

        let payload_bytes = content.len() as u64;
        if payload_bytes > MAX_MESSAGE_PAYLOAD_BYTES {
            return Err(CommsError::MessageSizeExceeded {
                size_bytes: payload_bytes,
                limit_bytes: MAX_MESSAGE_PAYLOAD_BYTES,
            });
        }

        if let Some(ref meta) = attachment_metadata {
            if meta.size_bytes > MAX_ATTACHMENT_BYTES {
                return Err(CommsError::AttachmentSizeExceeded {
                    size_bytes: meta.size_bytes,
                    limit_bytes: MAX_ATTACHMENT_BYTES,
                });
            }
        }

        let content_digest = format!("sha256:{}", Self::sha256_hex(content));
        let record = CommsMessageRecord {
            message_id: message_id.to_string(),
            channel_id: channel_id.to_string(),
            org_id: channel.org_id.clone(),
            sender_id: sender_id.to_string(),
            content: content.to_string(),
            content_digest,
            delivery_state: MessageDeliveryState::Sent,
            timestamp: "2026-10-03T07:20:00Z".to_string(),
            attachment_digest,
            attachment_metadata,
            retry_count: 0,
            is_deleted: false,
        };

        self.messages.push(record);
        Ok(self.messages.last().expect("message just posted"))
    }

    /// Advance message delivery state to `Delivered`.
    pub fn mark_delivered(&mut self, message_id: &str) -> Result<(), CommsError> {
        let msg = self
            .messages
            .iter_mut()
            .find(|m| m.message_id == message_id)
            .ok_or_else(|| CommsError::MessageNotFound(message_id.to_string()))?;

        msg.delivery_state = MessageDeliveryState::Delivered;
        Ok(())
    }

    /// Advance message delivery state to `Acknowledged`.
    pub fn acknowledge_receipt(&mut self, message_id: &str) -> Result<(), CommsError> {
        let msg = self
            .messages
            .iter_mut()
            .find(|m| m.message_id == message_id)
            .ok_or_else(|| CommsError::MessageNotFound(message_id.to_string()))?;

        msg.delivery_state = MessageDeliveryState::Acknowledged;
        Ok(())
    }

    /// Retrieve messages for a channel.
    pub fn list_messages_for_channel(&self, channel_id: &str) -> Vec<&CommsMessageRecord> {
        self.messages
            .iter()
            .filter(|m| m.channel_id == channel_id && !m.is_deleted)
            .collect()
    }

    /// Dispatch a security or workflow notification to a recipient's inbox.
    pub fn dispatch_notification(
        &mut self,
        req: DispatchNotificationRequest<'_>,
    ) -> Result<&NotificationRecord, CommsError> {
        let notification = NotificationRecord {
            id: req.id.to_string(),
            org_id: req.org_id.to_string(),
            recipient: req.recipient.to_string(),
            event: req.event,
            severity: req.severity,
            title: req.title.to_string(),
            summary: req.summary.to_string(),
            route: req.route.to_string(),
            read_state: NotificationReadState::Unread,
            timestamp: "2026-10-03T07:25:00Z".to_string(),
        };

        self.notifications.push(notification);
        Ok(self.notifications.last().expect("notification just pushed"))
    }

    /// List notifications for a recipient, optionally filtering for unread only.
    pub fn list_inbox(&self, recipient: &str, unread_only: bool) -> Vec<&NotificationRecord> {
        self.notifications
            .iter()
            .filter(|n| {
                n.recipient == recipient
                    && (!unread_only || n.read_state == NotificationReadState::Unread)
            })
            .collect()
    }

    /// Get unread notification count for a recipient.
    pub fn unread_count(&self, recipient: &str) -> usize {
        self.notifications
            .iter()
            .filter(|n| n.recipient == recipient && n.read_state == NotificationReadState::Unread)
            .count()
    }

    /// Mark an individual notification as read.
    pub fn mark_notification_read(
        &mut self,
        recipient: &str,
        notification_id: &str,
    ) -> Result<(), CommsError> {
        let notif = self
            .notifications
            .iter_mut()
            .find(|n| n.id == notification_id && n.recipient == recipient)
            .ok_or_else(|| CommsError::NotificationNotFound(notification_id.to_string()))?;

        notif.read_state = NotificationReadState::Read;
        Ok(())
    }

    /// Mark all unread notifications for a recipient as read.
    pub fn mark_all_notifications_read(&mut self, recipient: &str) {
        for n in self
            .notifications
            .iter_mut()
            .filter(|n| n.recipient == recipient)
        {
            n.read_state = NotificationReadState::Read;
        }
    }
}
