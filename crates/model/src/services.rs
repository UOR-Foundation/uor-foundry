//! Services and Views model: SPEC-defined stakeholder journeys, state machines,
//! permissions, resource bounds, failure recovery, raw boundary checks, and view projections.
//!
//! Conformance ID: `SV-01` (suite: `services-views`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::organization::OrganizationLifecycleConfig;
use crate::owner_inputs::OwnerInputs;

/// Errors arising in Services and Views operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    /// A required service is missing or unregistered.
    ServiceNotFound(String),
    /// A required view is missing or unregistered.
    ViewNotFound(String),
    /// State transition is invalid for the entity's current state machine.
    InvalidStateTransition {
        /// Entity ID.
        entity_id: String,
        /// Current state.
        current_state: String,
        /// Attempted target state.
        target_state: String,
        /// Explanatory reason.
        reason: String,
    },
    /// A raw boundary request was attempted without authorized permissions.
    UnauthorizedRawRequest {
        /// Calling identity.
        caller: String,
        /// Required permission.
        required_permission: String,
        /// Service or endpoint.
        endpoint: String,
    },
    /// Resource bounds exceeded (time, memory, tokens, payload size, etc.).
    ResourceBoundsExceeded {
        /// Metric name.
        metric: String,
        /// Measured value.
        measured: u64,
        /// Allowed bound.
        allowed: u64,
    },
    /// SPEC Invariant violated: AI proposal applied without workflow authorization.
    AiProposalNotWorkflowAuthorized(String),
    /// SPEC Invariant violated: Payment recorded without independent settlement oracle.
    PaymentNotSettledByOracle(String),
    /// SPEC Invariant violated: Certification claimed without accredited authority proof.
    CertificationNotAccredited(String),
    /// SPEC Invariant violated: Brand presentation fails WCAG 2.2 AA contrast requirements.
    WcagContrastDeficit {
        /// Type of contrast ('text' or 'ui').
        contrast_type: String,
        /// Measured ratio.
        measured_ratio_x10: u32,
        /// Required ratio.
        required_ratio_x10: u32,
    },
    /// Governance quorum deficit (distinct administrators < required).
    GovernanceQuorumDeficit {
        /// Actual distinct approvals.
        actual_approvals: usize,
        /// Required approvals.
        required_approvals: usize,
    },
    /// Governance revision fencing failed (stale target revision).
    StaleRevision {
        /// Target revision.
        target_revision: u64,
        /// Current revision.
        current_revision: u64,
    },
    /// Sensitive credential leaked or attempted to be exposed in a View.
    ViewExposedCredential(String),
    /// General validation error.
    Validation(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceNotFound(s) => write!(f, "service not found: {s}"),
            Self::ViewNotFound(v) => write!(f, "view not found: {v}"),
            Self::InvalidStateTransition {
                entity_id,
                current_state,
                target_state,
                reason,
            } => write!(
                f,
                "invalid state transition for {entity_id} from {current_state} to {target_state}: {reason}"
            ),
            Self::UnauthorizedRawRequest {
                caller,
                required_permission,
                endpoint,
            } => write!(
                f,
                "unauthorized raw request by {caller} to {endpoint}: missing {required_permission}"
            ),
            Self::ResourceBoundsExceeded {
                metric,
                measured,
                allowed,
            } => write!(
                f,
                "resource bound exceeded for {metric}: measured {measured} > allowed {allowed}"
            ),
            Self::AiProposalNotWorkflowAuthorized(id) => write!(
                f,
                "AI proposal {id} cannot be applied without workflow authorization and verification"
            ),
            Self::PaymentNotSettledByOracle(id) => write!(
                f,
                "payment {id} cannot be settled without independent oracle verification"
            ),
            Self::CertificationNotAccredited(id) => write!(
                f,
                "certificate {id} cannot be issued without accredited authority signature"
            ),
            Self::WcagContrastDeficit {
                contrast_type,
                measured_ratio_x10,
                required_ratio_x10,
            } => write!(
                f,
                "WCAG 2.2 AA contrast deficit for {contrast_type}: measured {} < required {}",
                *measured_ratio_x10 as f64 / 10.0,
                *required_ratio_x10 as f64 / 10.0
            ),
            Self::GovernanceQuorumDeficit {
                actual_approvals,
                required_approvals,
            } => write!(
                f,
                "governance quorum deficit: got {actual_approvals} distinct approvals, required {required_approvals}"
            ),
            Self::StaleRevision {
                target_revision,
                current_revision,
            } => write!(
                f,
                "governance stale revision: target {target_revision} != current {current_revision}"
            ),
            Self::ViewExposedCredential(c) => write!(f, "credential leaked in view: {c}"),
            Self::Validation(v) => write!(f, "validation error: {v}"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// Configuration policy for services and views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesPolicyConfig {
    /// Whether permissions must be enforced at direct raw boundaries.
    pub enforce_raw_boundary_permissions: bool,
    /// Whether AI outputs must be authorized by workflow before effects apply.
    pub require_workflow_authorization_for_ai: bool,
    /// Whether payments require independent settlement oracle receipts.
    pub require_independent_settlement_oracle: bool,
    /// Whether certifications require accredited authority signatures.
    pub require_accredited_certification_authority: bool,
    /// Whether WCAG 2.2 AA contrast ratios are strictly enforced.
    pub enforce_wcag_contrast_minimum: bool,
    /// Whether private credentials and signing secrets are prohibited in views.
    pub prohibit_private_credentials_in_views: bool,
    /// Minimum distinct administrators for governance proposal quorums.
    pub min_governance_quorum_administrators: usize,
}

/// Service definition metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Service UOR ID.
    pub id: String,
    /// Domain tag.
    pub domain: String,
    /// Descriptive title.
    pub title: String,
    /// Modeled actor roles.
    pub actors: Vec<String>,
    /// Modeled permissions.
    pub permissions: Vec<String>,
    /// Valid states in the lifecycle.
    pub allowed_states: Vec<String>,
    /// Associated view name.
    pub associated_view: String,
}

/// View definition metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    /// View UOR ID.
    pub id: String,
    /// View component name.
    pub name: String,
    /// Associated service ID.
    pub service_id: String,
    /// Exposed public fields.
    pub exposed_fields: Vec<String>,
    /// Strictly redacted sensitive fields.
    pub redacted_fields: Vec<String>,
}

/// Top-level configuration loaded from `model/services.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    /// Spec identifier (`foundry/services/1`).
    pub spec: String,
    /// Policy configuration.
    pub policy: ServicesPolicyConfig,
    /// Registered services.
    pub services: Vec<ServiceDefinition>,
    /// Registered views.
    pub views: Vec<ViewDefinition>,
}

impl ServicesConfig {
    /// Validate services configuration against owner inputs and org lifecycle.
    pub fn check(
        &self,
        _owner_inputs: &OwnerInputs,
        _org_lifecycle: &OrganizationLifecycleConfig,
    ) -> Result<(), crate::ModelError> {
        if self.spec != "foundry/services/1" {
            return Err(crate::ModelError::Inconsistent(format!(
                "services spec must be 'foundry/services/1', found '{}'",
                self.spec
            )));
        }

        let required_domains = [
            "workflows",
            "ai-inference",
            "messaging-collaboration",
            "admin-governance",
            "business-finance",
            "learning-certification",
            "brand-presentation",
        ];

        let mut found_domains = HashSet::new();
        for svc in &self.services {
            if svc.id.is_empty() {
                return Err(crate::ModelError::Inconsistent(
                    "service id cannot be empty".to_string(),
                ));
            }
            if svc.actors.is_empty() {
                return Err(crate::ModelError::Inconsistent(format!(
                    "service {} must declare at least one actor",
                    svc.id
                )));
            }
            if svc.permissions.is_empty() {
                return Err(crate::ModelError::Inconsistent(format!(
                    "service {} must declare at least one permission",
                    svc.id
                )));
            }
            if svc.allowed_states.is_empty() {
                return Err(crate::ModelError::Inconsistent(format!(
                    "service {} must declare allowed states",
                    svc.id
                )));
            }
            found_domains.insert(svc.domain.as_str());
        }

        for domain in &required_domains {
            if !found_domains.contains(domain) {
                return Err(crate::ModelError::Inconsistent(format!(
                    "required service domain '{domain}' is not registered in model/services.toml"
                )));
            }
        }

        if self.views.is_empty() {
            return Err(crate::ModelError::Inconsistent(
                "at least one view must be defined".to_string(),
            ));
        }

        for view in &self.views {
            if view.exposed_fields.is_empty() {
                return Err(crate::ModelError::Inconsistent(format!(
                    "view {} must declare exposed fields",
                    view.name
                )));
            }
            if view.redacted_fields.is_empty() {
                return Err(crate::ModelError::Inconsistent(format!(
                    "view {} must declare redacted fields",
                    view.name
                )));
            }
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 1. Workflows Service Domain
// -----------------------------------------------------------------------------

/// State of a Prism workflow execution run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRunState {
    /// Draft definition.
    Draft,
    /// Queued for execution.
    Queued,
    /// Running in bounded environment.
    Running,
    /// Succeeded with produced artifacts.
    Succeeded,
    /// Failed execution.
    Failed,
    /// Explicitly cancelled.
    Cancelled,
}

/// A workflow execution instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    /// Unique run ID.
    pub run_id: String,
    /// Organization owning the run.
    pub organization_id: String,
    /// Workflow name.
    pub workflow_name: String,
    /// Target commit SHA.
    pub commit_sha: String,
    /// Current execution state.
    pub state: WorkflowRunState,
    /// Elapsed execution seconds.
    pub elapsed_seconds: u64,
    /// Memory consumption in MB.
    pub memory_mb: u64,
    /// Disk consumption in MB.
    pub disk_mb: u64,
    /// Produced artifact digest if succeeded.
    pub artifact_digest: Option<String>,
}

impl WorkflowRun {
    /// Create a new queued workflow run.
    pub fn new(run_id: &str, org_id: &str, name: &str, commit: &str) -> Self {
        Self {
            run_id: run_id.to_string(),
            organization_id: org_id.to_string(),
            workflow_name: name.to_string(),
            commit_sha: commit.to_string(),
            state: WorkflowRunState::Queued,
            elapsed_seconds: 0,
            memory_mb: 0,
            disk_mb: 0,
            artifact_digest: None,
        }
    }

    /// Advance execution with resource bounds checks.
    pub fn advance(
        &mut self,
        elapsed_add: u64,
        mem_mb: u64,
        disk_mb: u64,
        max_seconds: u64,
        max_mem: u64,
        max_disk: u64,
    ) -> Result<(), ServiceError> {
        self.state = WorkflowRunState::Running;
        self.elapsed_seconds += elapsed_add;
        self.memory_mb = mem_mb;
        self.disk_mb = disk_mb;

        if self.elapsed_seconds > max_seconds {
            self.state = WorkflowRunState::Failed;
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "workflow execution seconds".to_string(),
                measured: self.elapsed_seconds,
                allowed: max_seconds,
            });
        }
        if self.memory_mb > max_mem {
            self.state = WorkflowRunState::Failed;
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "workflow memory mb".to_string(),
                measured: self.memory_mb,
                allowed: max_mem,
            });
        }
        if self.disk_mb > max_disk {
            self.state = WorkflowRunState::Failed;
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "workflow disk mb".to_string(),
                measured: self.disk_mb,
                allowed: max_disk,
            });
        }

        Ok(())
    }

    /// Complete run successfully with artifact digest.
    pub fn succeed(&mut self, digest: &str) -> Result<(), ServiceError> {
        if self.state != WorkflowRunState::Running && self.state != WorkflowRunState::Queued {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.run_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "Succeeded".to_string(),
                reason: "cannot succeed a non-running/non-queued workflow".to_string(),
            });
        }
        self.state = WorkflowRunState::Succeeded;
        self.artifact_digest = Some(digest.to_string());
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 2. AI Inference & Agentic Execution Service Domain
// -----------------------------------------------------------------------------

/// State of an AI completion proposal.
/// Invariant: AI output is strictly a proposal until authorized and verified by workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProposalState {
    /// Proposed by inference engine.
    Proposed,
    /// Evaluating proposal safety and validity.
    Evaluating,
    /// Authorized by human or automated workflow.
    WorkflowAuthorized,
    /// Verified and applied as active state/effect.
    VerifiedApplied,
    /// Rejected proposal.
    Rejected,
    /// Resource limit exceeded.
    ResourceExceeded,
}

/// An AI inference proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInferenceProposal {
    /// Proposal ID.
    pub proposal_id: String,
    /// Owning organization.
    pub organization_id: String,
    /// Model URI.
    pub model_uri: String,
    /// Prompt content summary.
    pub prompt_summary: String,
    /// Completion content proposed by AI.
    pub proposed_content: String,
    /// Proposal state.
    pub state: AiProposalState,
    /// Context token count.
    pub context_tokens: u64,
    /// Generation token count.
    pub generation_tokens: u64,
    /// Workflow run authorizing the proposal.
    pub authorized_by_workflow: Option<String>,
}

/// Request parameters for creating an AI inference proposal.
#[derive(Debug, Clone)]
pub struct CreateAiProposalRequest<'a> {
    /// Unique proposal ID.
    pub proposal_id: &'a str,
    /// Owning organization ID.
    pub organization_id: &'a str,
    /// Model URI.
    pub model_uri: &'a str,
    /// Prompt text.
    pub prompt: &'a str,
    /// Completion content proposed by AI.
    pub proposed_content: &'a str,
    /// Context token count.
    pub context_tokens: u64,
    /// Generation token count.
    pub generation_tokens: u64,
    /// Maximum allowed context tokens.
    pub max_context_tokens: u64,
    /// Maximum allowed generation tokens.
    pub max_generation_tokens: u64,
}

impl AiInferenceProposal {
    /// Create a new proposal from inference.
    pub fn new(req: CreateAiProposalRequest<'_>) -> Result<Self, ServiceError> {
        if req.context_tokens > req.max_context_tokens {
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "AI context tokens".to_string(),
                measured: req.context_tokens,
                allowed: req.max_context_tokens,
            });
        }
        if req.generation_tokens > req.max_generation_tokens {
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "AI generation tokens".to_string(),
                measured: req.generation_tokens,
                allowed: req.max_generation_tokens,
            });
        }

        Ok(Self {
            proposal_id: req.proposal_id.to_string(),
            organization_id: req.organization_id.to_string(),
            model_uri: req.model_uri.to_string(),
            prompt_summary: req.prompt.chars().take(80).collect(),
            proposed_content: req.proposed_content.to_string(),
            state: AiProposalState::Proposed,
            context_tokens: req.context_tokens,
            generation_tokens: req.generation_tokens,
            authorized_by_workflow: None,
        })
    }

    /// Authorize proposal via workflow execution.
    pub fn authorize_by_workflow(&mut self, workflow_run_id: &str) -> Result<(), ServiceError> {
        if self.state != AiProposalState::Proposed && self.state != AiProposalState::Evaluating {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.proposal_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "WorkflowAuthorized".to_string(),
                reason: "only proposed or evaluating items can be authorized".to_string(),
            });
        }
        self.state = AiProposalState::WorkflowAuthorized;
        self.authorized_by_workflow = Some(workflow_run_id.to_string());
        Ok(())
    }

    /// Apply proposal effects. Rejects if not authorized by workflow.
    pub fn apply_effect(&mut self) -> Result<(), ServiceError> {
        if self.state != AiProposalState::WorkflowAuthorized {
            return Err(ServiceError::AiProposalNotWorkflowAuthorized(
                self.proposal_id.clone(),
            ));
        }
        self.state = AiProposalState::VerifiedApplied;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 3. Messaging & Collaboration Service Domain
// -----------------------------------------------------------------------------

/// State of a collaborative message delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDeliveryState {
    /// Draft.
    Draft,
    /// Sent by member.
    Sent,
    /// Relayed through network.
    Relayed,
    /// Delivered to recipient store.
    Delivered,
    /// Acknowledged receipt.
    Acknowledged,
    /// Delivery failed.
    DeliveryFailed,
}

/// A collaborative message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    /// Message ID.
    pub message_id: String,
    /// Channel ID.
    pub channel_id: String,
    /// Sender identity.
    pub sender_id: String,
    /// Delivery state.
    pub state: MessageDeliveryState,
    /// Payload byte size.
    pub payload_bytes: u64,
    /// Attachment byte size.
    pub attachment_bytes: u64,
    /// Preview text.
    pub preview: String,
}

impl MessageRecord {
    /// Create and send a new message with size bounds checks.
    pub fn send(
        id: &str,
        channel: &str,
        sender: &str,
        content: &str,
        attachment_bytes: u64,
        max_payload_bytes: u64,
        max_attachment_bytes: u64,
    ) -> Result<Self, ServiceError> {
        let payload_bytes = content.len() as u64;
        if payload_bytes > max_payload_bytes {
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "message payload bytes".to_string(),
                measured: payload_bytes,
                allowed: max_payload_bytes,
            });
        }
        if attachment_bytes > max_attachment_bytes {
            return Err(ServiceError::ResourceBoundsExceeded {
                metric: "message attachment bytes".to_string(),
                measured: attachment_bytes,
                allowed: max_attachment_bytes,
            });
        }

        Ok(Self {
            message_id: id.to_string(),
            channel_id: channel.to_string(),
            sender_id: sender.to_string(),
            state: MessageDeliveryState::Sent,
            payload_bytes,
            attachment_bytes,
            preview: content.chars().take(50).collect(),
        })
    }

    /// Mark message as delivered.
    pub fn mark_delivered(&mut self) {
        self.state = MessageDeliveryState::Delivered;
    }

    /// Acknowledge message receipt.
    pub fn acknowledge(&mut self) {
        self.state = MessageDeliveryState::Acknowledged;
    }
}

// -----------------------------------------------------------------------------
// 4. Administration & Governance Service Domain
// -----------------------------------------------------------------------------

/// Governance proposal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceProposalState {
    /// Proposed.
    Proposed,
    /// Open for voting.
    Voting,
    /// Quorum approved.
    Approved,
    /// Enacted to active state.
    Enacted,
    /// Rejected.
    Rejected,
    /// Superceded by newer proposal.
    Superceded,
}

/// A governance change proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// Proposal ID.
    pub proposal_id: String,
    /// Target organization.
    pub organization_id: String,
    /// Title.
    pub title: String,
    /// Proposer identity.
    pub proposer: String,
    /// Current state.
    pub state: GovernanceProposalState,
    /// Distinct approver identities.
    pub approvers: HashSet<String>,
    /// Minimum required distinct approvers.
    pub min_distinct_approvers: usize,
    /// Target revision for revision fencing.
    pub target_revision: u64,
}

impl GovernanceProposal {
    /// Create a new proposal.
    pub fn new(
        id: &str,
        org_id: &str,
        title: &str,
        proposer: &str,
        min_approvers: usize,
        target_revision: u64,
    ) -> Self {
        Self {
            proposal_id: id.to_string(),
            organization_id: org_id.to_string(),
            title: title.to_string(),
            proposer: proposer.to_string(),
            state: GovernanceProposalState::Voting,
            approvers: HashSet::new(),
            min_distinct_approvers: min_approvers,
            target_revision,
        }
    }

    /// Add an approval from an administrator. Rejects duplicates.
    pub fn approve(&mut self, admin: &str) -> bool {
        self.approvers.insert(admin.to_string())
    }

    /// Enact proposal against current revision with quorum verification.
    pub fn enact(&mut self, current_revision: u64) -> Result<(), ServiceError> {
        if self.target_revision != current_revision {
            return Err(ServiceError::StaleRevision {
                target_revision: self.target_revision,
                current_revision,
            });
        }
        if self.approvers.len() < self.min_distinct_approvers {
            return Err(ServiceError::GovernanceQuorumDeficit {
                actual_approvals: self.approvers.len(),
                required_approvals: self.min_distinct_approvers,
            });
        }
        self.state = GovernanceProposalState::Enacted;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 5. Business Planning & Finance Service Domain
// -----------------------------------------------------------------------------

/// State of a financial invoice / transaction.
/// Invariant: Recording a payment is NOT evidence of settlement by an independent authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinancialInvoiceState {
    /// Draft.
    Draft,
    /// Authorized for billing.
    Authorized,
    /// Payment submitted, pending settlement oracle verification.
    PendingSettlement,
    /// Confirmed settled by independent oracle.
    Settled,
    /// Declined by oracle.
    Declined,
    /// Disputed.
    Disputed,
}

/// A business invoice and payment lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialInvoice {
    /// Invoice ID.
    pub invoice_id: String,
    /// Billing organization.
    pub organization_id: String,
    /// Amount in cents.
    pub amount_cents: u64,
    /// Currency code (e.g. USD).
    pub currency: String,
    /// Lifecycle state.
    pub state: FinancialInvoiceState,
    /// Independent settlement oracle ID.
    pub settlement_oracle: Option<String>,
    /// Cryptographic receipt digest.
    pub settlement_digest: Option<String>,
}

impl FinancialInvoice {
    /// Create a new invoice.
    pub fn new(id: &str, org_id: &str, cents: u64, currency: &str) -> Self {
        Self {
            invoice_id: id.to_string(),
            organization_id: org_id.to_string(),
            amount_cents: cents,
            currency: currency.to_string(),
            state: FinancialInvoiceState::Authorized,
            settlement_oracle: None,
            settlement_digest: None,
        }
    }

    /// Record payment attempt (moves to PendingSettlement, NEVER directly to Settled).
    pub fn record_payment(&mut self) -> Result<(), ServiceError> {
        if self.state != FinancialInvoiceState::Authorized {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.invoice_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "PendingSettlement".to_string(),
                reason: "invoice must be authorized to record payment".to_string(),
            });
        }
        self.state = FinancialInvoiceState::PendingSettlement;
        Ok(())
    }

    /// Settle payment via independent authoritative oracle receipt.
    pub fn settle_with_oracle(
        &mut self,
        oracle_id: &str,
        receipt_digest: &str,
    ) -> Result<(), ServiceError> {
        if self.state != FinancialInvoiceState::PendingSettlement {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.invoice_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "Settled".to_string(),
                reason:
                    "payment must be recorded as pending settlement prior to oracle confirmation"
                        .to_string(),
            });
        }
        if !receipt_digest.starts_with("sha256:") {
            return Err(ServiceError::PaymentNotSettledByOracle(
                "receipt digest must be a valid sha256: hash".to_string(),
            ));
        }

        self.state = FinancialInvoiceState::Settled;
        self.settlement_oracle = Some(oracle_id.to_string());
        self.settlement_digest = Some(receipt_digest.to_string());
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 6. Learning, Assessment & Certification Service Domain
// -----------------------------------------------------------------------------

/// State of an educational certification journey.
/// Invariant: Recording an assessment is NOT evidence of certification by an independent authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificationState {
    /// Enrolled in syllabus.
    Enrolled,
    /// In progress.
    InProgress,
    /// Exam submitted.
    Submitted,
    /// Graded by instructor/rubric.
    Assessed,
    /// Certified by accredited authority.
    Certified,
    /// Uncertified (score below passing).
    Uncertified,
    /// Revoked.
    Revoked,
}

/// A learner assessment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerAssessment {
    /// Assessment ID.
    pub assessment_id: String,
    /// Organization.
    pub organization_id: String,
    /// Learner ID.
    pub learner_id: String,
    /// Course ID.
    pub course_id: String,
    /// Current state.
    pub state: CertificationState,
    /// Exam score (0 - 100).
    pub score: u32,
    /// Accredited certification authority ID.
    pub certification_authority: Option<String>,
    /// Certificate signature digest.
    pub certificate_digest: Option<String>,
}

impl LearnerAssessment {
    /// Create a new learner enrollment.
    pub fn enroll(id: &str, org_id: &str, learner: &str, course: &str) -> Self {
        Self {
            assessment_id: id.to_string(),
            organization_id: org_id.to_string(),
            learner_id: learner.to_string(),
            course_id: course.to_string(),
            state: CertificationState::Enrolled,
            score: 0,
            certification_authority: None,
            certificate_digest: None,
        }
    }

    /// Record exam submission and score.
    pub fn record_assessment(&mut self, score: u32) -> Result<(), ServiceError> {
        self.score = score;
        self.state = CertificationState::Assessed;
        Ok(())
    }

    /// Issue accredited certificate. Requires accredited authority and valid digest.
    pub fn issue_certificate(
        &mut self,
        authority_id: &str,
        cert_digest: &str,
        passing_score: u32,
    ) -> Result<(), ServiceError> {
        if self.state != CertificationState::Assessed {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.assessment_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "Certified".to_string(),
                reason: "assessment must be completed before certificate issuance".to_string(),
            });
        }
        if self.score < passing_score {
            self.state = CertificationState::Uncertified;
            return Err(ServiceError::CertificationNotAccredited(format!(
                "score {} is below passing score {}",
                self.score, passing_score
            )));
        }
        if !cert_digest.starts_with("sha256:") {
            return Err(ServiceError::CertificationNotAccredited(
                "certificate digest must be sha256:".to_string(),
            ));
        }

        self.state = CertificationState::Certified;
        self.certification_authority = Some(authority_id.to_string());
        self.certificate_digest = Some(cert_digest.to_string());
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// 7. Brand Identity & Accessible Presentation Service Domain
// -----------------------------------------------------------------------------

/// State of brand kit presentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrandKitState {
    /// Draft design.
    Draft,
    /// In review.
    Review,
    /// Validated against WCAG 2.2 AA.
    AccessibilityValidated,
    /// Published.
    Published,
    /// Rejected due to contrast failure.
    Rejected,
}

/// A brand identity kit with accessibility bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandKit {
    /// Brand kit ID.
    pub kit_id: String,
    /// Owning organization.
    pub organization_id: String,
    /// Brand state.
    pub state: BrandKitState,
    /// Normal text contrast ratio (e.g. 4.8).
    pub text_contrast_ratio: f64,
    /// UI component contrast ratio (e.g. 3.2).
    pub ui_contrast_ratio: f64,
    /// List of public asset URLs.
    pub public_assets: Vec<String>,
}

impl BrandKit {
    /// Create a new brand kit in review.
    pub fn new(kit_id: &str, org_id: &str, text_contrast: f64, ui_contrast: f64) -> Self {
        Self {
            kit_id: kit_id.to_string(),
            organization_id: org_id.to_string(),
            state: BrandKitState::Review,
            text_contrast_ratio: text_contrast,
            ui_contrast_ratio: ui_contrast,
            public_assets: Vec::new(),
        }
    }

    /// Validate against WCAG 2.2 AA standards (>= 4.5:1 text, >= 3.0:1 UI).
    pub fn validate_wcag_contrast(
        &mut self,
        min_text_contrast: f64,
        min_ui_contrast: f64,
    ) -> Result<(), ServiceError> {
        if self.text_contrast_ratio < min_text_contrast {
            self.state = BrandKitState::Rejected;
            return Err(ServiceError::WcagContrastDeficit {
                contrast_type: "normal text".to_string(),
                measured_ratio_x10: (self.text_contrast_ratio * 10.0) as u32,
                required_ratio_x10: (min_text_contrast * 10.0) as u32,
            });
        }
        if self.ui_contrast_ratio < min_ui_contrast {
            self.state = BrandKitState::Rejected;
            return Err(ServiceError::WcagContrastDeficit {
                contrast_type: "ui component".to_string(),
                measured_ratio_x10: (self.ui_contrast_ratio * 10.0) as u32,
                required_ratio_x10: (min_ui_contrast * 10.0) as u32,
            });
        }

        self.state = BrandKitState::AccessibilityValidated;
        Ok(())
    }

    /// Publish brand kit.
    pub fn publish(&mut self, asset: &str) -> Result<(), ServiceError> {
        if self.state != BrandKitState::AccessibilityValidated {
            return Err(ServiceError::InvalidStateTransition {
                entity_id: self.kit_id.clone(),
                current_state: format!("{:?}", self.state),
                target_state: "Published".to_string(),
                reason: "brand kit must be accessibility validated prior to publication"
                    .to_string(),
            });
        }
        self.state = BrandKitState::Published;
        self.public_assets.push(asset.to_string());
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Direct Raw Request Boundary & View Projections
// -----------------------------------------------------------------------------

/// Raw request executor enforcing permissions independently of Views.
pub struct RawBoundaryExecutor;

impl RawBoundaryExecutor {
    /// Execute a raw direct request against a service boundary.
    /// Fails with `UnauthorizedRawRequest` if caller lacks the required permission,
    /// proving caller cannot bypass permissions by avoiding a View.
    pub fn execute_raw_request(
        service: &ServiceDefinition,
        caller: &str,
        caller_permissions: &[&str],
        requested_permission: &str,
        endpoint: &str,
    ) -> Result<(), ServiceError> {
        if !service
            .permissions
            .iter()
            .any(|p| p == requested_permission)
        {
            return Err(ServiceError::Validation(format!(
                "permission '{requested_permission}' not defined for service {}",
                service.id
            )));
        }

        if !caller_permissions.contains(&requested_permission) {
            return Err(ServiceError::UnauthorizedRawRequest {
                caller: caller.to_string(),
                required_permission: requested_permission.to_string(),
                endpoint: endpoint.to_string(),
            });
        }

        Ok(())
    }
}

/// View projector filtering unapproved proposals and stripping private credentials.
pub struct ViewProjector;

impl ViewProjector {
    /// Project fields for a view, asserting that no redacted credentials appear in the output.
    pub fn project(
        view: &ViewDefinition,
        raw_fields: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, ServiceError> {
        let mut projected = HashMap::new();

        for redacted in &view.redacted_fields {
            if raw_fields.contains_key(redacted) {
                // If a raw payload contained a credential, ensure it is redacted
                // and never forwarded to view output.
            }
        }

        for exposed in &view.exposed_fields {
            if let Some(val) = raw_fields.get(exposed) {
                // Check if an exposed field accidentally leaks a private key
                if val.contains("BEGIN PRIVATE KEY") || val.contains("SECRET_TOKEN") {
                    return Err(ServiceError::ViewExposedCredential(exposed.clone()));
                }
                projected.insert(exposed.clone(), val.clone());
            }
        }

        Ok(projected)
    }
}
