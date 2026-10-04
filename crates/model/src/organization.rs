//! Organization lifecycle, provisional setup, activation, and scoped authority (OL-01).
//!
//! Under SPEC.md:
//! 1. Anyone may enroll and create an organization with any display name.
//! 2. Names are display data, not globally unique identifiers; names confer no privileges.
//! 3. Creation is authenticated to establish the creator's explicit grants in provisional scope only.
//! 4. Provisional setup is an explicit bootstrap condition (sole founding administrator allowed temporarily).
//! 5. Activation requires policy-compliant distinct-user coverage: at least 2 distinct active administrators,
//!    full scope quorum coverage, and prohibition of single-owner bypass.
//! 6. Retirement of founding grants requires full coverage by other distinct users.
//! 7. Cross-organization isolation is strictly enforced across records, grants, and effects.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::authority::ProposalApproval;
use crate::{ModelError, OwnerInputs};

/// Organization lifecycle states per SPEC.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrganizationLifecycleState {
    /// Provisional bootstrap setup; collecting records and enrolling administrators.
    Provisional,
    /// Policy-compliant active operation with full multi-administrator coverage.
    Activated,
    /// Operation suspended; state frozen pending recovery or administrative review.
    Suspended,
    /// Organization retired; records retained per audit obligations.
    Retired,
}

impl OrganizationLifecycleState {
    /// Display name of the state.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provisional => "provisional",
            Self::Activated => "activated",
            Self::Suspended => "suspended",
            Self::Retired => "retired",
        }
    }
}

/// An organization record with lifecycle state, scoped administrators, and isolation boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationRecord {
    /// Unique UOR-referenced organization identifier (e.g. `uor:org:uor-foundation`).
    pub id: String,
    /// Display name (not unique; names confer no privilege).
    pub display_name: String,
    /// Current lifecycle state.
    pub state: OrganizationLifecycleState,
    /// Founding creator mailbox.
    pub creator_mailbox: String,
    /// Founding creator public key.
    pub creator_public_key: String,
    /// Enrolled administrators for this organization.
    pub administrators: Vec<OrgAdministrator>,
}

/// Administrator enrolled within an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgAdministrator {
    /// Administrator email address.
    pub mailbox: String,
    /// User identifier.
    pub user_id: String,
    /// Encoded public key.
    pub public_key: String,
    /// Status: "authenticated" or "provisional".
    pub status: String,
    /// Authorized scopes (e.g. "organization", "security", "operations").
    pub scopes: Vec<String>,
}

/// Request to create a new provisional organization.
#[derive(Debug, Clone)]
pub struct CreateOrganizationRequest {
    /// Unique UOR organization identifier.
    pub id: String,
    /// Requested display name.
    pub display_name: String,
    /// Mailbox of the creator.
    pub creator_mailbox: String,
    /// User identifier of the creator.
    pub creator_user_id: String,
    /// Public key of the creator.
    pub creator_public_key: String,
    /// Initial creator scopes within the new organization.
    pub creator_scopes: Vec<String>,
}

/// Request to activate a provisional organization.
#[derive(Debug, Clone)]
pub struct ActivateOrganizationRequest {
    /// Target organization identifier.
    pub organization_id: String,
    /// List of distinct administrators fulfilling the activation policy.
    pub administrators: Vec<OrgAdministrator>,
    /// Mailboxes of administrators approving activation.
    pub approving_mailboxes: Vec<String>,
}

/// Request to retire a founding administrator grant.
#[derive(Debug, Clone)]
pub struct RetireFoundingGrantRequest {
    /// Target organization identifier.
    pub organization_id: String,
    /// Mailbox of the founding administrator being retired.
    pub founding_mailbox: String,
}

/// Cross-organization access request for isolation validation.
#[derive(Debug, Clone)]
pub struct CrossOrgAccessRequest {
    /// Calling organization ID.
    pub caller_org_id: String,
    /// Caller user mailbox.
    pub caller_mailbox: String,
    /// Target organization ID being accessed.
    pub target_org_id: String,
    /// Operation name.
    pub operation: String,
}

/// Request to update an organization's display name safely preserving immutable org_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrganizationRequest {
    /// Target organization identifier.
    pub organization_id: String,
    /// New display name (must be non-empty).
    pub new_display_name: String,
    /// Mailbox of the requesting administrator.
    pub requesting_mailbox: String,
}

/// Status of an organization member invitation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvitationStatus {
    /// Invitation is pending recipient acceptance or decline.
    Pending,
    /// Invitation was accepted and enrolled into organization administrators.
    Accepted,
    /// Invitation was explicitly declined by recipient.
    Declined,
    /// Invitation was revoked by an authorized administrator.
    Revoked,
    /// Invitation expired without action.
    Expired,
}

/// Invitation record for enrolling a member with scoped roles into an organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgInvitationRecord {
    /// Unique invitation identifier (e.g. `uor:invite:<uuid>`).
    pub id: String,
    /// Target organization identifier.
    pub organization_id: String,
    /// Mailbox of the invited recipient.
    pub target_mailbox: String,
    /// Administrative or member scopes granted upon acceptance.
    pub invited_scopes: Vec<String>,
    /// Mailbox of the inviting administrator.
    pub inviter_mailbox: String,
    /// User identifier of the inviting administrator.
    pub inviter_user_id: String,
    /// Current status of the invitation.
    pub status: InvitationStatus,
    /// Timestamp when invitation was issued.
    pub created_at: u64,
    /// Timestamp when invitation expires.
    pub expires_at: u64,
    /// Revision number of the organization authority record at issuance.
    pub revision_binding: u64,
}

/// Request to issue a new organization member invitation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvitationRequest {
    /// Unique invitation identifier.
    pub id: String,
    /// Target organization identifier.
    pub organization_id: String,
    /// Mailbox of the invited recipient.
    pub target_mailbox: String,
    /// Scopes to grant to the recipient upon acceptance.
    pub invited_scopes: Vec<String>,
    /// Mailbox of the inviting administrator.
    pub inviter_mailbox: String,
    /// User identifier of the inviting administrator.
    pub inviter_user_id: String,
    /// Issuance timestamp.
    pub created_at: u64,
    /// Expiration timestamp.
    pub expires_at: u64,
    /// Current organization revision binding.
    pub revision_binding: u64,
}

/// Top-level organization lifecycle model configuration loaded from `model/organization_lifecycle.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationLifecycleConfig {
    /// Schema specification.
    pub spec: String,
    /// Allowed lifecycle transitions.
    pub allowed_transitions: Vec<LifecycleTransitionRule>,
    /// Platform baseline organization rules.
    pub rules: OrganizationRules,
}

/// Transition rule between organization lifecycle states.
#[derive(Debug, Clone, Deserialize)]
pub struct LifecycleTransitionRule {
    /// Source state.
    pub from: String,
    /// Destination state.
    pub to: String,
    /// Minimum distinct administrators required for the transition.
    pub minimum_distinct_administrators: usize,
    /// Description of the rule.
    pub description: String,
}

/// Baseline organization rules enforced across the platform.
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationRules {
    /// Enrollment is open to anyone.
    pub open_enrollment: bool,
    /// Organization creation is open with arbitrary display names.
    pub open_organization_creation: bool,
    /// Names confer zero platform or cross-organization privileges.
    pub names_confer_no_privileges: bool,
    /// Founding single administrator allowed only in provisional state.
    pub allow_sole_founding_admin_in_provisional: bool,
    /// Activation requires at least this many distinct administrators.
    pub activation_minimum_distinct_administrators: usize,
    /// Activation requires distinct users across all affected scopes.
    pub require_distinct_users_per_scope: bool,
    /// Prohibit single-owner bypass during activation.
    pub prohibit_single_owner_bypass: bool,
    /// Founding grant retirement requires full coverage by remaining distinct users.
    pub retirement_requires_full_coverage: bool,
    /// Cross-organization data, keys, and effects are strictly isolated.
    pub enforce_cross_org_isolation: bool,
}

/// Error type for organization lifecycle and isolation operations.
#[derive(Debug, PartialEq, Eq)]
pub enum OrganizationError {
    /// Organization identifier is invalid or already registered.
    InvalidOrganizationId(String),
    /// Display name is invalid.
    InvalidDisplayName(String),
    /// Organization not found.
    OrganizationNotFound(String),
    /// Invalid state transition.
    InvalidStateTransition {
        /// Current state.
        current: String,
        /// Attempted destination state.
        target: String,
        /// Reason for failure.
        reason: String,
    },
    /// Insufficient distinct administrators for activation.
    InsufficientAdministrators {
        /// Number of distinct administrators found.
        actual: usize,
        /// Number required.
        required: usize,
    },
    /// Single-owner bypass attempt detected and rejected.
    SingleOwnerBypassRejected(String),
    /// Duplicate public key or email alias detected across administrators.
    DuplicateIdentityDisguise(String),
    /// Inadequate quorum coverage for an administrative scope.
    InadequateScopeCoverage {
        /// Scope that lacks coverage.
        scope: String,
        /// Number of distinct administrators covering this scope.
        covered: usize,
        /// Minimum required.
        required: usize,
    },
    /// Founding grant retirement rejected due to lack of replacement coverage.
    RetirementCoverageLacking(String),
    /// Cross-organization access denied (isolation boundary enforced).
    CrossOrgIsolationViolation {
        /// Calling organization.
        caller: String,
        /// Target organization.
        target: String,
    },
    /// Self-appointment or unapproved role elevation rejected.
    UnauthorizedRoleElevation(String),
    /// Dependent resources exist preventing retirement without force override.
    DependentResourcesActive(String),
    /// Invitation identifier not found.
    InvitationNotFound(String),
    /// Invitation expired before acceptance.
    InvitationExpired(String),
    /// Invalid invitation state transition or action.
    InvalidInvitationState(String),
    /// Recipient mailbox or key does not match invitation.
    InvitationRecipientMismatch(String),
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOrganizationId(msg) => write!(f, "invalid organization id: {msg}"),
            Self::InvalidDisplayName(msg) => write!(f, "invalid display name: {msg}"),
            Self::OrganizationNotFound(id) => write!(f, "organization not found: {id}"),
            Self::InvalidStateTransition {
                current,
                target,
                reason,
            } => {
                write!(f, "invalid transition from {current} to {target}: {reason}")
            }
            Self::InsufficientAdministrators { actual, required } => {
                write!(
                    f,
                    "insufficient administrators: {actual} provided, {required} required"
                )
            }
            Self::SingleOwnerBypassRejected(msg) => {
                write!(f, "single-owner bypass rejected: {msg}")
            }
            Self::DuplicateIdentityDisguise(msg) => {
                write!(f, "duplicate identity disguise rejected: {msg}")
            }
            Self::InadequateScopeCoverage {
                scope,
                covered,
                required,
            } => {
                write!(f, "inadequate coverage for scope '{scope}': {covered} covered, {required} required")
            }
            Self::RetirementCoverageLacking(msg) => {
                write!(f, "founding grant retirement rejected: {msg}")
            }
            Self::CrossOrgIsolationViolation { caller, target } => {
                write!(f, "cross-organization boundary violation: caller '{caller}' denied access to '{target}'")
            }
            Self::UnauthorizedRoleElevation(msg) => write!(f, "unauthorized role elevation: {msg}"),
            Self::DependentResourcesActive(msg) => write!(f, "dependent resources active: {msg}"),
            Self::InvitationNotFound(id) => write!(f, "invitation not found: {id}"),
            Self::InvitationExpired(id) => write!(f, "invitation expired: {id}"),
            Self::InvalidInvitationState(msg) => write!(f, "invalid invitation state: {msg}"),
            Self::InvitationRecipientMismatch(msg) => {
                write!(f, "invitation recipient mismatch: {msg}")
            }
        }
    }
}

impl std::error::Error for OrganizationError {}

/// Organization registry and lifecycle manager enforcing SPEC.md policies.
#[derive(Debug, Default)]
pub struct OrganizationManager {
    organizations: Vec<OrganizationRecord>,
    invitations: Vec<OrgInvitationRecord>,
}

impl OrganizationManager {
    /// Create a new empty organization manager.
    pub fn new() -> Self {
        Self {
            organizations: Vec::new(),
            invitations: Vec::new(),
        }
    }

    /// Create a new provisional organization (open enrollment and open creation).
    ///
    /// Anyone may create an organization with any display name, including names already used.
    /// Creation creates a provisional state with a sole founding administrator grant.
    pub fn create_organization(
        &mut self,
        req: CreateOrganizationRequest,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        if req.id.trim().is_empty() || !req.id.starts_with("uor:org:") {
            return Err(OrganizationError::InvalidOrganizationId(
                "organization id must start with uor:org:".to_string(),
            ));
        }
        if self.organizations.iter().any(|o| o.id == req.id) {
            return Err(OrganizationError::InvalidOrganizationId(format!(
                "organization id '{}' is already registered",
                req.id
            )));
        }
        if req.display_name.trim().is_empty() {
            return Err(OrganizationError::InvalidDisplayName(
                "display name must not be empty".to_string(),
            ));
        }
        if req.creator_mailbox.trim().is_empty() || !req.creator_mailbox.contains('@') {
            return Err(OrganizationError::SingleOwnerBypassRejected(
                "creator mailbox must be a valid email address".to_string(),
            ));
        }
        if req.creator_public_key.trim().is_empty() {
            return Err(OrganizationError::SingleOwnerBypassRejected(
                "creator public key must not be empty".to_string(),
            ));
        }

        let founding_admin = OrgAdministrator {
            mailbox: req.creator_mailbox.clone(),
            user_id: req.creator_user_id,
            public_key: req.creator_public_key.clone(),
            status: "authenticated".to_string(),
            scopes: if req.creator_scopes.is_empty() {
                vec!["organization".to_string(), "security".to_string()]
            } else {
                req.creator_scopes
            },
        };

        let record = OrganizationRecord {
            id: req.id,
            display_name: req.display_name,
            state: OrganizationLifecycleState::Provisional,
            creator_mailbox: req.creator_mailbox,
            creator_public_key: req.creator_public_key,
            administrators: vec![founding_admin],
        };

        self.organizations.push(record);
        Ok(self.organizations.last().expect("just inserted"))
    }

    /// Lookup an organization by its unique UOR ID.
    pub fn get(&self, id: &str) -> Option<&OrganizationRecord> {
        self.organizations.iter().find(|o| o.id == id)
    }

    /// Lookup an organization by its unique UOR ID (alias for get).
    pub fn get_organization(&self, id: &str) -> Option<&OrganizationRecord> {
        self.get(id)
    }

    /// Lookup an organization by unique UOR ID (mutable).
    pub fn get_mut(&mut self, id: &str) -> Option<&mut OrganizationRecord> {
        self.organizations.iter_mut().find(|o| o.id == id)
    }

    /// Activate a provisional organization according to activation policy.
    ///
    /// Requires:
    /// - Minimum 2 distinct active administrators.
    /// - Distinct mailboxes and distinct public keys (no aliases or key deduplication).
    /// - At least 2 distinct administrators per required scope ("organization", "security").
    /// - Approval by at least 2 distinct administrators.
    /// - Prohibiting single-owner bypass.
    pub fn activate_organization(
        &mut self,
        req: ActivateOrganizationRequest,
        rules: &OrganizationRules,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == req.organization_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(req.organization_id.clone()))?;

        if org.state != OrganizationLifecycleState::Provisional {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "activated".to_string(),
                reason: "only provisional organizations may be activated".to_string(),
            });
        }

        // 1. Verify minimum distinct administrator count
        if req.administrators.len() < rules.activation_minimum_distinct_administrators {
            return Err(OrganizationError::InsufficientAdministrators {
                actual: req.administrators.len(),
                required: rules.activation_minimum_distinct_administrators,
            });
        }

        // 2. Prohibit single-owner bypass and duplicate identity disguises
        let mut mailboxes = HashSet::new();
        let mut keys = HashSet::new();
        let mut user_ids = HashSet::new();

        for admin in &req.administrators {
            if !mailboxes.insert(&admin.mailbox) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate mailbox: {}",
                    admin.mailbox
                )));
            }
            if !keys.insert(&admin.public_key) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate public key detected for: {}",
                    admin.mailbox
                )));
            }
            if !user_ids.insert(&admin.user_id) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate user_id detected for: {}",
                    admin.user_id
                )));
            }
            if admin.status != "authenticated" {
                return Err(OrganizationError::SingleOwnerBypassRejected(format!(
                    "administrator {} is not authenticated",
                    admin.mailbox
                )));
            }
        }

        if mailboxes.len() < rules.activation_minimum_distinct_administrators {
            return Err(OrganizationError::SingleOwnerBypassRejected(
                "distinct user count is below minimum required administrators".to_string(),
            ));
        }

        // 3. Verify scope quorum coverage: at least 2 distinct administrators per required scope
        let required_scopes = ["organization", "security"];
        for scope in required_scopes {
            let count = req
                .administrators
                .iter()
                .filter(|a| a.scopes.iter().any(|s| s == scope))
                .count();
            if count < 2 {
                return Err(OrganizationError::InadequateScopeCoverage {
                    scope: scope.to_string(),
                    covered: count,
                    required: 2,
                });
            }
        }

        // 4. Verify approval quorum: at least 2 distinct approving administrators
        let mut approving_count = 0;
        let mut approved_users = HashSet::new();
        for mailbox in &req.approving_mailboxes {
            if let Some(admin) = req.administrators.iter().find(|a| &a.mailbox == mailbox) {
                if approved_users.insert(&admin.user_id) {
                    approving_count += 1;
                }
            }
        }
        if approving_count < rules.activation_minimum_distinct_administrators {
            return Err(OrganizationError::SingleOwnerBypassRejected(format!(
                "activation requires approval by at least {} distinct administrators, got {}",
                rules.activation_minimum_distinct_administrators, approving_count
            )));
        }

        // Update state to Activated
        org.administrators = req.administrators;
        org.state = OrganizationLifecycleState::Activated;

        Ok(org)
    }

    /// Retire a founding creator's grant, verifying that remaining distinct users
    /// collectively cover all required scopes with at least 2 distinct administrators per scope.
    pub fn retire_founding_grant(
        &mut self,
        req: RetireFoundingGrantRequest,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == req.organization_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(req.organization_id.clone()))?;

        if org.state != OrganizationLifecycleState::Activated {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "activated".to_string(),
                reason: "founding grant can only be retired on an activated organization"
                    .to_string(),
            });
        }

        // Check that founding admin exists
        let founding_index = org
            .administrators
            .iter()
            .position(|a| a.mailbox == req.founding_mailbox)
            .ok_or_else(|| {
                OrganizationError::RetirementCoverageLacking(format!(
                    "founding administrator '{}' not found in active administrators",
                    req.founding_mailbox
                ))
            })?;

        // Simulate removal and check resulting coverage
        let mut remaining = org.administrators.clone();
        remaining.remove(founding_index);

        if remaining.len() < 2 {
            return Err(OrganizationError::RetirementCoverageLacking(
                "retirement would leave fewer than 2 active administrators".to_string(),
            ));
        }

        let required_scopes = ["organization", "security"];
        for scope in required_scopes {
            let count = remaining
                .iter()
                .filter(|a| a.scopes.iter().any(|s| s == scope))
                .count();
            if count < 2 {
                return Err(OrganizationError::RetirementCoverageLacking(format!(
                    "retirement would leave scope '{scope}' with only {count} administrators (minimum 2 required)"
                )));
            }
        }

        // Apply retirement
        org.administrators = remaining;
        Ok(org)
    }

    /// Execute cross-organization isolation check.
    ///
    /// Requests are confined strictly to the calling organization's boundary.
    /// Access across organizations is rejected, even if the user has accounts in both organizations.
    pub fn verify_cross_org_access(
        &self,
        req: &CrossOrgAccessRequest,
    ) -> Result<(), OrganizationError> {
        if req.caller_org_id != req.target_org_id {
            return Err(OrganizationError::CrossOrgIsolationViolation {
                caller: req.caller_org_id.clone(),
                target: req.target_org_id.clone(),
            });
        }

        let org = self
            .get(&req.caller_org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(req.caller_org_id.clone()))?;

        if !org
            .administrators
            .iter()
            .any(|a| a.mailbox == req.caller_mailbox)
        {
            return Err(OrganizationError::UnauthorizedRoleElevation(format!(
                "user '{}' is not an authorized administrator of organization '{}'",
                req.caller_mailbox, req.caller_org_id
            )));
        }

        Ok(())
    }

    /// Update an organization's display name safely, preserving immutable org_id.
    pub fn update_organization(
        &mut self,
        req: UpdateOrganizationRequest,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        if req.new_display_name.trim().is_empty() {
            return Err(OrganizationError::InvalidDisplayName(
                "display name must not be empty".to_string(),
            ));
        }

        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == req.organization_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(req.organization_id.clone()))?;

        if org.state == OrganizationLifecycleState::Retired {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "updated".to_string(),
                reason: "cannot update display name of a retired organization".to_string(),
            });
        }

        // Caller must be an active administrator
        if !org
            .administrators
            .iter()
            .any(|a| a.mailbox == req.requesting_mailbox)
        {
            return Err(OrganizationError::UnauthorizedRoleElevation(format!(
                "user '{}' is not an authorized administrator of organization '{}'",
                req.requesting_mailbox, req.organization_id
            )));
        }

        org.display_name = req.new_display_name.trim().to_string();
        Ok(org)
    }

    /// Suspend an active organization pending security review or recovery.
    pub fn suspend_organization(
        &mut self,
        org_id: &str,
        admin_mailbox: &str,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(org_id.to_string()))?;

        if org.state != OrganizationLifecycleState::Activated {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "suspended".to_string(),
                reason: "only activated organizations may be suspended".to_string(),
            });
        }

        let admin = org
            .administrators
            .iter()
            .find(|a| a.mailbox == admin_mailbox)
            .ok_or_else(|| {
                OrganizationError::UnauthorizedRoleElevation(format!(
                    "user '{admin_mailbox}' is not an authorized administrator"
                ))
            })?;

        if !admin
            .scopes
            .iter()
            .any(|s| s == "organization" || s == "security")
        {
            return Err(OrganizationError::UnauthorizedRoleElevation(format!(
                "administrator '{admin_mailbox}' lacks organization or security scope to suspend"
            )));
        }

        org.state = OrganizationLifecycleState::Suspended;
        Ok(org)
    }

    /// Reactivate a suspended organization requiring multi-administrator quorum.
    pub fn reactivate_organization(
        &mut self,
        org_id: &str,
        approvers: &[ProposalApproval],
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(org_id.to_string()))?;

        if org.state != OrganizationLifecycleState::Suspended {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "activated".to_string(),
                reason: "only suspended organizations may be reactivated".to_string(),
            });
        }

        let mut distinct_users = HashSet::new();
        let mut distinct_keys = HashSet::new();
        let mut distinct_mailboxes = HashSet::new();

        for approval in approvers {
            let admin = org
                .administrators
                .iter()
                .find(|a| a.mailbox == approval.mailbox)
                .ok_or_else(|| {
                    OrganizationError::UnauthorizedRoleElevation(format!(
                        "approver '{}' is not an active administrator",
                        approval.mailbox
                    ))
                })?;

            if !admin
                .scopes
                .iter()
                .any(|s| s == "organization" || s == "security")
            {
                continue;
            }

            if !distinct_mailboxes.insert(&approval.mailbox) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate mailbox in reactivation approvals: {}",
                    approval.mailbox
                )));
            }
            if !distinct_keys.insert(&approval.public_key) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate public key in reactivation approvals: {}",
                    approval.public_key
                )));
            }
            distinct_users.insert(&approval.user_id);
        }

        if distinct_users.len() < 2 {
            return Err(OrganizationError::SingleOwnerBypassRejected(format!(
                "reactivation requires quorum of at least 2 distinct administrators, got {}",
                distinct_users.len()
            )));
        }

        org.state = OrganizationLifecycleState::Activated;
        Ok(org)
    }

    /// Retire an organization dynamically linking active project count with a `ProjectManager`.
    pub fn retire_organization_with_project_manager(
        &mut self,
        org_id: &str,
        approvers: &[ProposalApproval],
        force_override: bool,
        project_manager: &crate::projects::ProjectManager,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let active_project_count = project_manager.active_project_count(org_id);
        self.retire_organization(org_id, approvers, force_override, active_project_count)
    }

    /// Retire an organization with multi-administrator quorum and dependent resource safeguards.
    pub fn retire_organization(
        &mut self,
        org_id: &str,
        approvers: &[ProposalApproval],
        force_override: bool,
        active_project_count: usize,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(org_id.to_string()))?;

        if org.state == OrganizationLifecycleState::Retired {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "retired".to_string(),
                reason: "organization is already retired".to_string(),
            });
        }

        if active_project_count > 0 && !force_override {
            return Err(OrganizationError::DependentResourcesActive(format!(
                "cannot retire organization with {active_project_count} active dependent resources without force_override"
            )));
        }

        let mut distinct_users = HashSet::new();
        let mut distinct_keys = HashSet::new();
        let mut distinct_mailboxes = HashSet::new();

        for approval in approvers {
            let admin = org
                .administrators
                .iter()
                .find(|a| a.mailbox == approval.mailbox)
                .ok_or_else(|| {
                    OrganizationError::UnauthorizedRoleElevation(format!(
                        "approver '{}' is not an active administrator",
                        approval.mailbox
                    ))
                })?;

            if !admin
                .scopes
                .iter()
                .any(|s| s == "organization" || s == "security")
            {
                continue;
            }

            if !distinct_mailboxes.insert(&approval.mailbox) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate mailbox in retirement approvals: {}",
                    approval.mailbox
                )));
            }
            if !distinct_keys.insert(&approval.public_key) {
                return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                    "duplicate public key in retirement approvals: {}",
                    approval.public_key
                )));
            }
            distinct_users.insert(&approval.user_id);
        }

        if distinct_users.len() < 2 {
            return Err(OrganizationError::SingleOwnerBypassRejected(format!(
                "retirement requires quorum of at least 2 distinct administrators, got {}",
                distinct_users.len()
            )));
        }

        org.state = OrganizationLifecycleState::Retired;
        Ok(org)
    }

    /// List all organizations enrolled or created for a specific user mailbox.
    pub fn list_organizations_for_user(&self, mailbox: &str) -> Vec<&OrganizationRecord> {
        self.organizations
            .iter()
            .filter(|o| {
                o.creator_mailbox == mailbox
                    || o.administrators.iter().any(|a| a.mailbox == mailbox)
            })
            .collect()
    }

    /// Create and issue a new organization member invitation.
    pub fn create_invitation(
        &mut self,
        req: CreateInvitationRequest,
    ) -> Result<&OrgInvitationRecord, OrganizationError> {
        if !req.id.starts_with("uor:invite:") {
            return Err(OrganizationError::InvalidStateTransition {
                current: "init".to_string(),
                target: "pending".to_string(),
                reason: "invitation id must start with uor:invite:".to_string(),
            });
        }

        let org = self
            .get(&req.organization_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(req.organization_id.clone()))?;

        if org.state == OrganizationLifecycleState::Suspended
            || org.state == OrganizationLifecycleState::Retired
        {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "invitation".to_string(),
                reason: format!(
                    "cannot invite members to a {} organization",
                    org.state.as_str()
                ),
            });
        }

        // Inviter must be an authorized administrator with membership or organization scope
        let inviter = org
            .administrators
            .iter()
            .find(|a| a.mailbox == req.inviter_mailbox)
            .ok_or_else(|| {
                OrganizationError::UnauthorizedRoleElevation(format!(
                    "inviter '{}' is not an enrolled administrator",
                    req.inviter_mailbox
                ))
            })?;

        if !inviter
            .scopes
            .iter()
            .any(|s| s == "membership" || s == "organization")
        {
            return Err(OrganizationError::UnauthorizedRoleElevation(format!(
                "inviter '{}' lacks membership or organization scope",
                req.inviter_mailbox
            )));
        }

        if req.target_mailbox.trim().is_empty() || !req.target_mailbox.contains('@') {
            return Err(OrganizationError::SingleOwnerBypassRejected(
                "target mailbox must be a valid email address".to_string(),
            ));
        }

        if req.expires_at <= req.created_at {
            return Err(OrganizationError::InvalidInvitationState(
                "expires_at must be strictly after created_at".to_string(),
            ));
        }

        if self.invitations.iter().any(|i| {
            i.organization_id == req.organization_id
                && i.target_mailbox == req.target_mailbox
                && i.status == InvitationStatus::Pending
        }) {
            return Err(OrganizationError::InvalidInvitationState(format!(
                "pending invitation already exists for '{}' in organization '{}'",
                req.target_mailbox, req.organization_id
            )));
        }

        let record = OrgInvitationRecord {
            id: req.id,
            organization_id: req.organization_id,
            target_mailbox: req.target_mailbox,
            invited_scopes: req.invited_scopes,
            inviter_mailbox: req.inviter_mailbox,
            inviter_user_id: req.inviter_user_id,
            status: InvitationStatus::Pending,
            created_at: req.created_at,
            expires_at: req.expires_at,
            revision_binding: req.revision_binding,
        };

        self.invitations.push(record);
        Ok(self.invitations.last().expect("just inserted"))
    }

    /// Accept a pending invitation, enrolling the recipient as an administrator.
    pub fn accept_invitation(
        &mut self,
        invite_id: &str,
        recipient_mailbox: &str,
        recipient_user_id: &str,
        recipient_public_key: &str,
        current_timestamp: u64,
    ) -> Result<&OrganizationRecord, OrganizationError> {
        let invite_idx = self
            .invitations
            .iter()
            .position(|i| i.id == invite_id)
            .ok_or_else(|| OrganizationError::InvitationNotFound(invite_id.to_string()))?;

        let invite = &mut self.invitations[invite_idx];

        if invite.status != InvitationStatus::Pending {
            return Err(OrganizationError::InvalidInvitationState(format!(
                "invitation '{}' is not pending (status: {:?})",
                invite_id, invite.status
            )));
        }

        if current_timestamp > invite.expires_at {
            invite.status = InvitationStatus::Expired;
            return Err(OrganizationError::InvitationExpired(invite_id.to_string()));
        }

        if recipient_mailbox != invite.target_mailbox {
            return Err(OrganizationError::InvitationRecipientMismatch(format!(
                "recipient mailbox '{}' does not match invitation target '{}'",
                recipient_mailbox, invite.target_mailbox
            )));
        }

        if recipient_public_key.trim().is_empty() || recipient_user_id.trim().is_empty() {
            return Err(OrganizationError::SingleOwnerBypassRejected(
                "recipient user_id and public_key must not be empty".to_string(),
            ));
        }

        let org_id = invite.organization_id.clone();
        let scopes = invite.invited_scopes.clone();

        let org = self
            .organizations
            .iter_mut()
            .find(|o| o.id == org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(org_id.clone()))?;

        if org.state == OrganizationLifecycleState::Suspended
            || org.state == OrganizationLifecycleState::Retired
        {
            return Err(OrganizationError::InvalidStateTransition {
                current: org.state.as_str().to_string(),
                target: "enrolled".to_string(),
                reason: format!(
                    "cannot accept invitation for {} organization",
                    org.state.as_str()
                ),
            });
        }

        // Prohibit duplicate public keys disguising different mailboxes
        if org
            .administrators
            .iter()
            .any(|a| a.public_key == recipient_public_key && a.mailbox != recipient_mailbox)
        {
            return Err(OrganizationError::DuplicateIdentityDisguise(format!(
                "public key '{}' is already enrolled under a different mailbox",
                recipient_public_key
            )));
        }

        // Add or update admin
        if let Some(admin) = org
            .administrators
            .iter_mut()
            .find(|a| a.mailbox == recipient_mailbox)
        {
            for scope in scopes {
                if !admin.scopes.contains(&scope) {
                    admin.scopes.push(scope);
                }
            }
            admin.status = "authenticated".to_string();
            admin.public_key = recipient_public_key.to_string();
            admin.user_id = recipient_user_id.to_string();
        } else {
            org.administrators.push(OrgAdministrator {
                mailbox: recipient_mailbox.to_string(),
                user_id: recipient_user_id.to_string(),
                public_key: recipient_public_key.to_string(),
                status: "authenticated".to_string(),
                scopes,
            });
        }

        self.invitations[invite_idx].status = InvitationStatus::Accepted;
        Ok(org)
    }

    /// Decline a pending invitation.
    pub fn decline_invitation(
        &mut self,
        invite_id: &str,
        recipient_mailbox: &str,
    ) -> Result<(), OrganizationError> {
        let invite = self
            .invitations
            .iter_mut()
            .find(|i| i.id == invite_id)
            .ok_or_else(|| OrganizationError::InvitationNotFound(invite_id.to_string()))?;

        if invite.status != InvitationStatus::Pending {
            return Err(OrganizationError::InvalidInvitationState(format!(
                "invitation '{}' is not pending",
                invite_id
            )));
        }

        if recipient_mailbox != invite.target_mailbox {
            return Err(OrganizationError::InvitationRecipientMismatch(format!(
                "recipient mailbox '{}' does not match invitation target '{}'",
                recipient_mailbox, invite.target_mailbox
            )));
        }

        invite.status = InvitationStatus::Declined;
        Ok(())
    }

    /// Revoke a pending invitation by an authorized administrator.
    pub fn revoke_invitation(
        &mut self,
        invite_id: &str,
        admin_mailbox: &str,
    ) -> Result<(), OrganizationError> {
        let org_id = {
            let invite = self
                .invitations
                .iter()
                .find(|i| i.id == invite_id)
                .ok_or_else(|| OrganizationError::InvitationNotFound(invite_id.to_string()))?;

            if invite.status != InvitationStatus::Pending {
                return Err(OrganizationError::InvalidInvitationState(format!(
                    "invitation '{}' is not pending",
                    invite_id
                )));
            }
            invite.organization_id.clone()
        };

        let org = self
            .get(&org_id)
            .ok_or_else(|| OrganizationError::OrganizationNotFound(org_id.clone()))?;

        let admin = org
            .administrators
            .iter()
            .find(|a| a.mailbox == admin_mailbox)
            .ok_or_else(|| {
                OrganizationError::UnauthorizedRoleElevation(format!(
                    "user '{admin_mailbox}' is not an authorized administrator"
                ))
            })?;

        if !admin
            .scopes
            .iter()
            .any(|s| s == "membership" || s == "organization")
        {
            return Err(OrganizationError::UnauthorizedRoleElevation(format!(
                "administrator '{admin_mailbox}' lacks membership or organization scope to revoke invitation"
            )));
        }

        let invite = self
            .invitations
            .iter_mut()
            .find(|i| i.id == invite_id)
            .expect("checked above");

        invite.status = InvitationStatus::Revoked;
        Ok(())
    }

    /// List all invitations for a given organization.
    pub fn list_invitations_for_org(&self, org_id: &str) -> Vec<&OrgInvitationRecord> {
        self.invitations
            .iter()
            .filter(|i| i.organization_id == org_id)
            .collect()
    }

    /// List all invitations for a given recipient mailbox.
    pub fn list_invitations_for_user(&self, mailbox: &str) -> Vec<&OrgInvitationRecord> {
        self.invitations
            .iter()
            .filter(|i| i.target_mailbox == mailbox)
            .collect()
    }

    /// Lookup an invitation by its unique identifier.
    pub fn get_invitation(&self, invite_id: &str) -> Option<&OrgInvitationRecord> {
        self.invitations.iter().find(|i| i.id == invite_id)
    }
}

impl OrganizationLifecycleConfig {
    /// Cross-check organization lifecycle rules against owner inputs.
    pub fn check<'a>(
        &self,
        owner_inputs: impl Into<Option<&'a OwnerInputs>>,
    ) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);

        if self.spec != "foundry/organization-lifecycle/1" {
            return Err(bad(format!("unexpected lifecycle spec: {}", self.spec)));
        }

        if !self.rules.open_enrollment {
            return Err(bad("open_enrollment must be true".to_string()));
        }
        if !self.rules.open_organization_creation {
            return Err(bad("open_organization_creation must be true".to_string()));
        }
        if !self.rules.names_confer_no_privileges {
            return Err(bad("names_confer_no_privileges must be true".to_string()));
        }
        if !self.rules.allow_sole_founding_admin_in_provisional {
            return Err(bad(
                "allow_sole_founding_admin_in_provisional must be true".to_string()
            ));
        }
        if self.rules.activation_minimum_distinct_administrators < 2 {
            return Err(bad(
                "activation_minimum_distinct_administrators must be at least 2".to_string(),
            ));
        }
        if !self.rules.require_distinct_users_per_scope {
            return Err(bad(
                "require_distinct_users_per_scope must be true".to_string()
            ));
        }
        if !self.rules.prohibit_single_owner_bypass {
            return Err(bad("prohibit_single_owner_bypass must be true".to_string()));
        }
        if !self.rules.retirement_requires_full_coverage {
            return Err(bad(
                "retirement_requires_full_coverage must be true".to_string()
            ));
        }
        if !self.rules.enforce_cross_org_isolation {
            return Err(bad("enforce_cross_org_isolation must be true".to_string()));
        }

        // Cross-check an organization's policy only when that organization exists.
        if let Some(owner_inputs) = owner_inputs.into() {
            if self.rules.activation_minimum_distinct_administrators
                != owner_inputs.activation_policy.minimum_active_administrators
            {
                return Err(bad(format!(
                "activation minimum distinct administrators mismatch: config has {}, owner_inputs has {}",
                self.rules.activation_minimum_distinct_administrators,
                owner_inputs.activation_policy.minimum_active_administrators
            )));
            }
        }
        // Verify transition rules cover provisional -> activated
        let has_activation_rule = self.allowed_transitions.iter().any(|r| {
            r.from == "provisional" && r.to == "activated" && r.minimum_distinct_administrators >= 2
        });
        if !has_activation_rule {
            return Err(bad(
                "allowed_transitions must define provisional -> activated with at least 2 administrators".to_string(),
            ));
        }

        Ok(())
    }
}
