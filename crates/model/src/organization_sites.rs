//! Organization site lifecycle, activation quorums, physical/accessibility
//! assessments, and cross-organization isolation (OS-01).
//!
//! Under SPEC.md:
//! 1. Model normal creation, provisional setup, activation, isolated records, and site lifecycles.
//! 2. UOR's Foundation, HQ, First Foundry, and Citizen Gardens records use these same workflows;
//!    validate authorized policies and applicable physical/human assessments without seeded privileges.
//! 3. Cross-organization isolation is strictly enforced across records, states, and operations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::organization::{OrganizationLifecycleConfig, OrganizationLifecycleState};
use crate::owner_inputs::OwnerInputs;
use crate::ModelError;

/// Site lifecycle states per SPEC.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SiteLifecycleState {
    /// Provisional setup; site undergoing facility commissioning and assessments.
    Provisional,
    /// Fully active operational site meeting quorum and assessment requirements.
    Active,
    /// Temporarily offline for maintenance or repairs.
    Maintenance,
    /// Formally decommissioned site with preserved audit logs.
    Decommissioned,
}

impl SiteLifecycleState {
    /// Display label for the state.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Provisional => "provisional",
            Self::Active => "active",
            Self::Maintenance => "maintenance",
            Self::Decommissioned => "decommissioned",
        }
    }
}

/// Root configuration of `model/organization_sites.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteLifecycleConfig {
    /// Schema version tag.
    pub spec: String,
    /// Policy governing site activation and isolation.
    pub policy: SitePolicyConfig,
    /// Modeled site records.
    pub sites: Vec<SiteLifecycleRecord>,
    /// Modeled site assessments.
    pub assessments: Vec<SiteAssessmentLifecycleRecord>,
}

/// Policy constraints on site lifecycle.
#[derive(Debug, Clone, Deserialize)]
pub struct SitePolicyConfig {
    /// Minimum distinct administrators required to activate a site.
    pub activation_minimum_distinct_administrators: usize,
    /// Requirement for conforming physical security assessment.
    pub require_physical_security_assessment: bool,
    /// Requirement for conforming accessibility assessment.
    pub require_accessibility_assessment: bool,
    /// Prohibition of seeded privilege shortcuts or bypasses.
    pub prohibit_seeded_site_privileges: bool,
    /// Requirement that parent organization must be in Activated state.
    pub require_parent_organization_active: bool,
    /// Enforcement of cross-organization isolation.
    pub enforce_cross_org_isolation: bool,
}

/// A declarative site record in `model/organization_sites.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteLifecycleRecord {
    /// Unique UOR site identifier (e.g. `uor:site:uor-hq`).
    pub id: String,
    /// Owning organization identifier (e.g. `uor:org:uor-foundation`).
    pub organization_id: String,
    /// Display name of the site.
    pub name: String,
    /// Category of facility.
    pub site_type: String,
    /// Physical address.
    pub address: String,
    /// Governing jurisdiction.
    pub jurisdiction: String,
    /// Initial lifecycle state.
    pub initial_state: SiteLifecycleState,
    /// Administrators assigned operational authority for this site.
    pub assigned_administrators: Vec<String>,
    /// List of assessment identifiers required for activation.
    pub required_assessments: Vec<String>,
}

/// A site assessment record in `model/organization_sites.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SiteAssessmentLifecycleRecord {
    /// Unique assessment identifier.
    pub id: String,
    /// Target site identifier.
    pub site_id: String,
    /// Owning organization identifier.
    pub organization_id: String,
    /// Domain of assessment (e.g. `physical-security`, `accessibility`).
    pub assessment_type: String,
    /// Assessing body or authority.
    pub assessor: String,
    /// Normative standard applied.
    pub standard_applied: String,
    /// Result outcome (must be `conforming`).
    pub result: String,
    /// SHA-256 digest of verified evidence findings.
    pub evidence_digest: String,
    /// Expiration date of validity.
    pub valid_until: String,
}

/// Active operational state of a site managed by `SiteManager`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSiteState {
    /// Site identifier.
    pub id: String,
    /// Owning organization.
    pub organization_id: String,
    /// Display name.
    pub name: String,
    /// Classification.
    pub site_type: String,
    /// Physical location.
    pub address: String,
    /// Jurisdiction.
    pub jurisdiction: String,
    /// Current lifecycle state.
    pub state: SiteLifecycleState,
    /// Assigned administrators.
    pub assigned_administrators: Vec<String>,
    /// Confirmed assessments.
    pub confirmed_assessments: Vec<String>,
}

/// Request to create a new site.
#[derive(Debug, Clone)]
pub struct CreateSiteRequest {
    /// Site identifier.
    pub id: String,
    /// Owning organization.
    pub organization_id: String,
    /// Display name.
    pub name: String,
    /// Facility classification.
    pub site_type: String,
    /// Physical address.
    pub address: String,
    /// Jurisdiction.
    pub jurisdiction: String,
    /// Initial assigned administrator.
    pub creator_mailbox: String,
}

/// Request to activate a provisional site.
#[derive(Debug, Clone)]
pub struct ActivateSiteRequest {
    /// Target site identifier.
    pub site_id: String,
    /// Owning organization identifier.
    pub organization_id: String,
    /// Mailboxes of distinct approving administrators.
    pub approving_administrators: Vec<String>,
}

/// Request to transition a site's operational state.
#[derive(Debug, Clone)]
pub struct TransitionSiteRequest {
    /// Target site identifier.
    pub site_id: String,
    /// Owning organization.
    pub organization_id: String,
    /// Target state.
    pub target_state: SiteLifecycleState,
    /// Mailboxes of distinct approving administrators.
    pub approving_administrators: Vec<String>,
}

/// Request to access site records across or within organizations.
#[derive(Debug, Clone)]
pub struct AccessSiteRequest {
    /// Caller's organization identifier.
    pub caller_org_id: String,
    /// Target site identifier.
    pub site_id: String,
}

/// Domain errors for site lifecycle and isolation boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteError {
    /// Site not found.
    SiteNotFound(String),
    /// Site already exists.
    SiteAlreadyExists(String),
    /// Parent organization is not in Activated state.
    ParentOrganizationNotActive {
        /// Site ID.
        site_id: String,
        /// Current parent org state.
        org_state: String,
    },
    /// Insufficient distinct approving administrators for quorum.
    InsufficientApprovers {
        /// Site ID.
        site_id: String,
        /// Number of valid approvers found.
        found: usize,
        /// Minimum required approvers.
        minimum: usize,
    },
    /// Duplicate approving administrator detected.
    DuplicateApprover {
        /// Site ID.
        site_id: String,
        /// Duplicate mailbox.
        mailbox: String,
    },
    /// Approving administrator is not authorized for this site.
    UnauthorizedApprover {
        /// Site ID.
        site_id: String,
        /// Mailbox.
        mailbox: String,
    },
    /// Required physical security assessment is missing or non-conforming.
    MissingPhysicalSecurityAssessment(String),
    /// Required accessibility assessment is missing or non-conforming.
    MissingAccessibilityAssessment(String),
    /// Assessment evidence digest is invalid.
    InvalidEvidenceDigest {
        /// Assessment ID.
        assessment_id: String,
        /// Invalid digest string.
        digest: String,
    },
    /// Cross-organization isolation boundary violation.
    CrossOrgIsolationViolation {
        /// Site ID.
        site_id: String,
        /// Caller's organization.
        caller_org: String,
        /// Site's true owning organization.
        site_org: String,
    },
    /// Invalid state transition attempted.
    InvalidTransition {
        /// Site ID.
        site_id: String,
        /// Source state.
        from: SiteLifecycleState,
        /// Destination state.
        to: SiteLifecycleState,
    },
    /// Seeded privilege bypass attempted.
    SeededPrivilegeBypass(String),
}

impl std::fmt::Display for SiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SiteNotFound(id) => write!(f, "site not found: {id}"),
            Self::SiteAlreadyExists(id) => write!(f, "site already exists: {id}"),
            Self::ParentOrganizationNotActive { site_id, org_state } => {
                write!(
                    f,
                    "cannot activate site {site_id}: parent organization is {org_state}"
                )
            }
            Self::InsufficientApprovers {
                site_id,
                found,
                minimum,
            } => {
                write!(f, "site {site_id} requires {minimum} distinct approving administrators, found {found}")
            }
            Self::DuplicateApprover { site_id, mailbox } => {
                write!(f, "duplicate approval by {mailbox} on site {site_id}")
            }
            Self::UnauthorizedApprover { site_id, mailbox } => {
                write!(
                    f,
                    "administrator {mailbox} is not authorized on site {site_id}"
                )
            }
            Self::MissingPhysicalSecurityAssessment(id) => {
                write!(
                    f,
                    "site {id} missing conforming physical security assessment"
                )
            }
            Self::MissingAccessibilityAssessment(id) => {
                write!(f, "site {id} missing conforming accessibility assessment")
            }
            Self::InvalidEvidenceDigest {
                assessment_id,
                digest,
            } => {
                write!(f, "assessment {assessment_id} evidence digest must start with sha256:, got: {digest}")
            }
            Self::CrossOrgIsolationViolation {
                site_id,
                caller_org,
                site_org,
            } => {
                write!(f, "cross-organization isolation: {caller_org} cannot access site {site_id} of {site_org}")
            }
            Self::InvalidTransition { site_id, from, to } => {
                write!(
                    f,
                    "invalid transition for site {site_id} from {} to {}",
                    from.as_str(),
                    to.as_str()
                )
            }
            Self::SeededPrivilegeBypass(msg) => {
                write!(f, "seeded privilege bypass rejected: {msg}")
            }
        }
    }
}

impl std::error::Error for SiteError {}

/// Operational state manager for organization sites.
#[derive(Debug, Clone)]
pub struct SiteManager {
    policy: SitePolicyConfig,
    sites: HashMap<String, ActiveSiteState>,
    assessments: HashMap<String, SiteAssessmentLifecycleRecord>,
}

impl SiteManager {
    /// Create a new `SiteManager` initialized from configuration.
    pub fn new(config: &SiteLifecycleConfig) -> Self {
        let mut sites = HashMap::new();
        for s in &config.sites {
            sites.insert(
                s.id.clone(),
                ActiveSiteState {
                    id: s.id.clone(),
                    organization_id: s.organization_id.clone(),
                    name: s.name.clone(),
                    site_type: s.site_type.clone(),
                    address: s.address.clone(),
                    jurisdiction: s.jurisdiction.clone(),
                    state: s.initial_state,
                    assigned_administrators: s.assigned_administrators.clone(),
                    confirmed_assessments: s.required_assessments.clone(),
                },
            );
        }

        let mut assessments = HashMap::new();
        for a in &config.assessments {
            assessments.insert(a.id.clone(), a.clone());
        }

        Self {
            policy: config.policy.clone(),
            sites,
            assessments,
        }
    }

    /// Open enrollment/creation of a new site in `Provisional` state.
    pub fn create_site(&mut self, req: CreateSiteRequest) -> Result<String, SiteError> {
        if self.sites.contains_key(&req.id) {
            return Err(SiteError::SiteAlreadyExists(req.id));
        }

        let site = ActiveSiteState {
            id: req.id.clone(),
            organization_id: req.organization_id,
            name: req.name,
            site_type: req.site_type,
            address: req.address,
            jurisdiction: req.jurisdiction,
            state: SiteLifecycleState::Provisional,
            assigned_administrators: vec![req.creator_mailbox],
            confirmed_assessments: Vec::new(),
        };

        self.sites.insert(req.id.clone(), site);
        Ok(req.id)
    }

    /// Add an assessment record to a site.
    pub fn record_assessment(
        &mut self,
        assessment: SiteAssessmentLifecycleRecord,
    ) -> Result<(), SiteError> {
        if !assessment.evidence_digest.starts_with("sha256:") {
            return Err(SiteError::InvalidEvidenceDigest {
                assessment_id: assessment.id,
                digest: assessment.evidence_digest,
            });
        }

        let site = self
            .sites
            .get_mut(&assessment.site_id)
            .ok_or_else(|| SiteError::SiteNotFound(assessment.site_id.clone()))?;

        if site.organization_id != assessment.organization_id {
            return Err(SiteError::CrossOrgIsolationViolation {
                site_id: site.id.clone(),
                caller_org: assessment.organization_id,
                site_org: site.organization_id.clone(),
            });
        }

        if !site.confirmed_assessments.contains(&assessment.id) {
            site.confirmed_assessments.push(assessment.id.clone());
        }

        self.assessments.insert(assessment.id.clone(), assessment);
        Ok(())
    }

    /// Assign an additional administrator to a site.
    pub fn assign_administrator(&mut self, site_id: &str, mailbox: &str) -> Result<(), SiteError> {
        let site = self
            .sites
            .get_mut(site_id)
            .ok_or_else(|| SiteError::SiteNotFound(site_id.to_string()))?;

        if !site.assigned_administrators.contains(&mailbox.to_string()) {
            site.assigned_administrators.push(mailbox.to_string());
        }
        Ok(())
    }

    /// Activate a provisional site under strict multi-admin quorum and assessment coverage.
    pub fn activate_site(
        &mut self,
        req: ActivateSiteRequest,
        parent_org_state: OrganizationLifecycleState,
    ) -> Result<(), SiteError> {
        let site = self
            .sites
            .get_mut(&req.site_id)
            .ok_or_else(|| SiteError::SiteNotFound(req.site_id.clone()))?;

        // 1. Cross-org check
        if site.organization_id != req.organization_id {
            return Err(SiteError::CrossOrgIsolationViolation {
                site_id: site.id.clone(),
                caller_org: req.organization_id,
                site_org: site.organization_id.clone(),
            });
        }

        // 2. Parent organization must be active
        if self.policy.require_parent_organization_active
            && parent_org_state != OrganizationLifecycleState::Activated
        {
            return Err(SiteError::ParentOrganizationNotActive {
                site_id: site.id.clone(),
                org_state: parent_org_state.as_str().to_string(),
            });
        }

        // 3. State transition validity
        if site.state != SiteLifecycleState::Provisional {
            return Err(SiteError::InvalidTransition {
                site_id: site.id.clone(),
                from: site.state,
                to: SiteLifecycleState::Active,
            });
        }

        // 4. Distinct-user quorum check
        let mut seen = HashSet::new();
        for admin in &req.approving_administrators {
            if !seen.insert(admin.as_str()) {
                return Err(SiteError::DuplicateApprover {
                    site_id: site.id.clone(),
                    mailbox: admin.clone(),
                });
            }
            if !site.assigned_administrators.contains(admin) {
                return Err(SiteError::UnauthorizedApprover {
                    site_id: site.id.clone(),
                    mailbox: admin.clone(),
                });
            }
        }

        if seen.len() < self.policy.activation_minimum_distinct_administrators {
            return Err(SiteError::InsufficientApprovers {
                site_id: site.id.clone(),
                found: seen.len(),
                minimum: self.policy.activation_minimum_distinct_administrators,
            });
        }

        // 5. Physical security assessment coverage
        if self.policy.require_physical_security_assessment {
            let has_phys = site.confirmed_assessments.iter().any(|aid| {
                if let Some(a) = self.assessments.get(aid) {
                    a.assessment_type == "physical-security" && a.result == "conforming"
                } else {
                    false
                }
            });
            if !has_phys {
                return Err(SiteError::MissingPhysicalSecurityAssessment(
                    site.id.clone(),
                ));
            }
        }

        // 6. Accessibility assessment coverage
        if self.policy.require_accessibility_assessment {
            let has_access = site.confirmed_assessments.iter().any(|aid| {
                if let Some(a) = self.assessments.get(aid) {
                    a.assessment_type == "accessibility" && a.result == "conforming"
                } else {
                    false
                }
            });
            if !has_access {
                return Err(SiteError::MissingAccessibilityAssessment(site.id.clone()));
            }
        }

        site.state = SiteLifecycleState::Active;
        Ok(())
    }

    /// Transition a site to another state with distinct multi-admin quorum.
    pub fn transition_site(&mut self, req: TransitionSiteRequest) -> Result<(), SiteError> {
        let site = self
            .sites
            .get_mut(&req.site_id)
            .ok_or_else(|| SiteError::SiteNotFound(req.site_id.clone()))?;

        if site.organization_id != req.organization_id {
            return Err(SiteError::CrossOrgIsolationViolation {
                site_id: site.id.clone(),
                caller_org: req.organization_id,
                site_org: site.organization_id.clone(),
            });
        }

        // Check transition validity
        match (site.state, req.target_state) {
            (SiteLifecycleState::Active, SiteLifecycleState::Maintenance)
            | (SiteLifecycleState::Maintenance, SiteLifecycleState::Active)
            | (SiteLifecycleState::Active, SiteLifecycleState::Decommissioned)
            | (SiteLifecycleState::Maintenance, SiteLifecycleState::Decommissioned) => {}
            _ => {
                return Err(SiteError::InvalidTransition {
                    site_id: site.id.clone(),
                    from: site.state,
                    to: req.target_state,
                });
            }
        }

        // Validate distinct approver quorum
        let mut seen = HashSet::new();
        for admin in &req.approving_administrators {
            if !seen.insert(admin.as_str()) {
                return Err(SiteError::DuplicateApprover {
                    site_id: site.id.clone(),
                    mailbox: admin.clone(),
                });
            }
            if !site.assigned_administrators.contains(admin) {
                return Err(SiteError::UnauthorizedApprover {
                    site_id: site.id.clone(),
                    mailbox: admin.clone(),
                });
            }
        }

        if seen.len() < self.policy.activation_minimum_distinct_administrators {
            return Err(SiteError::InsufficientApprovers {
                site_id: site.id.clone(),
                found: seen.len(),
                minimum: self.policy.activation_minimum_distinct_administrators,
            });
        }

        site.state = req.target_state;
        Ok(())
    }

    /// Retrieve site details, enforcing cross-organization isolation.
    pub fn access_site_records(
        &self,
        req: AccessSiteRequest,
    ) -> Result<&ActiveSiteState, SiteError> {
        let site = self
            .sites
            .get(&req.site_id)
            .ok_or_else(|| SiteError::SiteNotFound(req.site_id.clone()))?;

        if self.policy.enforce_cross_org_isolation && site.organization_id != req.caller_org_id {
            return Err(SiteError::CrossOrgIsolationViolation {
                site_id: site.id.clone(),
                caller_org: req.caller_org_id,
                site_org: site.organization_id.clone(),
            });
        }

        Ok(site)
    }

    /// Get total active sites.
    pub fn active_site_count(&self) -> usize {
        self.sites
            .values()
            .filter(|s| s.state == SiteLifecycleState::Active)
            .count()
    }
}

impl SiteLifecycleConfig {
    /// Cross-check site configuration against owner inputs and organization lifecycle.
    pub fn check(
        &self,
        owner_inputs: &OwnerInputs,
        org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);

        if self.policy.activation_minimum_distinct_administrators < 2 {
            return Err(bad(
                "site activation requires at least 2 distinct administrators".to_string(),
            ));
        }

        if !self.policy.require_physical_security_assessment {
            return Err(bad(
                "require_physical_security_assessment must be true".to_string()
            ));
        }

        if !self.policy.require_accessibility_assessment {
            return Err(bad(
                "require_accessibility_assessment must be true".to_string()
            ));
        }

        if !self.policy.prohibit_seeded_site_privileges {
            return Err(bad(
                "prohibit_seeded_site_privileges must be true".to_string()
            ));
        }

        // Validate that owner inputs sites have corresponding lifecycle definitions
        for input_site in &owner_inputs.sites {
            let site = self
                .sites
                .iter()
                .find(|s| s.id == input_site.id || s.name == input_site.name)
                .ok_or_else(|| {
                    bad(format!(
                        "owner_inputs site '{}' has no matching organization site record",
                        input_site.name
                    ))
                })?;

            if site.organization_id != owner_inputs.organization.id {
                return Err(bad(format!(
                    "site {} organization_id mismatch: expected {}, got {}",
                    site.id, owner_inputs.organization.id, site.organization_id
                )));
            }
        }

        // Validate all assessments
        for a in &self.assessments {
            if !a.evidence_digest.starts_with("sha256:") {
                return Err(bad(format!(
                    "site assessment {} evidence digest must start with sha256:",
                    a.id
                )));
            }
            if a.result != "conforming" {
                return Err(bad(format!(
                    "site assessment {} result must be conforming, got {}",
                    a.id, a.result
                )));
            }
        }

        // Initialize and verify manager
        let mgr = SiteManager::new(self);
        if mgr.active_site_count() == 0 {
            return Err(bad(
                "organization must have at least one active site".to_string()
            ));
        }

        // Confirm parent org rules align
        if org_lifecycle
            .rules
            .activation_minimum_distinct_administrators
            != self.policy.activation_minimum_distinct_administrators
        {
            return Err(bad(
                "site activation quorum must match organization lifecycle minimum administrators"
                    .to_string(),
            ));
        }

        Ok(())
    }
}
