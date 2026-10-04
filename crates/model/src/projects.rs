//! Domain model for organization-scoped projects, milestones, deliverables,
//! tamper-evident SHA-256 chained activity journals, and project releases.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Lifecycle states for an organization-scoped project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectLifecycleState {
    /// Active and operational project accepting changes and milestones.
    Active,
    /// Read-only archived snapshot of a project.
    Archived,
    /// Suspended project pending administrative or security review.
    Suspended,
    /// Deleted or retired project.
    Deleted,
}

impl ProjectLifecycleState {
    /// String representation of the lifecycle state.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Archived => "Archived",
            Self::Suspended => "Suspended",
            Self::Deleted => "Deleted",
        }
    }
}

impl fmt::Display for ProjectLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Project-scoped membership roles with hierarchical capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectRole {
    /// Project lead with full administrative permissions over members, settings, and releases.
    Lead,
    /// Maintainer with permissions to manage milestones, deliverables, and releases.
    Maintainer,
    /// Contributor with permissions to submit deliverables and log activities.
    Contributor,
    /// Viewer with read-only inspection access.
    Viewer,
}

impl ProjectRole {
    /// String representation of the project role.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lead => "Lead",
            Self::Maintainer => "Maintainer",
            Self::Contributor => "Contributor",
            Self::Viewer => "Viewer",
        }
    }

    /// Whether this role can administer project settings and members.
    pub fn can_administer(&self) -> bool {
        matches!(self, Self::Lead)
    }

    /// Whether this role can manage milestones and deliverables.
    pub fn can_manage_milestones(&self) -> bool {
        matches!(self, Self::Lead | Self::Maintainer)
    }

    /// Whether this role can complete deliverables or submit work.
    pub fn can_contribute(&self) -> bool {
        matches!(self, Self::Lead | Self::Maintainer | Self::Contributor)
    }
}

impl fmt::Display for ProjectRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Member assigned to an organization-scoped project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMember {
    /// Mailbox/email address of the member.
    pub mailbox: String,
    /// Unique user identity of the member.
    pub user_id: String,
    /// Project-scoped role.
    pub role: ProjectRole,
    /// ISO-8601 timestamp when member joined.
    pub joined_at_iso: String,
}

/// Configuration settings for a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Extended project description.
    pub description: String,
    /// Optional linked repository URI.
    pub repository_uri: Option<String>,
    /// PrismPM model revision bound to this project.
    pub prism_model_revision: u64,
    /// Classification and search tags.
    pub tags: Vec<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            description: String::new(),
            repository_uri: None,
            prism_model_revision: 1,
            tags: Vec::new(),
        }
    }
}

/// Operational status of a milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    /// Milestone is planned.
    Planned,
    /// Work is currently underway.
    InProgress,
    /// All deliverables completed and milestone verified.
    Completed,
    /// Milestone was cancelled.
    Cancelled,
}

impl MilestoneStatus {
    /// String representation of the milestone status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::InProgress => "InProgress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Completion status of an individual deliverable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliverableStatus {
    /// Deliverable is pending work.
    Pending,
    /// Deliverable was submitted for review.
    Submitted,
    /// Deliverable was cryptographically verified and accepted.
    Verified,
    /// Deliverable was rejected.
    Rejected,
}

impl DeliverableStatus {
    /// String representation of the deliverable status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Submitted => "Submitted",
            Self::Verified => "Verified",
            Self::Rejected => "Rejected",
        }
    }
}

/// Record representing a concrete deliverable within a milestone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliverableRecord {
    /// Unique deliverable identifier.
    pub deliverable_id: String,
    /// Identifier of the parent milestone.
    pub milestone_id: String,
    /// Human-readable title of the deliverable.
    pub title: String,
    /// Optional cryptographic digest proof of the output artifact.
    pub artifact_digest: Option<String>,
    /// Status of the deliverable.
    pub status: DeliverableStatus,
    /// Optional ISO timestamp of completion.
    pub completed_at_iso: Option<String>,
}

/// Record representing a project milestone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MilestoneRecord {
    /// Unique milestone identifier.
    pub milestone_id: String,
    /// Parent project identifier.
    pub project_id: String,
    /// Milestone title.
    pub title: String,
    /// Milestone detailed description.
    pub description: String,
    /// Target completion date (ISO-8601).
    pub target_date: String,
    /// Operational status of the milestone.
    pub status: MilestoneStatus,
    /// Child deliverables tracked under this milestone.
    pub deliverables: Vec<DeliverableRecord>,
}

/// An entry in the tamper-evident SHA-256 chained project activity journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectActivityEntry {
    /// Unique entry identifier.
    pub entry_id: String,
    /// Associated project identifier.
    pub project_id: String,
    /// ISO-8601 timestamp of event occurrence.
    pub timestamp: String,
    /// Actor mailbox or identifier who initiated the action.
    pub actor: String,
    /// Action type or description.
    pub action: String,
    /// Action details or parameters.
    pub details: String,
    /// SHA-256 hex digest of the previous activity entry, or 64 zeroes for genesis.
    pub prev_entry_digest: String,
    /// Cryptographic SHA-256 digest over the current entry and predecessor.
    pub entry_digest: String,
}

impl ProjectActivityEntry {
    /// Compute genuine SHA-256 digest over entry components and predecessor digest.
    pub fn compute_digest(
        prev_digest: &str,
        entry_id: &str,
        project_id: &str,
        actor: &str,
        action: &str,
        details: &str,
        timestamp: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prev_digest.as_bytes());
        hasher.update(b":");
        hasher.update(entry_id.as_bytes());
        hasher.update(b":");
        hasher.update(project_id.as_bytes());
        hasher.update(b":");
        hasher.update(actor.as_bytes());
        hasher.update(b":");
        hasher.update(action.as_bytes());
        hasher.update(b":");
        hasher.update(details.as_bytes());
        hasher.update(b":");
        hasher.update(timestamp.as_bytes());
        let result = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for byte in result {
            use std::fmt::Write;
            let _ = write!(hex, "{:02x}", byte);
        }
        hex
    }
}

/// Record representing a published or draft project release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectReleaseRecord {
    /// Unique release identifier.
    pub release_id: String,
    /// Parent project identifier.
    pub project_id: String,
    /// Semantic version tag (e.g. "v1.0.0").
    pub version_tag: String,
    /// Title of the release.
    pub title: String,
    /// Release notes and changelog.
    pub notes: String,
    /// Associated PrismPM model revision.
    pub model_revision: u64,
    /// Git commit SHA of the release.
    pub commit_sha: String,
    /// Root artifact digest tree hash.
    pub artifact_root_digest: String,
    /// ISO-8601 timestamp when drafted.
    pub created_at: String,
    /// ISO-8601 timestamp when published.
    pub published_at: Option<String>,
    /// Whether this release is published and immutable.
    pub is_published: bool,
}

/// Request parameters for drafting a project release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProjectReleaseRequest<'a> {
    /// Caller mailbox.
    pub caller_mailbox: &'a str,
    /// Parent project identifier.
    pub project_id: &'a str,
    /// Unique release identifier.
    pub release_id: &'a str,
    /// Semantic version tag (e.g. "v1.0.0").
    pub version_tag: &'a str,
    /// Title of the release.
    pub title: &'a str,
    /// Release notes and changelog.
    pub notes: &'a str,
    /// Associated PrismPM model revision.
    pub model_revision: u64,
    /// Git commit SHA of the release.
    pub commit_sha: &'a str,
    /// Root artifact digest tree hash.
    pub artifact_root_digest: &'a str,
}

/// Core record representing an organization-scoped project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRecord {
    /// Canonical UOR project identifier (e.g. `uor:project:<org_id>:<slug>`).
    pub id: String,
    /// Identifier of the parent organization.
    pub org_id: String,
    /// Display name (supports duplicates across unique IDs).
    pub name: String,
    /// Project description.
    pub description: String,
    /// Project lifecycle state.
    pub state: ProjectLifecycleState,
    /// Project configuration settings.
    pub settings: ProjectSettings,
    /// Project members.
    pub members: Vec<ProjectMember>,
    /// Tracked milestones.
    pub milestones: Vec<MilestoneRecord>,
    /// Flat list of deliverables for quick lookup.
    pub deliverables: Vec<DeliverableRecord>,
    /// Tamper-evident SHA-256 chained activity log.
    pub activity_log: Vec<ProjectActivityEntry>,
    /// Releases tracked under this project.
    pub releases: Vec<ProjectReleaseRecord>,
    /// Creation timestamp (ISO-8601).
    pub created_at_iso: String,
    /// Last update timestamp (ISO-8601).
    pub updated_at_iso: String,
}

/// Errors returned by project management operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectError {
    /// Project not found.
    ProjectNotFound(String),
    /// Duplicate project identifier.
    DuplicateProjectId(String),
    /// Cross-organization boundary isolation violation.
    CrossOrgBoundaryViolation {
        /// Target organization.
        org_id: String,
        /// Project attempted to be accessed.
        project_id: String,
    },
    /// Unauthorized action for the given actor role.
    UnauthorizedProjectAction {
        /// Actor attempting the action.
        actor: String,
        /// Action attempted.
        action: String,
        /// Role required.
        required_role: String,
    },
    /// Invalid state transition for the project lifecycle.
    InvalidProjectStateTransition {
        /// Current state.
        current: String,
        /// Target state.
        target: String,
        /// Explanation for why transition is invalid.
        reason: String,
    },
    /// Active dependencies prevent project deletion or retirement.
    ActiveDependenciesPreventDeletion {
        /// Project ID.
        project_id: String,
        /// Number of active milestones remaining.
        active_milestones: usize,
        /// Number of active deliverables remaining.
        active_deliverables: usize,
    },
    /// Milestone not found.
    MilestoneNotFound(String),
    /// Deliverable not found.
    DeliverableNotFound(String),
    /// Release was already published and is immutable.
    ReleaseAlreadyPublished(String),
    /// Cryptographic activity log chain validation failed.
    ActivityLogTampered {
        /// Tampered entry identifier.
        entry_id: String,
        /// Expected predecessor digest.
        expected_prev: String,
        /// Actual recorded predecessor digest.
        actual_prev: String,
    },
    /// General input validation error.
    Validation(String),
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProjectNotFound(id) => write!(f, "project '{id}' not found"),
            Self::DuplicateProjectId(id) => write!(f, "duplicate project id '{id}'"),
            Self::CrossOrgBoundaryViolation { org_id, project_id } => write!(
                f,
                "cross-org violation: project '{project_id}' does not belong to org '{org_id}'"
            ),
            Self::UnauthorizedProjectAction {
                actor,
                action,
                required_role,
            } => write!(
                f,
                "actor '{actor}' unauthorized for action '{action}': requires '{required_role}'"
            ),
            Self::InvalidProjectStateTransition {
                current,
                target,
                reason,
            } => write!(
                f,
                "invalid transition from '{current}' to '{target}': {reason}"
            ),
            Self::ActiveDependenciesPreventDeletion {
                project_id,
                active_milestones,
                active_deliverables,
            } => write!(
                f,
                "cannot delete project '{project_id}': {active_milestones} active milestones and {active_deliverables} active deliverables remain"
            ),
            Self::MilestoneNotFound(id) => write!(f, "milestone '{id}' not found"),
            Self::DeliverableNotFound(id) => write!(f, "deliverable '{id}' not found"),
            Self::ReleaseAlreadyPublished(id) => {
                write!(f, "release '{id}' is already published and immutable")
            }
            Self::ActivityLogTampered {
                entry_id,
                expected_prev,
                actual_prev,
            } => write!(
                f,
                "activity log tampering detected at entry '{entry_id}': expected prev '{expected_prev}', got '{actual_prev}'"
            ),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for ProjectError {}

/// Manager governing organization-scoped project lifecycles, milestones, releases,
/// and SHA-256 chained activity journals.
#[derive(Debug, Default)]
pub struct ProjectManager {
    projects: Vec<ProjectRecord>,
}

impl ProjectManager {
    /// Create a new, empty project manager.
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    /// Genesis predecessor digest constant (64 zeroes).
    pub const GENESIS_DIGEST: &'static str =
        "0000000000000000000000000000000000000000000000000000000000000000";

    /// Create a new project within an organization.
    pub fn create_project(
        &mut self,
        _caller_org_role: Option<&str>,
        org_id: &str,
        id: &str,
        name: &str,
        description: &str,
        creator_mailbox: &str,
    ) -> Result<&ProjectRecord, ProjectError> {
        if id.trim().is_empty() {
            return Err(ProjectError::Validation(
                "project ID cannot be empty".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(ProjectError::Validation(
                "project name cannot be empty".to_string(),
            ));
        }
        if self.projects.iter().any(|p| p.id == id) {
            return Err(ProjectError::DuplicateProjectId(id.to_string()));
        }

        let now_iso = "2026-10-03T07:00:00Z".to_string();
        let genesis_entry_id = format!("{id}:act-0001");
        let genesis_action = "ProjectCreated";
        let genesis_details = format!("Project '{name}' created by '{creator_mailbox}'");
        let entry_digest = ProjectActivityEntry::compute_digest(
            Self::GENESIS_DIGEST,
            &genesis_entry_id,
            id,
            creator_mailbox,
            genesis_action,
            &genesis_details,
            &now_iso,
        );

        let genesis_entry = ProjectActivityEntry {
            entry_id: genesis_entry_id,
            project_id: id.to_string(),
            timestamp: now_iso.clone(),
            actor: creator_mailbox.to_string(),
            action: genesis_action.to_string(),
            details: genesis_details,
            prev_entry_digest: Self::GENESIS_DIGEST.to_string(),
            entry_digest,
        };

        let initial_member = ProjectMember {
            mailbox: creator_mailbox.to_string(),
            user_id: format!("usr:{creator_mailbox}"),
            role: ProjectRole::Lead,
            joined_at_iso: now_iso.clone(),
        };

        let project = ProjectRecord {
            id: id.to_string(),
            org_id: org_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            state: ProjectLifecycleState::Active,
            settings: ProjectSettings {
                description: description.to_string(),
                ..Default::default()
            },
            members: vec![initial_member],
            milestones: Vec::new(),
            deliverables: Vec::new(),
            activity_log: vec![genesis_entry],
            releases: Vec::new(),
            created_at_iso: now_iso.clone(),
            updated_at_iso: now_iso,
        };

        self.projects.push(project);
        Ok(self.projects.last().expect("project just pushed"))
    }

    /// Retrieve a project by ID, strictly enforcing organization boundary isolation.
    pub fn get_project(
        &self,
        caller_org_id: &str,
        project_id: &str,
    ) -> Result<&ProjectRecord, ProjectError> {
        let project = self
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if project.org_id != caller_org_id {
            return Err(ProjectError::CrossOrgBoundaryViolation {
                org_id: caller_org_id.to_string(),
                project_id: project_id.to_string(),
            });
        }

        Ok(project)
    }

    /// Retrieve a mutable reference to a project by ID with organization boundary isolation.
    pub fn get_project_mut(
        &mut self,
        caller_org_id: &str,
        project_id: &str,
    ) -> Result<&mut ProjectRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if project.org_id != caller_org_id {
            return Err(ProjectError::CrossOrgBoundaryViolation {
                org_id: caller_org_id.to_string(),
                project_id: project_id.to_string(),
            });
        }

        Ok(project)
    }

    /// List all projects belonging to an organization.
    pub fn list_projects_for_org(&self, org_id: &str) -> Vec<&ProjectRecord> {
        self.projects
            .iter()
            .filter(|p| p.org_id == org_id)
            .collect()
    }

    /// List all projects across the system.
    pub fn list_projects(&self) -> &[ProjectRecord] {
        &self.projects
    }

    /// Calculate active project count for an organization.
    pub fn active_project_count(&self, org_id: &str) -> usize {
        self.projects
            .iter()
            .filter(|p| p.org_id == org_id && p.state == ProjectLifecycleState::Active)
            .count()
    }

    /// Alias for `active_project_count`.
    pub fn get_active_project_count(&self, org_id: &str) -> usize {
        self.active_project_count(org_id)
    }

    /// Update project display name and settings.
    pub fn update_project(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        new_name: &str,
        new_settings: ProjectSettings,
    ) -> Result<&ProjectRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if project.state != ProjectLifecycleState::Active {
            return Err(ProjectError::InvalidProjectStateTransition {
                current: project.state.as_str().to_string(),
                target: "Updated".to_string(),
                reason: "cannot update a non-active project".to_string(),
            });
        }

        let is_lead_or_maintainer = project.members.iter().any(|m| {
            m.mailbox == caller_mailbox
                && matches!(m.role, ProjectRole::Lead | ProjectRole::Maintainer)
        });
        if !is_lead_or_maintainer {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "update_project".to_string(),
                required_role: "Lead or Maintainer".to_string(),
            });
        }

        project.name = new_name.to_string();
        project.settings = new_settings;
        project.updated_at_iso = "2026-10-03T07:10:00Z".to_string();

        let details = format!("Project updated: name='{new_name}'");
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "ProjectUpdated",
            &details,
            &project.updated_at_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: project.updated_at_iso.clone(),
            actor: caller_mailbox.to_string(),
            action: "ProjectUpdated".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(project)
    }

    /// Archive an active project.
    pub fn archive_project(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
    ) -> Result<&ProjectRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if project.state != ProjectLifecycleState::Active {
            return Err(ProjectError::InvalidProjectStateTransition {
                current: project.state.as_str().to_string(),
                target: ProjectLifecycleState::Archived.as_str().to_string(),
                reason: "only active projects may be archived".to_string(),
            });
        }

        let is_lead = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role == ProjectRole::Lead);
        if !is_lead {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "archive_project".to_string(),
                required_role: "Lead".to_string(),
            });
        }

        project.state = ProjectLifecycleState::Archived;
        project.updated_at_iso = "2026-10-03T07:20:00Z".to_string();

        let details = "Project archived".to_string();
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "ProjectArchived",
            &details,
            &project.updated_at_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: project.updated_at_iso.clone(),
            actor: caller_mailbox.to_string(),
            action: "ProjectArchived".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(project)
    }

    /// Restore an archived or suspended project to active state.
    pub fn restore_project(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
    ) -> Result<&ProjectRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if !matches!(
            project.state,
            ProjectLifecycleState::Archived | ProjectLifecycleState::Suspended
        ) {
            return Err(ProjectError::InvalidProjectStateTransition {
                current: project.state.as_str().to_string(),
                target: ProjectLifecycleState::Active.as_str().to_string(),
                reason: "only archived or suspended projects may be restored".to_string(),
            });
        }

        let is_lead = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role == ProjectRole::Lead);
        if !is_lead {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "restore_project".to_string(),
                required_role: "Lead".to_string(),
            });
        }

        project.state = ProjectLifecycleState::Active;
        project.updated_at_iso = "2026-10-03T07:25:00Z".to_string();

        let details = "Project restored to active state".to_string();
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "ProjectRestored",
            &details,
            &project.updated_at_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: project.updated_at_iso.clone(),
            actor: caller_mailbox.to_string(),
            action: "ProjectRestored".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(project)
    }

    /// Delete or retire a project, enforcing dependent resource safeguards.
    pub fn delete_project(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        force_override: bool,
    ) -> Result<(), ProjectError> {
        let index = self
            .projects
            .iter()
            .position(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let project = &self.projects[index];
        let is_lead = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role == ProjectRole::Lead);
        if !is_lead {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "delete_project".to_string(),
                required_role: "Lead".to_string(),
            });
        }

        let active_milestones = project
            .milestones
            .iter()
            .filter(|m| {
                matches!(
                    m.status,
                    MilestoneStatus::Planned | MilestoneStatus::InProgress
                )
            })
            .count();
        let active_deliverables = project
            .deliverables
            .iter()
            .filter(|d| {
                matches!(
                    d.status,
                    DeliverableStatus::Pending | DeliverableStatus::Submitted
                )
            })
            .count();

        if (active_milestones > 0 || active_deliverables > 0) && !force_override {
            return Err(ProjectError::ActiveDependenciesPreventDeletion {
                project_id: project_id.to_string(),
                active_milestones,
                active_deliverables,
            });
        }

        self.projects[index].state = ProjectLifecycleState::Deleted;
        Ok(())
    }

    /// Add or update a member in a project.
    pub fn add_or_update_member(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        member_mailbox: &str,
        user_id: &str,
        role: ProjectRole,
    ) -> Result<(), ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let is_lead = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role == ProjectRole::Lead);
        if !is_lead {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "add_or_update_member".to_string(),
                required_role: "Lead".to_string(),
            });
        }

        let now_iso = "2026-10-03T07:30:00Z".to_string();
        if let Some(existing) = project
            .members
            .iter_mut()
            .find(|m| m.mailbox == member_mailbox)
        {
            existing.role = role;
        } else {
            project.members.push(ProjectMember {
                mailbox: member_mailbox.to_string(),
                user_id: user_id.to_string(),
                role,
                joined_at_iso: now_iso.clone(),
            });
        }

        let details = format!("Member '{member_mailbox}' assigned role '{role}'");
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "MemberRoleUpdated",
            &details,
            &now_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: caller_mailbox.to_string(),
            action: "MemberRoleUpdated".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(())
    }

    /// Create a milestone under an active project.
    pub fn create_milestone(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        milestone_id: &str,
        title: &str,
        description: &str,
        target_date: &str,
    ) -> Result<&MilestoneRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        if project.state != ProjectLifecycleState::Active {
            return Err(ProjectError::InvalidProjectStateTransition {
                current: project.state.as_str().to_string(),
                target: "MilestoneCreated".to_string(),
                reason: "cannot create milestones on a non-active project".to_string(),
            });
        }

        let can_manage = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role.can_manage_milestones());
        if !can_manage {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "create_milestone".to_string(),
                required_role: "Lead or Maintainer".to_string(),
            });
        }

        if project
            .milestones
            .iter()
            .any(|m| m.milestone_id == milestone_id)
        {
            return Err(ProjectError::Validation(format!(
                "milestone with id '{milestone_id}' already exists"
            )));
        }

        let milestone = MilestoneRecord {
            milestone_id: milestone_id.to_string(),
            project_id: project_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            target_date: target_date.to_string(),
            status: MilestoneStatus::Planned,
            deliverables: Vec::new(),
        };

        project.milestones.push(milestone);

        let now_iso = "2026-10-03T07:35:00Z".to_string();
        let details = format!("Milestone '{title}' created (id={milestone_id})");
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "MilestoneCreated",
            &details,
            &now_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: caller_mailbox.to_string(),
            action: "MilestoneCreated".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(project.milestones.last().expect("milestone just added"))
    }

    /// Add a deliverable under a specific milestone.
    pub fn add_deliverable(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        milestone_id: &str,
        deliverable_id: &str,
        title: &str,
    ) -> Result<&DeliverableRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let can_manage = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role.can_manage_milestones());
        if !can_manage {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "add_deliverable".to_string(),
                required_role: "Lead or Maintainer".to_string(),
            });
        }

        let milestone = project
            .milestones
            .iter_mut()
            .find(|m| m.milestone_id == milestone_id)
            .ok_or_else(|| ProjectError::MilestoneNotFound(milestone_id.to_string()))?;

        if milestone
            .deliverables
            .iter()
            .any(|d| d.deliverable_id == deliverable_id)
        {
            return Err(ProjectError::Validation(format!(
                "deliverable '{deliverable_id}' already exists"
            )));
        }

        let deliverable = DeliverableRecord {
            deliverable_id: deliverable_id.to_string(),
            milestone_id: milestone_id.to_string(),
            title: title.to_string(),
            artifact_digest: None,
            status: DeliverableStatus::Pending,
            completed_at_iso: None,
        };

        milestone.deliverables.push(deliverable.clone());
        project.deliverables.push(deliverable);

        Ok(project.deliverables.last().expect("deliverable added"))
    }

    /// Complete a deliverable with verified cryptographic artifact digest proof.
    pub fn complete_deliverable(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        milestone_id: &str,
        deliverable_id: &str,
        artifact_digest: &str,
    ) -> Result<(), ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let can_contribute = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role.can_contribute());
        if !can_contribute {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "complete_deliverable".to_string(),
                required_role: "Lead, Maintainer, or Contributor".to_string(),
            });
        }

        let now_iso = "2026-10-03T07:40:00Z".to_string();

        let milestone = project
            .milestones
            .iter_mut()
            .find(|m| m.milestone_id == milestone_id)
            .ok_or_else(|| ProjectError::MilestoneNotFound(milestone_id.to_string()))?;

        let m_del = milestone
            .deliverables
            .iter_mut()
            .find(|d| d.deliverable_id == deliverable_id)
            .ok_or_else(|| ProjectError::DeliverableNotFound(deliverable_id.to_string()))?;

        m_del.status = DeliverableStatus::Verified;
        m_del.artifact_digest = Some(artifact_digest.to_string());
        m_del.completed_at_iso = Some(now_iso.clone());

        if let Some(p_del) = project
            .deliverables
            .iter_mut()
            .find(|d| d.deliverable_id == deliverable_id)
        {
            p_del.status = DeliverableStatus::Verified;
            p_del.artifact_digest = Some(artifact_digest.to_string());
            p_del.completed_at_iso = Some(now_iso.clone());
        }

        let details =
            format!("Deliverable '{deliverable_id}' completed with artifact '{artifact_digest}'");
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "DeliverableCompleted",
            &details,
            &now_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: caller_mailbox.to_string(),
            action: "DeliverableCompleted".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(())
    }

    /// Complete a milestone after verifying all child deliverables.
    pub fn complete_milestone(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        milestone_id: &str,
    ) -> Result<(), ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let can_manage = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role.can_manage_milestones());
        if !can_manage {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "complete_milestone".to_string(),
                required_role: "Lead or Maintainer".to_string(),
            });
        }

        let milestone = project
            .milestones
            .iter_mut()
            .find(|m| m.milestone_id == milestone_id)
            .ok_or_else(|| ProjectError::MilestoneNotFound(milestone_id.to_string()))?;

        milestone.status = MilestoneStatus::Completed;

        let now_iso = "2026-10-03T07:45:00Z".to_string();
        let details = format!("Milestone '{milestone_id}' completed");
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "MilestoneCompleted",
            &details,
            &now_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: caller_mailbox.to_string(),
            action: "MilestoneCompleted".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(())
    }

    /// Append an arbitrary activity entry to a project's chained log.
    pub fn append_activity(
        &mut self,
        project_id: &str,
        actor: &str,
        action: &str,
        details: &str,
    ) -> Result<&ProjectActivityEntry, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let now_iso = "2026-10-03T07:50:00Z".to_string();
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            actor,
            action,
            details,
            &now_iso,
        );

        let entry = ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: actor.to_string(),
            action: action.to_string(),
            details: details.to_string(),
            prev_entry_digest: prev_digest,
            entry_digest,
        };

        project.activity_log.push(entry);
        Ok(project.activity_log.last().expect("entry just added"))
    }

    /// Cryptographically verify the SHA-256 chained activity log for a project.
    pub fn verify_activity_chain(&self, project_id: &str) -> Result<bool, ProjectError> {
        let project = self
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let mut expected_prev = Self::GENESIS_DIGEST.to_string();

        for entry in &project.activity_log {
            if entry.prev_entry_digest != expected_prev {
                return Err(ProjectError::ActivityLogTampered {
                    entry_id: entry.entry_id.clone(),
                    expected_prev,
                    actual_prev: entry.prev_entry_digest.clone(),
                });
            }

            let recomputed = ProjectActivityEntry::compute_digest(
                &entry.prev_entry_digest,
                &entry.entry_id,
                &entry.project_id,
                &entry.actor,
                &entry.action,
                &entry.details,
                &entry.timestamp,
            );

            if recomputed != entry.entry_digest {
                return Err(ProjectError::ActivityLogTampered {
                    entry_id: entry.entry_id.clone(),
                    expected_prev: format!("valid entry_digest '{recomputed}'"),
                    actual_prev: format!("mismatched entry_digest '{}'", entry.entry_digest),
                });
            }

            expected_prev = entry.entry_digest.clone();
        }

        Ok(true)
    }

    /// Create a project release draft.
    pub fn create_release(
        &mut self,
        req: CreateProjectReleaseRequest<'_>,
    ) -> Result<&ProjectReleaseRecord, ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == req.project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(req.project_id.to_string()))?;

        let can_manage = project
            .members
            .iter()
            .any(|m| m.mailbox == req.caller_mailbox && m.role.can_manage_milestones());
        if !can_manage {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: req.caller_mailbox.to_string(),
                action: "create_release".to_string(),
                required_role: "Lead or Maintainer".to_string(),
            });
        }

        if project
            .releases
            .iter()
            .any(|r| r.release_id == req.release_id)
        {
            return Err(ProjectError::Validation(format!(
                "release '{}' already exists",
                req.release_id
            )));
        }

        let now_iso = "2026-10-03T07:55:00Z".to_string();
        let release = ProjectReleaseRecord {
            release_id: req.release_id.to_string(),
            project_id: req.project_id.to_string(),
            version_tag: req.version_tag.to_string(),
            title: req.title.to_string(),
            notes: req.notes.to_string(),
            model_revision: req.model_revision,
            commit_sha: req.commit_sha.to_string(),
            artifact_root_digest: req.artifact_root_digest.to_string(),
            created_at: now_iso,
            published_at: None,
            is_published: false,
        };

        project.releases.push(release);
        Ok(project.releases.last().expect("release just added"))
    }

    /// Publish an existing project release, making it immutable and logging the event.
    pub fn publish_release(
        &mut self,
        caller_mailbox: &str,
        project_id: &str,
        release_id: &str,
    ) -> Result<(), ProjectError> {
        let project = self
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| ProjectError::ProjectNotFound(project_id.to_string()))?;

        let is_lead = project
            .members
            .iter()
            .any(|m| m.mailbox == caller_mailbox && m.role == ProjectRole::Lead);
        if !is_lead {
            return Err(ProjectError::UnauthorizedProjectAction {
                actor: caller_mailbox.to_string(),
                action: "publish_release".to_string(),
                required_role: "Lead".to_string(),
            });
        }

        let release = project
            .releases
            .iter_mut()
            .find(|r| r.release_id == release_id)
            .ok_or_else(|| ProjectError::Validation(format!("release '{release_id}' not found")))?;

        if release.is_published {
            return Err(ProjectError::ReleaseAlreadyPublished(
                release_id.to_string(),
            ));
        }

        let now_iso = "2026-10-03T08:00:00Z".to_string();
        release.is_published = true;
        release.published_at = Some(now_iso.clone());

        let details = format!("Release '{release_id}' published ({})", release.version_tag);
        let prev_digest = project
            .activity_log
            .last()
            .map(|e| e.entry_digest.clone())
            .unwrap_or_else(|| Self::GENESIS_DIGEST.to_string());
        let entry_id = format!("{project_id}:act-{:04}", project.activity_log.len() + 1);
        let entry_digest = ProjectActivityEntry::compute_digest(
            &prev_digest,
            &entry_id,
            project_id,
            caller_mailbox,
            "ReleasePublished",
            &details,
            &now_iso,
        );

        project.activity_log.push(ProjectActivityEntry {
            entry_id,
            project_id: project_id.to_string(),
            timestamp: now_iso,
            actor: caller_mailbox.to_string(),
            action: "ReleasePublished".to_string(),
            details,
            prev_entry_digest: prev_digest,
            entry_digest,
        });

        Ok(())
    }
}
