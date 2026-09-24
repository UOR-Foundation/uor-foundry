//! OSCAL catalogs, profile resolution, component/system records, inheritance,
//! and authenticated assessments.
//!
//! Conformance ID: ST-01 (suite: standards-oscal).
//!
//! SPEC.md requires:
//! "Platform and organization facets are governed through OSCAL catalogs, resolved
//! profiles, component/system implementation records and assessment evidence. Every applicable
//! control must have a verified local implementation, verified inheritance, or
//! both. Inheritance identifies provider scope, exact subjects and revisions,
//! evidence, validity conditions, and consumer responsibilities.
//!
//! The mandatory PrismPM base profile cannot be weakened by an overlay or an
//! unsupported inapplicability claim. Missing implementations, planned work,
//! unassessed claims, and remediation records do not satisfy mandatory controls.
//!
//! Every adopted standard requires a complete normative-requirement inventory,
//! lawfully acquired pinned sources, formal bindings, implementation mappings,
//! and the complete applicable authoritative oracle/assessment coverage."

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::owner_inputs::{AdoptedStandard, AssessmentAuthority, OwnerInputs};
use crate::ModelError;

/// Valid assessment methods recognized by the standards boundary.
pub const VALID_ASSESSMENT_METHODS: &[&str] = &[
    "formal-proof",
    "test-corpus-agreement",
    "observed-operation",
    "human-assessment",
    "structural-validation",
];

/// Root configuration of `model/standards.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct StandardsConfig {
    /// Schema spec identifier.
    pub spec: String,
    /// Standards governance policy.
    pub policy: StandardsPolicy,
    /// OSCAL catalogs defining normative controls for adopted standards.
    pub catalogs: Vec<OscalCatalog>,
    /// OSCAL profiles defining catalog inclusion, overlays, and parameters.
    pub profiles: Vec<OscalProfile>,
    /// System and platform component implementation records.
    pub components: Vec<OscalComponent>,
    /// System records binding profiles, components, and inheritance.
    pub systems: Vec<OscalSystemRecord>,
    /// Authenticated assessment records signed by authorized assessment bodies.
    pub assessments: Vec<OscalAssessmentRecord>,
}

/// Standards governance policy constraints.
#[derive(Debug, Clone, Deserialize)]
pub struct StandardsPolicy {
    /// Identifier of the mandatory base profile catalog.
    pub mandatory_base_profile: String,
    /// Prohibition of unsupported control inapplicability claims.
    pub prohibit_unsupported_inapplicability: bool,
    /// Prohibition of weakening or excluding base profile controls.
    pub prohibit_weakening_base_profile: bool,
    /// Requirement for conforming assessment verdict.
    pub require_conforming_verdict: bool,
    /// Requirement for cryptographically bound authenticated assessors.
    pub require_authenticated_assessor: bool,
    /// Requirement for complete authoritative assessment coverage across all controls.
    pub require_authoritative_coverage: bool,
    /// Requirement for explicit inheritance contracts.
    pub require_explicit_inheritance: bool,
    /// Enforcement of cross-organization assessment isolation.
    pub enforce_cross_org_isolation: bool,
}

/// An OSCAL Catalog containing normative controls.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalCatalog {
    /// Catalog identifier (e.g. `CAT-NIST-800-63B`).
    pub catalog_id: String,
    /// Standard identifier matching `model/owner_inputs.toml`.
    pub standard_id: String,
    /// Canonical human-readable standard name.
    pub canonical_id: String,
    /// Standard edition / version.
    pub edition: String,
    /// Title of the catalog.
    pub title: String,
    /// List of normative controls in this catalog.
    pub controls: Vec<OscalControl>,
}

/// A normative control within an OSCAL catalog.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalControl {
    /// Control identifier (e.g. `NIST-800-63B:BACKUP-CODES`).
    pub id: String,
    /// Human-readable title of the control.
    pub title: String,
    /// Normative statement / requirement prose.
    pub statement: String,
}

/// An OSCAL Profile importing and tailoring catalogs.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalProfile {
    /// Profile identifier (e.g. `PROF-FOUNDRY-PRODUCTION`).
    pub profile_id: String,
    /// Profile title.
    pub title: String,
    /// List of catalog IDs imported by this profile.
    pub import_catalogs: Vec<String>,
    /// Identifier of mandatory base catalog.
    pub mandatory_base_profile: String,
    /// List of explicitly excluded control IDs (must not include base controls).
    #[serde(default)]
    pub excluded_controls: Vec<String>,
}

/// A component implementation record.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalComponent {
    /// Component identifier (e.g. `COMP-PRISMPM-SDK`, `COMP-UOR-FOUNDRY-CORE`).
    pub component_id: String,
    /// Human-readable title of the component.
    pub title: String,
    /// Classification of component (e.g. `upstream-platform`, `service-core`, `client-presentation`).
    pub component_type: String,
    /// List of requirements implemented by this component.
    pub implemented_requirements: Vec<ImplementedRequirement>,
}

/// A control requirement implemented by a component.
#[derive(Debug, Clone, Deserialize)]
pub struct ImplementedRequirement {
    /// Target control ID.
    pub control_id: String,
    /// Description of the implementation.
    pub description: String,
    /// Implementation status (must be `implemented` to satisfy mandatory controls).
    pub status: String,
}

/// A System Security Plan / System Record.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalSystemRecord {
    /// System identifier (e.g. `SYS-UOR-FOUNDRY`).
    pub system_id: String,
    /// Organization identifier.
    pub organization_id: String,
    /// Bound profile identifier.
    pub profile_id: String,
    /// System display name.
    pub system_name: String,
    /// List of component IDs active in this system.
    pub components: Vec<String>,
    /// Explicit inherited control records from upstream providers.
    #[serde(default)]
    pub inherited_controls: Vec<InheritedControlRecord>,
}

/// An inherited control satisfaction record.
#[derive(Debug, Clone, Deserialize)]
pub struct InheritedControlRecord {
    /// Inherited control ID.
    pub control_id: String,
    /// Provider component ID delivering the control.
    pub provider_component_id: String,
    /// Provider organization identifier.
    pub provider_organization: String,
    /// Provider scope of responsibility.
    pub provider_scope: String,
    /// Exact subject and revision (e.g. git commit hash, OCI digest).
    pub exact_subject_and_revision: String,
    /// SHA-256 digest of upstream evidence.
    pub evidence_digest: String,
    /// Operating validity conditions.
    pub validity_conditions: String,
    /// Downstream consumer responsibilities.
    pub consumer_responsibilities: String,
}

/// An authenticated assessment record signed by an authorized body.
#[derive(Debug, Clone, Deserialize)]
pub struct OscalAssessmentRecord {
    /// Unique assessment identifier.
    pub assessment_id: String,
    /// Target organization identifier.
    pub organization_id: String,
    /// Assessing authority ID from `owner_inputs.assessment_authorities`.
    pub assessor_authority_id: String,
    /// Cryptographic verification key or signature fingerprint.
    pub assessor_signature_key: String,
    /// Standard evaluated.
    pub standard_id: String,
    /// Controls confirmed by this assessment.
    pub assessed_controls: Vec<String>,
    /// Formal assessment method.
    pub assessment_method: String,
    /// Assessment outcome (must be `conforming`).
    pub verdict: String,
    /// SHA-256 digest of verified evidence findings.
    pub evidence_digest: String,
    /// Assessment execution date (ISO 8601).
    pub assessed_date: String,
    /// Expiration date of assessment validity.
    pub valid_until: String,
}

/// A resolved profile containing the unified active controls.
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    /// Profile identifier.
    pub profile_id: String,
    /// Profile title.
    pub title: String,
    /// Active resolved controls.
    pub controls: Vec<OscalControl>,
}

/// Verification report summarizing authenticated assessment findings.
#[derive(Debug, Clone)]
pub struct AssessmentVerificationReport {
    /// Total resolved controls evaluated.
    pub total_controls: usize,
    /// Total controls with conforming assessment evidence.
    pub conforming_controls: usize,
    /// List of assessing authority IDs.
    pub authorities_cited: Vec<String>,
}

/// Domain errors for standards, OSCAL resolution, and assessment verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardsError {
    /// Profile not found.
    ProfileNotFound(String),
    /// System record not found.
    SystemNotFound(String),
    /// Catalog not found.
    CatalogNotFound(String),
    /// Mandatory base profile is missing from profile imports.
    MandatoryBaseProfileMissing {
        /// Profile ID.
        profile_id: String,
        /// Expected base catalog ID.
        expected_base: String,
    },
    /// Base profile control was improperly weakened or excluded.
    BaseProfileWeakened {
        /// Profile ID.
        profile_id: String,
        /// Excluded control ID.
        control_id: String,
    },
    /// A control has no valid local or inherited implementation.
    UnsatisfiedControl {
        /// Control ID.
        control_id: String,
        /// Reason for lack of satisfaction.
        reason: String,
    },
    /// An inherited control record is missing a mandatory field.
    IncompleteInheritance {
        /// Control ID.
        control_id: String,
        /// Missing field description.
        field: &'static str,
    },
    /// An adopted standard has no matching OSCAL catalog.
    StandardNotCataloged {
        /// Standard ID.
        standard_id: String,
        /// Expected edition.
        edition: String,
    },
    /// Assessor authority is not recognized in owner inputs.
    AssessorNotAuthorized {
        /// Assessment ID.
        assessment_id: String,
        /// Authority ID.
        authority_id: String,
    },
    /// Assessor signature key does not match registered verification key.
    AssessorKeyMismatch {
        /// Assessment ID.
        assessment_id: String,
        /// Authority ID.
        authority_id: String,
        /// Expected registered key.
        expected: String,
        /// Found key.
        found: String,
    },
    /// Assessment verdict is non-conforming.
    NonConformingAssessment {
        /// Assessment ID.
        assessment_id: String,
        /// Verdict string.
        verdict: String,
    },
    /// Assessment evidence digest is malformed.
    InvalidEvidenceDigest {
        /// Assessment ID.
        assessment_id: String,
        /// Digest string.
        digest: String,
    },
    /// Assessment method is invalid.
    InvalidAssessmentMethod {
        /// Assessment ID.
        assessment_id: String,
        /// Invalid method string.
        method: String,
    },
    /// A control in the resolved profile lacks conforming authenticated assessment coverage.
    UnassessedControl {
        /// Control ID.
        control_id: String,
    },
    /// Cross-organization assessment isolation violation.
    CrossOrgIsolationViolation {
        /// Assessment ID.
        assessment_id: String,
        /// Target organization.
        expected_org: String,
        /// Evaluated organization.
        found_org: String,
    },
    /// Adopted standard lacks assessment coverage.
    MissingStandardAssessment {
        /// Standard ID.
        standard_id: String,
    },
}

impl std::fmt::Display for StandardsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProfileNotFound(id) => write!(f, "profile not found: {id}"),
            Self::SystemNotFound(id) => write!(f, "system record not found: {id}"),
            Self::CatalogNotFound(id) => write!(f, "catalog not found: {id}"),
            Self::MandatoryBaseProfileMissing {
                profile_id,
                expected_base,
            } => {
                write!(
                    f,
                    "profile {profile_id} is missing mandatory base profile: {expected_base}"
                )
            }
            Self::BaseProfileWeakened {
                profile_id,
                control_id,
            } => {
                write!(
                    f,
                    "profile {profile_id} weakens base profile by excluding control: {control_id}"
                )
            }
            Self::UnsatisfiedControl { control_id, reason } => {
                write!(f, "control {control_id} is not satisfied: {reason}")
            }
            Self::IncompleteInheritance { control_id, field } => {
                write!(
                    f,
                    "inherited control {control_id} missing mandatory contract field: {field}"
                )
            }
            Self::StandardNotCataloged {
                standard_id,
                edition,
            } => {
                write!(
                    f,
                    "adopted standard {standard_id} (edition {edition}) is not cataloged"
                )
            }
            Self::AssessorNotAuthorized {
                assessment_id,
                authority_id,
            } => {
                write!(
                    f,
                    "assessment {assessment_id} cites unauthorized authority: {authority_id}"
                )
            }
            Self::AssessorKeyMismatch {
                assessment_id,
                authority_id,
                expected,
                found,
            } => {
                write!(f, "assessment {assessment_id} key mismatch for {authority_id}: expected {expected}, got {found}")
            }
            Self::NonConformingAssessment {
                assessment_id,
                verdict,
            } => {
                write!(
                    f,
                    "assessment {assessment_id} has non-conforming verdict: {verdict}"
                )
            }
            Self::InvalidEvidenceDigest {
                assessment_id,
                digest,
            } => {
                write!(f, "assessment {assessment_id} evidence digest must start with sha256:, got: {digest}")
            }
            Self::InvalidAssessmentMethod {
                assessment_id,
                method,
            } => {
                write!(f, "assessment {assessment_id} has invalid method: {method}")
            }
            Self::UnassessedControl { control_id } => {
                write!(
                    f,
                    "control {control_id} has no conforming authenticated assessment"
                )
            }
            Self::CrossOrgIsolationViolation {
                assessment_id,
                expected_org,
                found_org,
            } => {
                write!(f, "assessment {assessment_id} belongs to {found_org}, cannot satisfy {expected_org}")
            }
            Self::MissingStandardAssessment { standard_id } => {
                write!(
                    f,
                    "adopted standard {standard_id} lacks authenticated assessment coverage"
                )
            }
        }
    }
}

impl std::error::Error for StandardsError {}

impl StandardsConfig {
    /// Resolve an OSCAL profile, validating base profile inclusion, non-weakening
    /// constraints, and catalog control completeness.
    pub fn resolve_profile(&self, profile_id: &str) -> Result<ResolvedProfile, StandardsError> {
        let profile = self
            .profiles
            .iter()
            .find(|p| p.profile_id == profile_id)
            .ok_or_else(|| StandardsError::ProfileNotFound(profile_id.to_string()))?;

        // 1. Mandatory base profile must be imported
        if !profile
            .import_catalogs
            .contains(&self.policy.mandatory_base_profile)
        {
            return Err(StandardsError::MandatoryBaseProfileMissing {
                profile_id: profile_id.to_string(),
                expected_base: self.policy.mandatory_base_profile.clone(),
            });
        }

        // 2. Base profile controls cannot be weakened/excluded
        if self.policy.prohibit_weakening_base_profile {
            let base_cat = self
                .catalogs
                .iter()
                .find(|c| c.catalog_id == self.policy.mandatory_base_profile)
                .ok_or_else(|| {
                    StandardsError::CatalogNotFound(self.policy.mandatory_base_profile.clone())
                })?;

            for base_ctrl in &base_cat.controls {
                if profile.excluded_controls.contains(&base_ctrl.id) {
                    return Err(StandardsError::BaseProfileWeakened {
                        profile_id: profile_id.to_string(),
                        control_id: base_ctrl.id.clone(),
                    });
                }
            }
        }

        // 3. Collect controls across imported catalogs
        let mut resolved_controls = Vec::new();
        let mut seen_ids = HashSet::new();

        for cat_id in &profile.import_catalogs {
            let cat = self
                .catalogs
                .iter()
                .find(|c| &c.catalog_id == cat_id)
                .ok_or_else(|| StandardsError::CatalogNotFound(cat_id.clone()))?;

            for ctrl in &cat.controls {
                if profile.excluded_controls.contains(&ctrl.id) {
                    continue;
                }
                if seen_ids.insert(ctrl.id.clone()) {
                    resolved_controls.push(ctrl.clone());
                }
            }
        }

        Ok(ResolvedProfile {
            profile_id: profile.profile_id.clone(),
            title: profile.title.clone(),
            controls: resolved_controls,
        })
    }

    /// Validate system implementation records against a resolved profile,
    /// ensuring every control has a verified local implementation or verified inheritance.
    pub fn validate_system(
        &self,
        system_id: &str,
        profile: &ResolvedProfile,
    ) -> Result<(), StandardsError> {
        let system = self
            .systems
            .iter()
            .find(|s| s.system_id == system_id)
            .ok_or_else(|| StandardsError::SystemNotFound(system_id.to_string()))?;

        // Index local implemented requirements by active components
        let mut local_implemented: HashMap<String, String> = HashMap::new();
        for comp_id in &system.components {
            if let Some(comp) = self.components.iter().find(|c| &c.component_id == comp_id) {
                for req in &comp.implemented_requirements {
                    if req.status == "implemented" {
                        local_implemented.insert(req.control_id.clone(), req.description.clone());
                    }
                }
            }
        }

        // Index inherited controls
        let mut inherited_map: HashMap<String, &InheritedControlRecord> = HashMap::new();
        for inh in &system.inherited_controls {
            // Validate inheritance contract fields
            if inh.provider_component_id.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "provider_component_id",
                });
            }
            if inh.provider_organization.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "provider_organization",
                });
            }
            if inh.provider_scope.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "provider_scope",
                });
            }
            if inh.exact_subject_and_revision.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "exact_subject_and_revision",
                });
            }
            if !inh.evidence_digest.starts_with("sha256:") {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "evidence_digest (must start with sha256:)",
                });
            }
            if inh.validity_conditions.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "validity_conditions",
                });
            }
            if inh.consumer_responsibilities.trim().is_empty() {
                return Err(StandardsError::IncompleteInheritance {
                    control_id: inh.control_id.clone(),
                    field: "consumer_responsibilities",
                });
            }

            inherited_map.insert(inh.control_id.clone(), inh);
        }

        // Verify every control in resolved profile is satisfied
        for ctrl in &profile.controls {
            let is_local = local_implemented.contains_key(&ctrl.id);
            let is_inherited = inherited_map.contains_key(&ctrl.id);

            if !is_local && !is_inherited {
                return Err(StandardsError::UnsatisfiedControl {
                    control_id: ctrl.id.clone(),
                    reason: "no verified local implementation or inherited control record"
                        .to_string(),
                });
            }
        }

        Ok(())
    }

    /// Verify authenticated assessments against registered authorities, adopted standards,
    /// and control coverage.
    pub fn verify_assessments(
        &self,
        profile: &ResolvedProfile,
        authorities: &[AssessmentAuthority],
        standards: &[AdoptedStandard],
        target_org_id: &str,
    ) -> Result<AssessmentVerificationReport, StandardsError> {
        let mut assessed_controls: HashSet<String> = HashSet::new();
        let mut covered_standards: HashSet<String> = HashSet::new();
        let mut authorities_cited = Vec::new();

        for assessment in &self.assessments {
            // Cross-org isolation check
            if self.policy.enforce_cross_org_isolation
                && assessment.organization_id != target_org_id
            {
                return Err(StandardsError::CrossOrgIsolationViolation {
                    assessment_id: assessment.assessment_id.clone(),
                    expected_org: target_org_id.to_string(),
                    found_org: assessment.organization_id.clone(),
                });
            }

            // Assessor authorization & key binding
            let auth = authorities
                .iter()
                .find(|a| a.id == assessment.assessor_authority_id)
                .ok_or_else(|| StandardsError::AssessorNotAuthorized {
                    assessment_id: assessment.assessment_id.clone(),
                    authority_id: assessment.assessor_authority_id.clone(),
                })?;

            if auth.verification_key != assessment.assessor_signature_key {
                return Err(StandardsError::AssessorKeyMismatch {
                    assessment_id: assessment.assessment_id.clone(),
                    authority_id: auth.id.clone(),
                    expected: auth.verification_key.clone(),
                    found: assessment.assessor_signature_key.clone(),
                });
            }

            // Conforming verdict
            if self.policy.require_conforming_verdict && assessment.verdict != "conforming" {
                return Err(StandardsError::NonConformingAssessment {
                    assessment_id: assessment.assessment_id.clone(),
                    verdict: assessment.verdict.clone(),
                });
            }

            // Evidence digest
            if !assessment.evidence_digest.starts_with("sha256:") {
                return Err(StandardsError::InvalidEvidenceDigest {
                    assessment_id: assessment.assessment_id.clone(),
                    digest: assessment.evidence_digest.clone(),
                });
            }

            // Assessment method check
            if !VALID_ASSESSMENT_METHODS.contains(&assessment.assessment_method.as_str()) {
                return Err(StandardsError::InvalidAssessmentMethod {
                    assessment_id: assessment.assessment_id.clone(),
                    method: assessment.assessment_method.clone(),
                });
            }

            // Record covered standards and controls
            covered_standards.insert(assessment.standard_id.clone());
            if !authorities_cited.contains(&assessment.assessor_authority_id) {
                authorities_cited.push(assessment.assessor_authority_id.clone());
            }

            for ctrl_id in &assessment.assessed_controls {
                assessed_controls.insert(ctrl_id.clone());
            }
        }

        // Validate complete standard coverage across adopted standards
        for std in standards {
            if !covered_standards.contains(&std.standard_id) {
                return Err(StandardsError::MissingStandardAssessment {
                    standard_id: std.standard_id.clone(),
                });
            }
        }

        // Validate that every control in resolved profile is assessed
        if self.policy.require_authoritative_coverage {
            for ctrl in &profile.controls {
                if !assessed_controls.contains(&ctrl.id) {
                    return Err(StandardsError::UnassessedControl {
                        control_id: ctrl.id.clone(),
                    });
                }
            }
        }

        Ok(AssessmentVerificationReport {
            total_controls: profile.controls.len(),
            conforming_controls: assessed_controls.len(),
            authorities_cited,
        })
    }

    /// Cross-check standards boundary against owner inputs and policy.
    pub fn check(&self, owner_inputs: &OwnerInputs) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);

        // 1. Verify every adopted standard is cataloged with matching edition
        for adopted in &owner_inputs.standards {
            let cat = self
                .catalogs
                .iter()
                .find(|c| c.standard_id == adopted.standard_id)
                .ok_or_else(|| {
                    bad(format!(
                        "adopted standard {} is missing an OSCAL catalog",
                        adopted.standard_id
                    ))
                })?;

            if cat.edition != adopted.edition {
                return Err(bad(format!(
                    "catalog {} edition mismatch: owner inputs specifies {}, catalog has {}",
                    cat.catalog_id, adopted.edition, cat.edition
                )));
            }
        }

        // 2. Resolve production profile
        let profile = self
            .resolve_profile("PROF-FOUNDRY-PRODUCTION")
            .map_err(|e| bad(format!("resolving production profile: {e}")))?;

        // 3. Validate system security plan
        self.validate_system("SYS-UOR-FOUNDRY", &profile)
            .map_err(|e| bad(format!("validating system implementation: {e}")))?;

        // 4. Verify authenticated assessments
        self.verify_assessments(
            &profile,
            &owner_inputs.assessment_authorities,
            &owner_inputs.standards,
            &owner_inputs.organization.id,
        )
        .map_err(|e| bad(format!("verifying authenticated assessments: {e}")))?;

        Ok(())
    }
}
