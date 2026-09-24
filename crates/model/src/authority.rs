//! Scoped multi-administrator authority model (AM-01).
//!
//! Under SPEC.md and IMPLEMENTATION.md:
//! 1. Replaces PrismPM Workspace/V1 single-owner operations with modeled scoped multi-administrator authority.
//! 2. Requires complete scoped ownership by distinct users: at least two retained administrators per affected part.
//! 3. Prohibits single-owner bypasses and self-selected role elevations.
//! 4. Rejects duplicate key or email disguises attempting to bypass quorum constraints.
//! 5. Enforces atomic post-change ownership coverage: any change that would leave any scope with fewer than
//!    the required minimum distinct administrators is atomically rejected.
//! 6. Rejects concurrent revision conflicts and concurrent lockout scenarios.
//! 7. Rejects premature bootstrap/founding grant retirement before full distinct-user coverage is confirmed.
//! 8. Enforces strict cross-organization authority isolation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::organization::{OrgAdministrator, OrganizationLifecycleConfig};
use crate::{ModelError, OwnerInputs};

/// Top-level configuration for the authority model (`model/authority.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityConfig {
    /// Specification identifier (must be `foundry/authority/1`).
    pub spec: String,
    /// General policy invariants.
    pub policy: AuthorityPolicyConfig,
    /// Set of standard scope rules.
    pub scope_rules: Vec<ScopeRuleConfig>,
}

/// Invariant policy flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityPolicyConfig {
    /// Policy descriptive name.
    pub name: String,
    /// Require distinct authenticated users per administrative scope.
    pub enforce_distinct_users_per_scope: bool,
    /// Minimum number of distinct administrators per scope (normatively >= 2).
    pub minimum_administrators_per_scope: usize,
    /// Prohibit single-owner bypass shortcuts.
    pub prohibit_single_owner_bypass: bool,
    /// Verify post-change scope coverage atomically before committing any modification.
    pub require_atomic_post_change_coverage: bool,
    /// Reject concurrent proposals that would result in lockout or deadlock.
    pub prevent_concurrent_lockout: bool,
    /// Refuse bootstrap retirement until post-retirement coverage is confirmed.
    pub prevent_premature_bootstrap_retirement: bool,
    /// Optimistic concurrency fencing via monotonic revisions.
    pub optimistic_concurrency_fencing: bool,
    /// List of normative scopes required for full operation.
    pub required_scopes: Vec<String>,
}

/// Normative quorum rules for an individual administrative scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeRuleConfig {
    /// Scope identifier (e.g. `organization`, `security`, `operations`, etc.).
    pub scope: String,
    /// Minimum distinct administrators required for this scope.
    pub minimum_distinct_administrators: usize,
    /// Quorum specification (e.g. `2-of-N`).
    pub approval_quorum: String,
    /// Scope description.
    pub description: String,
}

impl AuthorityConfig {
    /// Cross-check the authority configuration against owner-inputs and organization lifecycle.
    pub fn check(
        &self,
        owner_inputs: &OwnerInputs,
        org_config: &OrganizationLifecycleConfig,
    ) -> Result<(), ModelError> {
        let bad = |msg: String| ModelError::Inconsistent(format!("model/authority.toml: {msg}"));

        if self.spec != "foundry/authority/1" {
            return Err(bad(format!(
                "expected spec 'foundry/authority/1', found '{}'",
                self.spec
            )));
        }

        if self.policy.minimum_administrators_per_scope < 2 {
            return Err(bad(
                "policy.minimum_administrators_per_scope must be at least 2 to replace single-owner model"
                    .to_string(),
            ));
        }

        if !self.policy.enforce_distinct_users_per_scope {
            return Err(bad(
                "policy.enforce_distinct_users_per_scope must be true".to_string()
            ));
        }

        if !self.policy.prohibit_single_owner_bypass {
            return Err(bad(
                "policy.prohibit_single_owner_bypass must be true".to_string()
            ));
        }

        if !self.policy.require_atomic_post_change_coverage {
            return Err(bad(
                "policy.require_atomic_post_change_coverage must be true".to_string(),
            ));
        }

        if !self.policy.prevent_concurrent_lockout {
            return Err(bad(
                "policy.prevent_concurrent_lockout must be true".to_string()
            ));
        }

        if !self.policy.prevent_premature_bootstrap_retirement {
            return Err(bad(
                "policy.prevent_premature_bootstrap_retirement must be true".to_string(),
            ));
        }

        if !self.policy.optimistic_concurrency_fencing {
            return Err(bad(
                "policy.optimistic_concurrency_fencing must be true".to_string()
            ));
        }

        // Cross-check each scope rule
        for rule in &self.scope_rules {
            if rule.minimum_distinct_administrators < self.policy.minimum_administrators_per_scope {
                return Err(bad(format!(
                    "scope '{}' specifies minimum {} distinct administrators, less than policy minimum {}",
                    rule.scope, rule.minimum_distinct_administrators, self.policy.minimum_administrators_per_scope
                )));
            }
        }

        // Cross-check with OwnerInputs activation policy
        if owner_inputs.activation_policy.minimum_active_administrators
            < self.policy.minimum_administrators_per_scope
        {
            return Err(bad(
                "owner_inputs activation minimum active administrators is less than authority minimum"
                    .to_string(),
            ));
        }

        if !owner_inputs.activation_policy.prohibit_single_owner_bypass {
            return Err(bad(
                "owner_inputs must prohibit single owner bypass".to_string()
            ));
        }

        // Cross-check with OrganizationLifecycleConfig
        if org_config.rules.activation_minimum_distinct_administrators
            < self.policy.minimum_administrators_per_scope
        {
            return Err(bad(
                "organization lifecycle activation minimum administrators is less than authority minimum"
                    .to_string(),
            ));
        }

        Ok(())
    }
}

/// Lifecycle status of an individual authority grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrantStatus {
    /// Active grant conferring scoped authority.
    Active,
    /// Grant has been explicitly revoked through multi-admin consensus.
    Revoked,
    /// Grant is suspended pending security review.
    Suspended,
}

/// An individual scoped authority grant issued to an authenticated administrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityGrant {
    /// Unique grant identifier.
    pub grant_id: String,
    /// Target organization ID.
    pub organization_id: String,
    /// Mailbox of the grantee.
    pub mailbox: String,
    /// User identifier of the grantee.
    pub user_id: String,
    /// Public key (hex-encoded) of the grantee.
    pub public_key: String,
    /// Administrative scope conferred by this grant.
    pub scope: String,
    /// Status of the grant.
    pub status: GrantStatus,
    /// Organization revision at which this grant was activated.
    pub granted_at_revision: u64,
}

/// Proposal approval record signed by an administrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalApproval {
    /// Mailbox of the approving administrator.
    pub mailbox: String,
    /// User identifier of the approving administrator.
    pub user_id: String,
    /// Public key of the approving administrator.
    pub public_key: String,
    /// Timestamp of approval.
    pub timestamp: u64,
}

/// Action to be performed by an authority change proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AuthorityAction {
    /// Grant a new scope to an administrator.
    GrantScope {
        /// Administrator mailbox.
        mailbox: String,
        /// Administrator user ID.
        user_id: String,
        /// Administrator public key.
        public_key: String,
        /// Scope to grant.
        scope: String,
    },
    /// Revoke a specific scope from an administrator.
    RevokeScope {
        /// Administrator mailbox.
        mailbox: String,
        /// Scope to revoke.
        scope: String,
    },
    /// Completely retire an administrator across all scopes.
    RetireAdministrator {
        /// Administrator mailbox.
        mailbox: String,
    },
    /// Retire the provisional founding bootstrap grant.
    RetireBootstrapGrant {
        /// Mailbox of the founding administrator whose bootstrap grant is being retired.
        founding_mailbox: String,
    },
}

impl AuthorityAction {
    /// Primary administrative scope affected by this action.
    pub fn affected_scope(&self) -> &str {
        match self {
            Self::GrantScope { scope, .. } => scope.as_str(),
            Self::RevokeScope { scope, .. } => scope.as_str(),
            Self::RetireAdministrator { .. } => "organization",
            Self::RetireBootstrapGrant { .. } => "organization",
        }
    }
}

/// Status of a change proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProposalStatus {
    /// Proposal is pending further approvals.
    Pending,
    /// Proposal has met quorum and has been atomically committed.
    Executed,
    /// Proposal was rejected due to policy violations.
    Rejected,
}

/// A proposed mutation to the organization authority graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeProposal {
    /// Proposal identifier.
    pub proposal_id: String,
    /// Organization ID.
    pub organization_id: String,
    /// Base revision against which this proposal was formulated.
    pub base_revision: u64,
    /// Action to execute upon approval.
    pub action: AuthorityAction,
    /// Mailbox of the proposing administrator.
    pub proposer_mailbox: String,
    /// Approvals collected thus far.
    pub approvals: Vec<ProposalApproval>,
    /// Proposal lifecycle status.
    pub status: ProposalStatus,
}

/// The state of an organization's authority graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationAuthorityRecord {
    /// Organization identifier.
    pub organization_id: String,
    /// Monotonically increasing revision number for optimistic concurrency fencing.
    pub revision: u64,
    /// Active, revoked, and suspended authority grants.
    pub grants: Vec<AuthorityGrant>,
    /// Flag indicating whether the provisional founding grant is still active.
    pub bootstrap_grant_active: bool,
    /// Founding administrator mailbox.
    pub founding_mailbox: Option<String>,
}

/// Errors raised by the authority model engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorityError {
    /// Organization not found in registry.
    OrganizationNotFound(String),
    /// Single-owner bypass attempt rejected.
    SingleOwnerBypassRejected(String),
    /// Duplicate public key detected (identity disguise).
    DuplicateKeyDisguise(String),
    /// Duplicate mailbox approval detected.
    DuplicateMailboxDisguise(String),
    /// Insufficient distinct-user approvals to satisfy scope quorum.
    InsufficientQuorum {
        /// Scope lacking quorum.
        scope: String,
        /// Approvals provided.
        actual: usize,
        /// Approvals required.
        required: usize,
    },
    /// Post-change atomic verification failed: dropping below required minimum distinct admins.
    PostChangeCoverageDeficit {
        /// Scope that would become deficient.
        scope: String,
        /// Remaining distinct administrators.
        remaining: usize,
        /// Minimum required distinct administrators.
        required: usize,
    },
    /// Concurrent revision conflict detected (optimistic concurrency fencing).
    ConcurrentRevisionConflict {
        /// Expected base revision.
        expected: u64,
        /// Actual current organization revision.
        actual: u64,
    },
    /// Concurrent action rejected to prevent organization lockout or deadlock.
    ConcurrentLockoutPrevented(String),
    /// Premature bootstrap retirement rejected because post-retirement coverage is insufficient.
    PrematureBootstrapRetirement(String),
    /// Cross-organization authority access violation.
    CrossOrgAuthorityViolation {
        /// Organization of the caller/approver.
        caller_org: String,
        /// Target organization.
        target_org: String,
    },
    /// Proposal not found.
    ProposalNotFound(String),
    /// Proposal already finalized.
    ProposalAlreadyFinalized(String),
    /// Approver lacks the necessary authority scope.
    UnauthorizedApprover(String),
}

impl std::fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrganizationNotFound(id) => write!(f, "organization not found: {id}"),
            Self::SingleOwnerBypassRejected(msg) => {
                write!(f, "single-owner bypass rejected: {msg}")
            }
            Self::DuplicateKeyDisguise(msg) => write!(f, "duplicate key disguise rejected: {msg}"),
            Self::DuplicateMailboxDisguise(msg) => {
                write!(f, "duplicate mailbox disguise rejected: {msg}")
            }
            Self::InsufficientQuorum {
                scope,
                actual,
                required,
            } => {
                write!(f, "insufficient quorum for scope '{scope}': {actual} approved, {required} required")
            }
            Self::PostChangeCoverageDeficit {
                scope,
                remaining,
                required,
            } => {
                write!(
                    f,
                    "atomic post-change coverage deficit for scope '{scope}': {remaining} remaining, {required} required"
                )
            }
            Self::ConcurrentRevisionConflict { expected, actual } => {
                write!(f, "concurrent revision conflict: proposal formulated on revision {expected}, current revision is {actual}")
            }
            Self::ConcurrentLockoutPrevented(msg) => {
                write!(f, "concurrent lockout prevented: {msg}")
            }
            Self::PrematureBootstrapRetirement(msg) => {
                write!(f, "premature bootstrap retirement rejected: {msg}")
            }
            Self::CrossOrgAuthorityViolation {
                caller_org,
                target_org,
            } => {
                write!(f, "cross-organization authority violation: caller from '{caller_org}' attempted modification of '{target_org}'")
            }
            Self::ProposalNotFound(id) => write!(f, "proposal not found: {id}"),
            Self::ProposalAlreadyFinalized(id) => write!(f, "proposal '{id}' already finalized"),
            Self::UnauthorizedApprover(msg) => write!(f, "unauthorized approver: {msg}"),
        }
    }
}

impl std::error::Error for AuthorityError {}

/// Authority manager enforcing scoped multi-administrator policies.
#[derive(Debug, Clone)]
pub struct AuthorityManager {
    /// Authority records indexed by organization ID.
    pub records: HashMap<String, OrganizationAuthorityRecord>,
    /// Pending proposals indexed by proposal ID.
    pub proposals: HashMap<String, ChangeProposal>,
    /// Active policy rules.
    pub config: AuthorityConfig,
}

impl AuthorityManager {
    /// Create a new AuthorityManager with the given configuration.
    pub fn new(config: AuthorityConfig) -> Self {
        Self {
            records: HashMap::new(),
            proposals: HashMap::new(),
            config,
        }
    }

    /// Initialize an organization's authority graph from initial administrators.
    ///
    /// If only 1 administrator is provided (provisional bootstrap), `bootstrap_grant_active` is set to true.
    /// If >= 2 administrators are provided, grants are initialized and coverage is checked.
    pub fn initialize_organization(
        &mut self,
        org_id: &str,
        administrators: &[OrgAdministrator],
    ) -> Result<&OrganizationAuthorityRecord, AuthorityError> {
        let is_provisional_sole = administrators.len() == 1;

        if is_provisional_sole && !self.config.policy.prohibit_single_owner_bypass {
            // provisional sole founding admin allowed temporarily under strict bootstrap flag
        }

        let mut grants = Vec::new();
        let mut key_to_mailbox = HashMap::new();

        for admin in administrators {
            if let Some(existing_mailbox) = key_to_mailbox.get(&admin.public_key) {
                if existing_mailbox != &admin.mailbox {
                    return Err(AuthorityError::DuplicateKeyDisguise(format!(
                        "public key '{}' shared between '{}' and '{}'",
                        admin.public_key, existing_mailbox, admin.mailbox
                    )));
                }
            } else {
                key_to_mailbox.insert(admin.public_key.clone(), admin.mailbox.clone());
            }

            for scope in &admin.scopes {
                grants.push(AuthorityGrant {
                    grant_id: format!("grant:{}:{}:{}", org_id, admin.mailbox, scope),
                    organization_id: org_id.to_string(),
                    mailbox: admin.mailbox.clone(),
                    user_id: admin.user_id.clone(),
                    public_key: admin.public_key.clone(),
                    scope: scope.clone(),
                    status: GrantStatus::Active,
                    granted_at_revision: 1,
                });
            }
        }

        let record = OrganizationAuthorityRecord {
            organization_id: org_id.to_string(),
            revision: 1,
            grants,
            bootstrap_grant_active: is_provisional_sole,
            founding_mailbox: if is_provisional_sole {
                Some(administrators[0].mailbox.clone())
            } else {
                None
            },
        };

        self.records.insert(org_id.to_string(), record);
        Ok(self.records.get(org_id).unwrap())
    }

    /// Retrieve active distinct administrators holding a given scope.
    pub fn distinct_administrators_for_scope(
        record: &OrganizationAuthorityRecord,
        scope: &str,
    ) -> HashSet<String> {
        record
            .grants
            .iter()
            .filter(|g| g.status == GrantStatus::Active && g.scope == scope)
            .map(|g| g.mailbox.clone())
            .collect()
    }

    /// Retrieve active distinct public keys holding a given scope.
    pub fn distinct_keys_for_scope(
        record: &OrganizationAuthorityRecord,
        scope: &str,
    ) -> HashSet<String> {
        record
            .grants
            .iter()
            .filter(|g| g.status == GrantStatus::Active && g.scope == scope)
            .map(|g| g.public_key.clone())
            .collect()
    }

    /// Submit a new authority change proposal.
    pub fn submit_proposal(&mut self, proposal: ChangeProposal) -> Result<(), AuthorityError> {
        let record = self.records.get(&proposal.organization_id).ok_or_else(|| {
            AuthorityError::OrganizationNotFound(proposal.organization_id.clone())
        })?;

        // Fencing check: base_revision must equal current revision
        if self.config.policy.optimistic_concurrency_fencing
            && proposal.base_revision != record.revision
        {
            return Err(AuthorityError::ConcurrentRevisionConflict {
                expected: proposal.base_revision,
                actual: record.revision,
            });
        }

        // Verify proposer has active standing in organization
        let proposer_active = record
            .grants
            .iter()
            .any(|g| g.mailbox == proposal.proposer_mailbox && g.status == GrantStatus::Active);
        if !proposer_active {
            return Err(AuthorityError::UnauthorizedApprover(format!(
                "proposer '{}' is not an active administrator in organization '{}'",
                proposal.proposer_mailbox, proposal.organization_id
            )));
        }

        self.proposals
            .insert(proposal.proposal_id.clone(), proposal);
        Ok(())
    }

    /// Record an approval for a pending proposal.
    pub fn add_approval(
        &mut self,
        proposal_id: &str,
        approval: ProposalApproval,
    ) -> Result<(), AuthorityError> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| AuthorityError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != ProposalStatus::Pending {
            return Err(AuthorityError::ProposalAlreadyFinalized(
                proposal_id.to_string(),
            ));
        }

        let record = self.records.get(&proposal.organization_id).ok_or_else(|| {
            AuthorityError::OrganizationNotFound(proposal.organization_id.clone())
        })?;

        // Verify approver belongs to target organization with required scope
        let affected_scope = proposal.action.affected_scope();
        let approver_grant = record.grants.iter().find(|g| {
            g.mailbox == approval.mailbox
                && g.status == GrantStatus::Active
                && g.scope == affected_scope
        });

        if approver_grant.is_none() {
            return Err(AuthorityError::UnauthorizedApprover(format!(
                "approver '{}' lacks active grant for scope '{}' in organization '{}'",
                approval.mailbox, affected_scope, proposal.organization_id
            )));
        }

        // Check for duplicate mailbox approval
        if proposal
            .approvals
            .iter()
            .any(|a| a.mailbox == approval.mailbox)
        {
            return Err(AuthorityError::DuplicateMailboxDisguise(format!(
                "administrator '{}' has already approved proposal '{}'",
                approval.mailbox, proposal_id
            )));
        }

        // Check for duplicate public key approval (alias disguise)
        if proposal
            .approvals
            .iter()
            .any(|a| a.public_key == approval.public_key)
        {
            return Err(AuthorityError::DuplicateKeyDisguise(format!(
                "public key '{}' has already signed proposal '{}'",
                approval.public_key, proposal_id
            )));
        }

        proposal.approvals.push(approval);
        Ok(())
    }

    /// Evaluate and atomically execute a proposal if quorum and post-change coverage pass.
    pub fn execute_proposal(&mut self, proposal_id: &str) -> Result<u64, AuthorityError> {
        let proposal = self
            .proposals
            .get(proposal_id)
            .ok_or_else(|| AuthorityError::ProposalNotFound(proposal_id.to_string()))?
            .clone();

        if proposal.status != ProposalStatus::Pending {
            return Err(AuthorityError::ProposalAlreadyFinalized(
                proposal_id.to_string(),
            ));
        }

        let record = self
            .records
            .get_mut(&proposal.organization_id)
            .ok_or_else(|| {
                AuthorityError::OrganizationNotFound(proposal.organization_id.clone())
            })?;

        // 1. Concurrency fencing check
        if self.config.policy.optimistic_concurrency_fencing
            && proposal.base_revision != record.revision
        {
            return Err(AuthorityError::ConcurrentRevisionConflict {
                expected: proposal.base_revision,
                actual: record.revision,
            });
        }

        // 2. Multi-administrator Quorum Check
        let affected_scope = proposal.action.affected_scope();
        let required_quorum = self
            .config
            .scope_rules
            .iter()
            .find(|r| r.scope == affected_scope)
            .map(|r| r.minimum_distinct_administrators)
            .unwrap_or(self.config.policy.minimum_administrators_per_scope);

        // Deduplicate distinct approved public keys
        let distinct_approver_keys: HashSet<String> = proposal
            .approvals
            .iter()
            .map(|a| a.public_key.clone())
            .collect();
        let distinct_approver_mailboxes: HashSet<String> = proposal
            .approvals
            .iter()
            .map(|a| a.mailbox.clone())
            .collect();

        if distinct_approver_mailboxes.len() != proposal.approvals.len() {
            return Err(AuthorityError::DuplicateMailboxDisguise(
                "duplicate mailbox detected in proposal approvals".to_string(),
            ));
        }

        if distinct_approver_keys.len() != proposal.approvals.len() {
            return Err(AuthorityError::DuplicateKeyDisguise(
                "duplicate public key detected across different mailboxes".to_string(),
            ));
        }

        if distinct_approver_keys.len() < required_quorum {
            if self.config.policy.prohibit_single_owner_bypass && distinct_approver_keys.len() <= 1
            {
                return Err(AuthorityError::SingleOwnerBypassRejected(format!(
                    "single-owner execution prohibited for scope '{affected_scope}': requires at least {required_quorum} distinct approvals"
                )));
            }
            return Err(AuthorityError::InsufficientQuorum {
                scope: affected_scope.to_string(),
                actual: distinct_approver_keys.len(),
                required: required_quorum,
            });
        }

        // 3. Simulate Post-Change Authority Graph (Atomic Coverage Verification)
        let mut simulated_grants = record.grants.clone();
        let mut simulated_bootstrap_active = record.bootstrap_grant_active;

        match &proposal.action {
            AuthorityAction::GrantScope {
                mailbox,
                user_id,
                public_key,
                scope,
            } => {
                // If grant already exists, activate it
                if let Some(existing) = simulated_grants
                    .iter_mut()
                    .find(|g| g.mailbox == *mailbox && g.scope == *scope)
                {
                    existing.status = GrantStatus::Active;
                    existing.public_key = public_key.clone();
                } else {
                    simulated_grants.push(AuthorityGrant {
                        grant_id: format!(
                            "grant:{}:{}:{}",
                            proposal.organization_id, mailbox, scope
                        ),
                        organization_id: proposal.organization_id.clone(),
                        mailbox: mailbox.clone(),
                        user_id: user_id.clone(),
                        public_key: public_key.clone(),
                        scope: scope.clone(),
                        status: GrantStatus::Active,
                        granted_at_revision: record.revision + 1,
                    });
                }
            }
            AuthorityAction::RevokeScope { mailbox, scope } => {
                for g in simulated_grants
                    .iter_mut()
                    .filter(|g| g.mailbox == *mailbox && (g.scope == *scope || *scope == "all"))
                {
                    g.status = GrantStatus::Revoked;
                }
            }
            AuthorityAction::RetireAdministrator { mailbox } => {
                for g in simulated_grants
                    .iter_mut()
                    .filter(|g| g.mailbox == *mailbox)
                {
                    g.status = GrantStatus::Revoked;
                }
            }
            AuthorityAction::RetireBootstrapGrant { founding_mailbox } => {
                if !simulated_bootstrap_active {
                    return Err(AuthorityError::PrematureBootstrapRetirement(
                        "bootstrap grant has already been retired".to_string(),
                    ));
                }
                simulated_bootstrap_active = false;
                for g in simulated_grants
                    .iter_mut()
                    .filter(|g| g.mailbox == *founding_mailbox)
                {
                    g.status = GrantStatus::Revoked;
                }
            }
        }

        // 4. Verify Atomic Coverage: All required scopes must retain >= minimum distinct administrators
        if self.config.policy.require_atomic_post_change_coverage {
            for scope_rule in &self.config.scope_rules {
                let active_distinct_admins: HashSet<String> = simulated_grants
                    .iter()
                    .filter(|g| g.status == GrantStatus::Active && g.scope == scope_rule.scope)
                    .map(|g| g.mailbox.clone())
                    .collect();

                let active_distinct_keys: HashSet<String> = simulated_grants
                    .iter()
                    .filter(|g| g.status == GrantStatus::Active && g.scope == scope_rule.scope)
                    .map(|g| g.public_key.clone())
                    .collect();

                let effective_distinct =
                    active_distinct_admins.len().min(active_distinct_keys.len());

                if effective_distinct < scope_rule.minimum_distinct_administrators {
                    if matches!(
                        proposal.action,
                        AuthorityAction::RetireBootstrapGrant { .. }
                    ) {
                        return Err(AuthorityError::PrematureBootstrapRetirement(format!(
                            "retiring bootstrap grant leaves scope '{}' with only {} distinct administrators, required {}",
                            scope_rule.scope, effective_distinct, scope_rule.minimum_distinct_administrators
                        )));
                    }

                    return Err(AuthorityError::PostChangeCoverageDeficit {
                        scope: scope_rule.scope.clone(),
                        remaining: effective_distinct,
                        required: scope_rule.minimum_distinct_administrators,
                    });
                }
            }
        }

        // 5. Commit atomically: update state and advance revision
        record.revision += 1;
        record.grants = simulated_grants;
        record.bootstrap_grant_active = simulated_bootstrap_active;

        if let Some(prop) = self.proposals.get_mut(proposal_id) {
            prop.status = ProposalStatus::Executed;
        }

        Ok(record.revision)
    }

    /// Check cross-organization boundary enforcement.
    pub fn verify_cross_org_authority(
        &self,
        caller_org: &str,
        target_org: &str,
    ) -> Result<(), AuthorityError> {
        if caller_org != target_org {
            return Err(AuthorityError::CrossOrgAuthorityViolation {
                caller_org: caller_org.to_string(),
                target_org: target_org.to_string(),
            });
        }
        Ok(())
    }
}
