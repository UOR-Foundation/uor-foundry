//! Typed representation and validation of owner-controlled acceptance inputs.
//!
//! Owner-controlled inputs are per-organization records required by
//! `SPEC.md` and `IMPLEMENTATION.md`. They are approved and bound by the
//! organization's governing authorities rather than invented by platform
//! defaults.

use serde::Deserialize;

use crate::ModelError;

/// Root structure of `model/owner_inputs.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct OwnerInputs {
    /// Schema version tag.
    pub spec: String,
    /// Organization identity record.
    pub organization: OrganizationIdentity,
    /// Legal entity record.
    pub legal_entity: LegalEntity,
    /// Modeled physical and community sites.
    pub sites: Vec<SiteRecord>,
    /// Site assessment records.
    pub site_assessments: Vec<SiteAssessmentRecord>,
    /// Authenticated administrator identities and keys.
    pub administrators: Vec<AdministratorRecord>,
    /// Membership and role admission policies.
    pub membership_policy: MembershipPolicy,
    /// Activation policy and quorum requirements.
    pub activation_policy: ActivationPolicy,
    /// Account and credential recovery rules.
    pub recovery_rules: RecoveryRules,
    /// Adopted standards and editions.
    pub standards: Vec<AdoptedStandard>,
    /// Authorized assessment bodies.
    pub assessment_authorities: Vec<AssessmentAuthority>,
    /// Business plan and operational procedures.
    pub business_operations: BusinessOperations,
    /// Brand identity and presentation standards.
    pub brand_identity: BrandIdentity,
    /// Authorized publication decisions and targets.
    pub publication_approvals: PublicationApprovals,
    /// Payment scope rules and certification authority governance.
    pub payment_certification_rules: PaymentCertificationRules,
    /// Resilience, workload, fault bounds, RPO/RTO, and replica obligations.
    pub resilience_bounds: ResilienceBounds,
}

/// Organizational identity.
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationIdentity {
    /// UOR identifier for the organization (e.g. `uor:org:uor-foundation`).
    pub id: String,
    /// Display name of the organization.
    pub display_name: String,
    /// Core mission statement.
    pub mission: String,
    /// Operating model (e.g. `Citizen Gardens`).
    pub operating_model: String,
}

/// Legal entity details and governance jurisdiction.
#[derive(Debug, Clone, Deserialize)]
pub struct LegalEntity {
    /// Registered legal name.
    pub entity_name: String,
    /// Legal entity structure (e.g. Non-Profit Organization).
    pub entity_type: String,
    /// Statutory registration identifier.
    pub registration_identifier: String,
    /// Primary legal jurisdiction.
    pub governing_jurisdiction: String,
    /// Operating jurisdictions.
    pub operating_jurisdictions: Vec<String>,
    /// Status of statutory filings and registrations.
    pub statutory_filings_status: String,
    /// SHA-256 digest of foundational legal charter.
    pub charter_digest: String,
}

/// Modeled physical site or facility.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteRecord {
    /// UOR site identifier.
    pub id: String,
    /// Site name.
    pub name: String,
    /// Facility classification.
    pub site_type: String,
    /// Physical location or address.
    pub address: String,
    /// Local jurisdiction.
    pub jurisdiction: String,
    /// Operational status.
    pub status: String,
}

/// Assessment record for a physical or operational site.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteAssessmentRecord {
    /// Assessment record identifier.
    pub id: String,
    /// Target site identifier.
    pub site_id: String,
    /// Assessment domain (e.g. physical security, accessibility).
    pub assessment_type: String,
    /// Assessing body or authority.
    pub assessor: String,
    /// Applied normative standard.
    pub standard_applied: String,
    /// Outcome of the assessment (e.g. conforming).
    pub result: String,
    /// SHA-256 digest of signed assessment evidence.
    pub evidence_digest: String,
    /// Validity expiration date (ISO 8601).
    pub valid_until: String,
}

/// Authenticated administrator identity, mailbox, and public key.
#[derive(Debug, Clone, Deserialize)]
pub struct AdministratorRecord {
    /// Verified administrator email address.
    pub mailbox: String,
    /// UOR user identifier.
    pub user_id: String,
    /// Cryptographic key algorithm (e.g. ed25519).
    pub key_algorithm: String,
    /// Encoded public key material.
    pub public_key: String,
    /// Public key fingerprint.
    pub key_fingerprint: String,
    /// Authentication and verification status.
    pub status: String,
    /// Authorized administrative scopes.
    pub scopes: Vec<String>,
}

/// Membership and role admission policies.
#[derive(Debug, Clone, Deserialize)]
pub struct MembershipPolicy {
    /// Overall admission policy mode.
    pub admission_policy: String,
    /// Authority required to approve faculty appointment.
    pub faculty_admission_approver: String,
    /// Enrollment model for public participants.
    pub participant_enrollment: String,
    /// Quorum required for role elevation.
    pub role_elevation_quorum: String,
    /// Prohibition against self-selected or self-appointed roles.
    pub prohibit_self_appointment: bool,
}

/// Activation policy and administrative redundancy rules.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivationPolicy {
    /// Minimum active administrators required for activation.
    pub minimum_active_administrators: usize,
    /// Requirement that administrators be distinct verified users.
    pub require_distinct_users: bool,
    /// Requirement that every modeled scope maintain quorum coverage.
    pub require_quorum_coverage_per_scope: bool,
    /// Explicit prohibition against single-owner bypass.
    pub prohibit_single_owner_bypass: bool,
    /// Requirement that provisional bootstrap grants are retired only after full coverage.
    pub provisional_bootstrap_retirement_requires_full_coverage: bool,
}

/// Credential and account recovery rules.
#[derive(Debug, Clone, Deserialize)]
pub struct RecoveryRules {
    /// Time-to-live for email verification challenges in seconds.
    pub email_challenge_ttl_seconds: u64,
    /// Enforcement of single-use challenge tokens to prevent replay.
    pub email_challenge_replay_protection: bool,
    /// Recovery code standard (e.g. NIST SP 800-63B-4).
    pub backup_code_standard: String,
    /// Entropy bits per generated backup code.
    pub backup_code_entropy_bits: u32,
    /// Hash storage scheme for backup codes (never plaintext).
    pub backup_code_storage_scheme: String,
    /// Single-use redemption constraint for backup codes.
    pub backup_code_single_use: bool,
    /// Mandatory invalidation of active sessions upon credential recovery.
    pub session_invalidation_on_recovery: bool,
    /// Rejection of recovery attempts targeting revoked grants.
    pub reject_revoked_grant_recovery: bool,
}

/// Adopted normative standard and rights record.
#[derive(Debug, Clone, Deserialize)]
pub struct AdoptedStandard {
    /// Short identifier for the standard.
    pub standard_id: String,
    /// Canonical specification identifier.
    pub canonical_id: String,
    /// Adopted edition or version.
    pub edition: String,
    /// Standards development organization or issuer.
    pub issuer: String,
    /// Role of the source (e.g. normative, binding-only).
    pub source_role: String,
    /// License or acquisition terms.
    pub license: String,
    /// Redistribution permissions (e.g. citation-only, redistributable).
    pub redistribution: String,
    /// URL of the canonical specification.
    pub source_url: String,
}

/// Authorized assessment authority.
#[derive(Debug, Clone, Deserialize)]
pub struct AssessmentAuthority {
    /// Authority identifier.
    pub id: String,
    /// Human-readable authority name.
    pub name: String,
    /// Category of authority.
    pub authority_type: String,
    /// Verification key or certificate fingerprint.
    pub verification_key: String,
    /// Formal citation of authority or accreditation.
    pub citation: String,
}

/// Operational and business plan governance.
#[derive(Debug, Clone, Deserialize)]
pub struct BusinessOperations {
    /// Modeled governance structure.
    pub governance_model: String,
    /// SHA-256 digest of approved business plan.
    pub business_plan_digest: String,
    /// SHA-256 digest of operating procedures.
    pub operating_procedures_digest: String,
    /// Minimum audit record retention period in years.
    pub audit_logging_retention_years: u32,
    /// Formal approval status.
    pub status: String,
}

/// Brand identity, presentation, and accessibility rules.
#[derive(Debug, Clone, Deserialize)]
pub struct BrandIdentity {
    /// Approved brand name.
    pub brand_name: String,
    /// SHA-256 digest of approved brand assets and guidelines.
    pub brand_kit_digest: String,
    /// Target accessibility standard.
    pub accessibility_target: String,
    /// Enforcement of safe label rendering without raw HTML/script injection.
    pub safe_label_rendering: bool,
}

/// Authorized publication release decisions and deployment targets.
#[derive(Debug, Clone, Deserialize)]
pub struct PublicationApprovals {
    /// Authorization status for staged functional core publication.
    pub staged_core_authorized: bool,
    /// Authorization status for draft preview publication (must be false).
    pub preview_publication_authorized: bool,
    /// List of authorized publication URLs.
    pub approved_targets: Vec<String>,
    /// Pending publication targets awaiting routing confirmation.
    pub pending_targets: Vec<String>,
    /// Mailboxes of administrators signing the publication decision.
    pub approving_administrators: Vec<String>,
}

/// Payment scope constraints and certification authority rules.
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentCertificationRules {
    /// Permitted payment categories.
    pub permitted_payment_types: Vec<String>,
    /// Prohibition of speculative trading or unauthorized tokenization.
    pub prohibit_speculative_trading: bool,
    /// Requirement for verifiable settlement evidence.
    pub settlement_verification_required: bool,
    /// Approved financial counterparties.
    pub authorized_counterparties: Vec<String>,
    /// Name of the certification issuance authority.
    pub certification_issuer: String,
    /// Verification key fingerprint of the certification authority.
    pub certification_issuer_key: String,
    /// Requirement for cryptographic certificate revocation registry.
    pub certification_revocation_registry: bool,
}

/// Service availability, workload bounds, fault tolerance, and durability targets.
#[derive(Debug, Clone, Deserialize)]
pub struct ResilienceBounds {
    /// Target service availability percentage (e.g. "99.9%").
    pub service_level_objective: String,
    /// Maximum concurrent participants in a single shared workspace.
    pub max_concurrent_participants_per_workspace: u32,
    /// Maximum text/json message payload size in bytes.
    pub max_message_bytes: u64,
    /// Maximum binary blob payload size in bytes.
    pub max_blob_bytes: u64,
    /// Storage quota per shared workspace in bytes.
    pub max_storage_quota_per_workspace_bytes: u64,
    /// Maximum allowable clock drift in milliseconds.
    pub max_clock_drift_ms: u64,
    /// Byzantine fault tolerance threshold expression.
    pub byzantine_fault_tolerance_ratio: String,
    /// Recovery Point Objective for confirmed local writes in seconds.
    pub rpo_local_committed_seconds: u64,
    /// Recovery Point Objective for replicated state in seconds.
    pub rpo_replicated_state_seconds: u64,
    /// Recovery Time Objective for local session recovery in seconds.
    pub rto_local_session_seconds: u64,
    /// Recovery Time Objective for network reconciliation in seconds.
    pub rto_peer_reconciliation_seconds: u64,
    /// Minimum required independent peer replicas for durability confirmation.
    pub min_independent_peer_replicas: usize,
    /// Event log retention policy.
    pub event_log_retention: String,
    /// Tombstone and redaction policy.
    pub tombstone_redaction_policy: String,
    /// Mandatory retention period for audit logs in years.
    pub audit_retention_years: u32,
}

impl OwnerInputs {
    /// Cross-check all owner-controlled acceptance inputs against platform rules.
    pub fn check(&self) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);

        // 1. Identity & Legal Entity
        if self.organization.id.trim().is_empty() {
            return Err(bad("organization.id must not be empty".to_string()));
        }
        if self.organization.display_name.trim().is_empty() {
            return Err(bad(
                "organization.display_name must not be empty".to_string()
            ));
        }
        if self.legal_entity.statutory_filings_status != "approved" {
            return Err(bad(format!(
                "legal_entity statutory filings must be approved, got: {}",
                self.legal_entity.statutory_filings_status
            )));
        }
        if !self.legal_entity.charter_digest.starts_with("sha256:") {
            return Err(bad(
                "legal_entity.charter_digest must start with sha256:".to_string()
            ));
        }

        // 2. Sites & Assessments
        if self.sites.is_empty() {
            return Err(bad("owner inputs must define at least one site".to_string()));
        }
        if self.site_assessments.is_empty() {
            return Err(bad(
                "owner inputs must define at least one site assessment".to_string()
            ));
        }
        for assessment in &self.site_assessments {
            if !self.sites.iter().any(|s| s.id == assessment.site_id) {
                return Err(bad(format!(
                    "site assessment {} references unknown site {}",
                    assessment.id, assessment.site_id
                )));
            }
            if assessment.result != "conforming" {
                return Err(bad(format!(
                    "site assessment {} result must be conforming, got: {}",
                    assessment.id, assessment.result
                )));
            }
            if !assessment.evidence_digest.starts_with("sha256:") {
                return Err(bad(format!(
                    "site assessment {} evidence_digest must start with sha256:",
                    assessment.id
                )));
            }
        }

        // 3. Administrators, Quorums, and Activation Policy
        if self.activation_policy.minimum_active_administrators < 2 {
            return Err(bad(
                "activation_policy.minimum_active_administrators must be at least 2 for redundancy"
                    .to_string(),
            ));
        }
        if !self.activation_policy.prohibit_single_owner_bypass {
            return Err(bad(
                "activation_policy must prohibit single-owner bypass".to_string()
            ));
        }
        if self.administrators.len() < self.activation_policy.minimum_active_administrators {
            return Err(bad(format!(
                "administrators count ({}) is less than minimum active administrators ({})",
                self.administrators.len(),
                self.activation_policy.minimum_active_administrators
            )));
        }
        // Must contain designated initial administrator mailbox
        if !self
            .administrators
            .iter()
            .any(|a| a.mailbox == "trinity@uor.foundation")
        {
            return Err(bad(
                "administrators must contain designated initial mailbox trinity@uor.foundation"
                    .to_string(),
            ));
        }
        // Check for distinct mailboxes and keys
        let mut seen_mailboxes = Vec::new();
        let mut seen_keys = Vec::new();
        for admin in &self.administrators {
            if seen_mailboxes.contains(&admin.mailbox.as_str()) {
                return Err(bad(format!(
                    "duplicate administrator mailbox: {}",
                    admin.mailbox
                )));
            }
            seen_mailboxes.push(admin.mailbox.as_str());

            if seen_keys.contains(&admin.public_key.as_str()) {
                return Err(bad(format!(
                    "duplicate administrator public key for: {}",
                    admin.mailbox
                )));
            }
            seen_keys.push(admin.public_key.as_str());

            if admin.status != "authenticated" {
                return Err(bad(format!(
                    "administrator {} must be authenticated, got: {}",
                    admin.mailbox, admin.status
                )));
            }
            if admin.scopes.is_empty() {
                return Err(bad(format!(
                    "administrator {} must have at least one authorized scope",
                    admin.mailbox
                )));
            }
        }

        // Verify quorum coverage per scope: required scopes must have at least 2 distinct admins
        let required_scopes = ["organization", "security"];
        for scope in required_scopes {
            let count = self
                .administrators
                .iter()
                .filter(|a| a.scopes.iter().any(|s| s == scope))
                .count();
            if count < 2 {
                return Err(bad(format!(
                    "scope '{scope}' requires at least 2 distinct active administrators, found {count}"
                )));
            }
        }

        // 4. Membership Policy
        if !self.membership_policy.prohibit_self_appointment {
            return Err(bad(
                "membership_policy must prohibit self-appointment".to_string()
            ));
        }

        // 5. Recovery Rules
        if self.recovery_rules.email_challenge_ttl_seconds == 0 {
            return Err(bad(
                "email_challenge_ttl_seconds must be greater than 0".to_string()
            ));
        }
        if !self.recovery_rules.email_challenge_replay_protection {
            return Err(bad(
                "email_challenge_replay_protection must be enabled".to_string()
            ));
        }
        if self.recovery_rules.backup_code_entropy_bits < 128 {
            return Err(bad(
                "backup_code_entropy_bits must be at least 128 bits per NIST SP 800-63B"
                    .to_string(),
            ));
        }
        if !self.recovery_rules.backup_code_single_use {
            return Err(bad("backup_code_single_use must be enabled".to_string()));
        }
        if !self.recovery_rules.session_invalidation_on_recovery {
            return Err(bad(
                "session_invalidation_on_recovery must be enabled".to_string()
            ));
        }

        // 6. Standards & Authorities
        if self.standards.is_empty() {
            return Err(bad("owner inputs must define adopted standards".to_string()));
        }
        if self.assessment_authorities.is_empty() {
            return Err(bad(
                "owner inputs must define assessment authorities".to_string()
            ));
        }

        // 7. Business Operations & Brand
        if self.business_operations.status != "approved" {
            return Err(bad(format!(
                "business_operations status must be approved, got: {}",
                self.business_operations.status
            )));
        }
        if !self
            .business_operations
            .business_plan_digest
            .starts_with("sha256:")
        {
            return Err(bad(
                "business_plan_digest must start with sha256:".to_string()
            ));
        }
        if !self.brand_identity.safe_label_rendering {
            return Err(bad(
                "brand_identity.safe_label_rendering must be enabled".to_string()
            ));
        }

        // 8. Publication Approvals
        if !self.publication_approvals.staged_core_authorized {
            return Err(bad(
                "staged_core_authorized must be true for initial release".to_string(),
            ));
        }
        if self.publication_approvals.preview_publication_authorized {
            return Err(bad(
                "preview publication is not authorized by platform policy".to_string(),
            ));
        }
        if !self
            .publication_approvals
            .approved_targets
            .contains(&"https://uor-foundation.github.io/foundry-web/".to_string())
        {
            return Err(bad(
                "approved_targets must contain default publication target https://uor-foundation.github.io/foundry-web/"
                    .to_string(),
            ));
        }
        if self.publication_approvals.approving_administrators.len() < 2 {
            return Err(bad(
                "publication approvals require at least 2 approving administrators".to_string(),
            ));
        }

        // 9. Payment & Certification Rules
        if !self
            .payment_certification_rules
            .settlement_verification_required
        {
            return Err(bad(
                "settlement_verification_required must be true".to_string()
            ));
        }
        if !self
            .payment_certification_rules
            .prohibit_speculative_trading
        {
            return Err(bad("prohibit_speculative_trading must be true".to_string()));
        }

        // 10. Resilience Bounds
        if self
            .resilience_bounds
            .max_concurrent_participants_per_workspace
            < 2
        {
            return Err(bad(
                "max_concurrent_participants_per_workspace must be at least 2".to_string(),
            ));
        }
        if self.resilience_bounds.rpo_local_committed_seconds != 0 {
            return Err(bad(
                "rpo_local_committed_seconds must be 0 for committed local state".to_string(),
            ));
        }
        if self.resilience_bounds.rto_local_session_seconds > 5 {
            return Err(bad(
                "rto_local_session_seconds must be at most 5 seconds".to_string()
            ));
        }
        if self.resilience_bounds.rto_peer_reconciliation_seconds > 30 {
            return Err(bad(
                "rto_peer_reconciliation_seconds must be at most 30 seconds".to_string(),
            ));
        }
        if self.resilience_bounds.min_independent_peer_replicas < 2 {
            return Err(bad(
                "min_independent_peer_replicas must be at least 2".to_string()
            ));
        }
        if self.resilience_bounds.audit_retention_years < 7 {
            return Err(bad(
                "audit_retention_years must be at least 7 years".to_string()
            ));
        }

        Ok(())
    }
}
