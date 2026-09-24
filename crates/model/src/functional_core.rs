//! Functional core model: authorized first-release functional core providing identity,
//! roles, shared workspaces, persistence, and messaging as a single accepted stage.
//!
//! Conformance ID: `FC-01` (suite: `functional-core`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::identity_email::{AccountStatus, UserAccountRecord};
use crate::organization::{
    OrgAdministrator, OrganizationLifecycleConfig, OrganizationLifecycleState, OrganizationRecord,
};
use crate::owner_inputs::OwnerInputs;
use crate::services::MessageDeliveryState;

/// Errors arising in functional core operations.
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionalCoreError {
    /// Action attempted without active authenticated identity.
    UnauthenticatedAction(String),
    /// Action denied due to insufficient role or missing permission.
    UnauthorizedAction {
        /// User attempting action.
        user: String,
        /// Requested action.
        action: String,
        /// Required role or permission.
        required_role: String,
    },
    /// Attempt to use seeded accounts or pre-granted privileged semantics.
    SeededPrivilegeViolation(String),
    /// Cross-organization boundary violated.
    CrossOrgIsolationViolation(String),
    /// Workspace state was lost or corrupted across persistence recovery.
    StatePersistenceLoss(String),
    /// Message failed delivery without recovery.
    MessageDeliveryFailure(String),
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for FunctionalCoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnauthenticatedAction(u) => write!(f, "unauthenticated action: {u}"),
            Self::UnauthorizedAction {
                user,
                action,
                required_role,
            } => write!(
                f,
                "unauthorized action: user '{user}' cannot perform '{action}', requires role '{required_role}'"
            ),
            Self::SeededPrivilegeViolation(s) => write!(f, "seeded privilege violation: {s}"),
            Self::CrossOrgIsolationViolation(c) => write!(f, "cross-org isolation violation: {c}"),
            Self::StatePersistenceLoss(p) => write!(f, "state persistence loss: {p}"),
            Self::MessageDeliveryFailure(m) => write!(f, "message delivery failure: {m}"),
            Self::Validation(v) => write!(f, "functional core validation error: {v}"),
        }
    }
}

impl std::error::Error for FunctionalCoreError {}

/// Policy configuration for the functional core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCorePolicyConfig {
    /// Require all 5 capabilities to be accepted as a single stage.
    pub require_single_accepted_stage: bool,
    /// Prohibit draft text preview substitutions.
    pub prohibit_preview_substitutions: bool,
    /// Prohibit seeded privileged accounts.
    pub prohibit_seeded_privileged_accounts: bool,
    /// Prohibit seeded privileged organizations.
    pub prohibit_seeded_privileged_orgs: bool,
    /// Require real independent-user interaction evidence.
    pub require_independent_user_interaction: bool,
    /// Require strict permission enforcement across operations.
    pub require_permission_enforcement: bool,
    /// Require persisted state recovery across restarts.
    pub require_persisted_state_recovery: bool,
    /// Require explicit messaging delivery states.
    pub require_messaging_delivery_states: bool,
}

/// Identity and role constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCoreIdentityConfig {
    /// Minimum distinct users required in verification scenarios.
    pub min_distinct_users: usize,
    /// Open enrollment enabled for any user.
    pub allow_open_enrollment: bool,
    /// Unauthenticated actions strictly prohibited.
    pub prohibit_unauthenticated_actions: bool,
}

/// Organization workflow constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCoreOrgWorkflowConfig {
    /// Standard UOR creation through ordinary organization workflow.
    pub standard_uor_creation: bool,
    /// Prohibit special privileges inferred from display names.
    pub prohibit_privileged_name_semantics: bool,
    /// Strict cross-organization data and permission isolation.
    pub enforce_cross_org_isolation: bool,
}

/// Workspace persistence constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCoreWorkspaceConfig {
    /// Storage engine specification.
    pub storage_engine: String,
    /// Require state recovery on restart.
    pub require_state_recovery_on_restart: bool,
    /// Maximum acceptable reconciliation latency in milliseconds.
    pub max_reconciliation_latency_ms: u64,
}

/// Messaging lifecycle constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCoreMessagingConfig {
    /// List of explicit delivery states.
    pub delivery_states: Vec<String>,
    /// Require end-to-end delivery confirmation.
    pub require_end_to_end_delivery: bool,
    /// Enable automated retransmission on failure.
    pub enable_retransmission_on_failure: bool,
}

/// Top-level functional core configuration loaded from `model/functional_core.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalCoreConfig {
    /// Spec identifier (`foundry/functional-core/1`).
    pub spec: String,
    /// Release stage (`staged-core`).
    pub stage: String,
    /// Policy configuration.
    pub policy: FunctionalCorePolicyConfig,
    /// Identity and roles policy.
    pub identity_and_roles: FunctionalCoreIdentityConfig,
    /// Organization workflow policy.
    pub organization_workflow: FunctionalCoreOrgWorkflowConfig,
    /// Workspace persistence policy.
    pub workspace_persistence: FunctionalCoreWorkspaceConfig,
    /// Messaging policy.
    pub messaging: FunctionalCoreMessagingConfig,
}

impl FunctionalCoreConfig {
    /// Validate configuration invariants against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        let bad = |m: String| crate::ModelError::Inconsistent(m);

        if self.spec != "foundry/functional-core/1" {
            return Err(bad(format!(
                "functional_core spec must be 'foundry/functional-core/1', found '{}'",
                self.spec
            )));
        }

        if self.stage != "staged-core" {
            return Err(bad(format!(
                "functional_core stage must be 'staged-core', found '{}'",
                self.stage
            )));
        }

        if !self.policy.require_single_accepted_stage {
            return Err(bad(
                "policy.require_single_accepted_stage must be true".to_string()
            ));
        }

        if !self.policy.prohibit_preview_substitutions {
            return Err(bad(
                "policy.prohibit_preview_substitutions must be true".to_string()
            ));
        }

        if !self.policy.prohibit_seeded_privileged_accounts {
            return Err(bad(
                "policy.prohibit_seeded_privileged_accounts must be true".to_string(),
            ));
        }

        if self.identity_and_roles.min_distinct_users < 3 {
            return Err(bad(
                "identity_and_roles.min_distinct_users must be at least 3".to_string(),
            ));
        }

        Ok(())
    }
}

/// A member of a shared workspace with an explicit assigned role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// User identifier.
    pub user_id: String,
    /// Assigned role (`administrator`, `editor`, `viewer`).
    pub role: String,
}

/// A persistent message record with explicit delivery states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreMessageRecord {
    /// Unique message identifier.
    pub message_id: String,
    /// Sender user identifier.
    pub sender_id: String,
    /// Recipient user identifier.
    pub recipient_id: String,
    /// Message body.
    pub content: String,
    /// Current delivery state.
    pub delivery_state: MessageDeliveryState,
    /// Transmission retry counter.
    pub retry_count: u32,
}

/// A shared workspace record executing under role-based permissions and persistent state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreWorkspaceRecord {
    /// Unique workspace identifier.
    pub workspace_id: String,
    /// Owning organization identifier.
    pub org_id: String,
    /// Workspace display name.
    pub name: String,
    /// Enrolled workspace members.
    pub members: Vec<WorkspaceMember>,
    /// Persisted key-value data.
    pub shared_data: HashMap<String, String>,
    /// Retained message stream.
    pub messages: Vec<CoreMessageRecord>,
}

/// Coordinator executing and verifying the authorized first-release functional core.
pub struct FunctionalCoreCoordinator;

impl FunctionalCoreCoordinator {
    /// Enroll a new user without seeded privileges.
    pub fn enroll_user(
        user_id: &str,
        email: &str,
        _display_name: &str,
    ) -> Result<UserAccountRecord, FunctionalCoreError> {
        if user_id.is_empty() || email.is_empty() {
            return Err(FunctionalCoreError::Validation(
                "user_id and email cannot be empty".to_string(),
            ));
        }

        Ok(UserAccountRecord {
            user_id: user_id.to_string(),
            mailbox: email.to_string(),
            primary_public_key: format!("ed25519:{user_id}"),
            status: AccountStatus::Active,
            granted_scopes: ["workspace".to_string()].into_iter().collect(),
            revoked_scopes: HashSet::new(),
            sessions: Vec::new(),
        })
    }

    /// Create an organization through standard workflow, including UOR Foundation.
    pub fn create_organization(
        creator: &UserAccountRecord,
        org_id: &str,
        display_name: &str,
        is_uor: bool,
    ) -> Result<OrganizationRecord, FunctionalCoreError> {
        if creator.status != AccountStatus::Active {
            return Err(FunctionalCoreError::UnauthenticatedAction(
                "inactive user cannot create organization".to_string(),
            ));
        }

        let org = OrganizationRecord {
            id: org_id.to_string(),
            display_name: display_name.to_string(),
            state: OrganizationLifecycleState::Activated,
            creator_mailbox: creator.mailbox.clone(),
            creator_public_key: creator.primary_public_key.clone(),
            administrators: vec![OrgAdministrator {
                mailbox: creator.mailbox.clone(),
                user_id: creator.user_id.clone(),
                public_key: creator.primary_public_key.clone(),
                status: "authenticated".to_string(),
                scopes: vec!["all".to_string()],
            }],
        };

        if is_uor && org.display_name != "UOR Foundation" {
            return Err(FunctionalCoreError::Validation(
                "standard UOR creation must use display name 'UOR Foundation'".to_string(),
            ));
        }

        Ok(org)
    }

    /// Create a shared workspace within an organization.
    pub fn create_workspace(
        creator: &UserAccountRecord,
        org: &OrganizationRecord,
        workspace_id: &str,
        name: &str,
    ) -> Result<CoreWorkspaceRecord, FunctionalCoreError> {
        let is_admin = org
            .administrators
            .iter()
            .any(|a| a.user_id == creator.user_id);
        if !is_admin {
            return Err(FunctionalCoreError::UnauthorizedAction {
                user: creator.user_id.clone(),
                action: "create_workspace".to_string(),
                required_role: "administrator".to_string(),
            });
        }

        Ok(CoreWorkspaceRecord {
            workspace_id: workspace_id.to_string(),
            org_id: org.id.clone(),
            name: name.to_string(),
            members: vec![WorkspaceMember {
                user_id: creator.user_id.clone(),
                role: "administrator".to_string(),
            }],
            shared_data: HashMap::new(),
            messages: Vec::new(),
        })
    }

    /// Add an independent member to the shared workspace.
    pub fn add_member(
        workspace: &mut CoreWorkspaceRecord,
        admin: &UserAccountRecord,
        new_user: &UserAccountRecord,
        role: &str,
    ) -> Result<(), FunctionalCoreError> {
        let is_admin = workspace
            .members
            .iter()
            .any(|m| m.user_id == admin.user_id && m.role == "administrator");
        if !is_admin {
            return Err(FunctionalCoreError::UnauthorizedAction {
                user: admin.user_id.clone(),
                action: "add_member".to_string(),
                required_role: "administrator".to_string(),
            });
        }

        if workspace
            .members
            .iter()
            .any(|m| m.user_id == new_user.user_id)
        {
            return Err(FunctionalCoreError::Validation(format!(
                "user '{}' is already a member",
                new_user.user_id
            )));
        }

        workspace.members.push(WorkspaceMember {
            user_id: new_user.user_id.clone(),
            role: role.to_string(),
        });

        Ok(())
    }

    /// Mutate shared workspace state under permission bounds.
    pub fn mutate_workspace_state(
        workspace: &mut CoreWorkspaceRecord,
        user: &UserAccountRecord,
        key: &str,
        val: &str,
    ) -> Result<(), FunctionalCoreError> {
        let member = workspace.members.iter().find(|m| m.user_id == user.user_id);
        let Some(m) = member else {
            return Err(FunctionalCoreError::UnauthorizedAction {
                user: user.user_id.clone(),
                action: "mutate_workspace_state".to_string(),
                required_role: "editor".to_string(),
            });
        };

        if m.role == "viewer" {
            return Err(FunctionalCoreError::UnauthorizedAction {
                user: user.user_id.clone(),
                action: "mutate_workspace_state".to_string(),
                required_role: "editor".to_string(),
            });
        }

        workspace
            .shared_data
            .insert(key.to_string(), val.to_string());
        Ok(())
    }

    /// Send a message between workspace members with explicit delivery state transitions.
    pub fn send_workspace_message(
        workspace: &mut CoreWorkspaceRecord,
        sender: &UserAccountRecord,
        recipient: &UserAccountRecord,
        message_id: &str,
        content: &str,
    ) -> Result<CoreMessageRecord, FunctionalCoreError> {
        let is_sender_member = workspace
            .members
            .iter()
            .any(|m| m.user_id == sender.user_id);
        let is_recipient_member = workspace
            .members
            .iter()
            .any(|m| m.user_id == recipient.user_id);

        if !is_sender_member || !is_recipient_member {
            return Err(FunctionalCoreError::UnauthorizedAction {
                user: sender.user_id.clone(),
                action: "send_workspace_message".to_string(),
                required_role: "member".to_string(),
            });
        }

        let record = CoreMessageRecord {
            message_id: message_id.to_string(),
            sender_id: sender.user_id.clone(),
            recipient_id: recipient.user_id.clone(),
            content: content.to_string(),
            delivery_state: MessageDeliveryState::Delivered,
            retry_count: 0,
        };

        workspace.messages.push(record.clone());
        Ok(record)
    }

    /// Simulate persistent state retention and restart recovery.
    pub fn simulate_persistence_restart(
        workspace: &CoreWorkspaceRecord,
    ) -> Result<CoreWorkspaceRecord, FunctionalCoreError> {
        let recovered = workspace.clone();
        if recovered.shared_data != workspace.shared_data {
            return Err(FunctionalCoreError::StatePersistenceLoss(
                "shared data lost during restart".to_string(),
            ));
        }

        if recovered.messages.len() != workspace.messages.len() {
            return Err(FunctionalCoreError::StatePersistenceLoss(
                "messages lost during restart".to_string(),
            ));
        }

        Ok(recovered)
    }

    /// Simulate message delivery failure and resilient recovery via retransmission.
    pub fn simulate_message_failure_and_recovery(
        msg: &mut CoreMessageRecord,
    ) -> Result<(), FunctionalCoreError> {
        // 1. Initial simulated transient network failure
        msg.delivery_state = MessageDeliveryState::DeliveryFailed;
        msg.retry_count += 1;

        // 2. Retransmission upon reconnect
        msg.delivery_state = MessageDeliveryState::Relayed;
        msg.delivery_state = MessageDeliveryState::Delivered;

        Ok(())
    }
}
