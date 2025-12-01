//! Workspace DTOs and domain conversions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use version_migrate::{FromDomain, IntoDomain, Versioned};

use orcs_core::workspace::{
    ProjectContext, SessionWorkspace, TempFile, Workspace, WorkspaceResources,
};

use super::uploaded_file::UploadedFileV1_4_0;

/// Represents a temporary file created during operations (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct TempFileV1 {
    /// Unique identifier for the temp file
    pub id: String,
    /// Path to the temporary file
    pub path: PathBuf,
    /// Purpose or description of the temp file
    pub purpose: String,
    /// Timestamp when the file was created
    pub created_at: i64,
    /// Whether the file should be deleted after session ends
    pub auto_delete: bool,
}

/// Project-specific context and metadata (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct ProjectContextV1 {
    /// Programming languages detected in the project
    #[serde(default)]
    pub languages: Vec<String>,
    /// Build system or framework (e.g., "cargo", "npm", "maven")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_system: Option<String>,
    /// Project description or purpose
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Git repository URL if available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
    /// Additional metadata as key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Collection of all resources managed within a workspace (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceResourcesV1 {
    /// Files uploaded by the user or system
    #[serde(default)]
    pub uploaded_files: Vec<UploadedFileV1_4_0>,
    /// Temporary files created during session operations
    #[serde(default)]
    pub temp_files: Vec<TempFileV1>,
}

/// Represents a project-level workspace (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct WorkspaceV1 {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResourcesV1,
    /// Project-specific context and metadata
    pub project_context: ProjectContextV1,
}

/// Represents a project-level workspace (DTO V1.1.0).
/// Added last_accessed and is_favorite fields.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct WorkspaceV1_1_0 {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResourcesV1,
    /// Project-specific context and metadata
    pub project_context: ProjectContextV1,
    /// Last accessed timestamp (UNIX timestamp in seconds)
    #[serde(default)]
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite
    #[serde(default)]
    pub is_favorite: bool,
}

/// Represents a project-level workspace (DTO V1.2.0).
/// Added last_active_session_id field for workspace-specific active session management.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.2.0")]
pub struct WorkspaceV1_2_0 {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResourcesV1,
    /// Project-specific context and metadata
    pub project_context: ProjectContextV1,
    /// Last accessed timestamp (UNIX timestamp in seconds)
    #[serde(default)]
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite
    #[serde(default)]
    pub is_favorite: bool,
    /// ID of the last active session in this workspace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_session_id: Option<String>,
}

/// Represents a project-level workspace (DTO V1.3.0).
/// Updated to support UploadedFile V1.4.0 (is_favorite, sort_order).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct WorkspaceV1_3_0 {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources (with UploadedFile V1.4.0)
    pub resources: WorkspaceResourcesV1,
    /// Project-specific context and metadata
    pub project_context: ProjectContextV1,
    /// Last accessed timestamp (UNIX timestamp in seconds)
    #[serde(default)]
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite
    #[serde(default)]
    pub is_favorite: bool,
    /// ID of the last active session in this workspace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_session_id: Option<String>,
}

/// Session-specific workspace view (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct SessionWorkspaceV1 {
    /// ID of the parent workspace
    pub workspace_id: String,
    /// ID of the current session
    pub session_id: String,
    /// Temporary files specific to this session
    #[serde(default)]
    pub session_temp_files: Vec<TempFileV1>,
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert TempFileV1 DTO to domain model.
impl IntoDomain<TempFile> for TempFileV1 {
    fn into_domain(self) -> TempFile {
        TempFile {
            id: self.id,
            path: self.path,
            purpose: self.purpose,
            created_at: self.created_at,
            auto_delete: self.auto_delete,
        }
    }
}

/// Convert domain model to TempFileV1 DTO for persistence.
impl From<&TempFile> for TempFileV1 {
    fn from(temp_file: &TempFile) -> Self {
        TempFileV1 {
            id: temp_file.id.clone(),
            path: temp_file.path.clone(),
            purpose: temp_file.purpose.clone(),
            created_at: temp_file.created_at,
            auto_delete: temp_file.auto_delete,
        }
    }
}

/// Convert ProjectContextV1 DTO to domain model.
impl IntoDomain<ProjectContext> for ProjectContextV1 {
    fn into_domain(self) -> ProjectContext {
        ProjectContext {
            languages: self.languages,
            build_system: self.build_system,
            description: self.description,
            repository_url: self.repository_url,
            metadata: self.metadata,
        }
    }
}

/// Convert domain model to ProjectContextV1 DTO for persistence.
impl From<&ProjectContext> for ProjectContextV1 {
    fn from(project_context: &ProjectContext) -> Self {
        ProjectContextV1 {
            languages: project_context.languages.clone(),
            build_system: project_context.build_system.clone(),
            description: project_context.description.clone(),
            repository_url: project_context.repository_url.clone(),
            metadata: project_context.metadata.clone(),
        }
    }
}

/// Convert WorkspaceResourcesV1 DTO to domain model.
impl IntoDomain<WorkspaceResources> for WorkspaceResourcesV1 {
    fn into_domain(self) -> WorkspaceResources {
        WorkspaceResources {
            uploaded_files: self
                .uploaded_files
                .into_iter()
                .map(|f| f.into_domain())
                .collect(),
            temp_files: self
                .temp_files
                .into_iter()
                .map(|t| t.into_domain())
                .collect(),
        }
    }
}

/// Convert domain model to WorkspaceResourcesV1 DTO for persistence.
impl From<&WorkspaceResources> for WorkspaceResourcesV1 {
    fn from(resources: &WorkspaceResources) -> Self {
        WorkspaceResourcesV1 {
            uploaded_files: resources
                .uploaded_files
                .iter()
                .map(UploadedFileV1_4_0::from)
                .collect(),
            temp_files: resources.temp_files.iter().map(TempFileV1::from).collect(),
        }
    }
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from WorkspaceV1 to WorkspaceV1_1_0.
/// Added last_accessed and is_favorite fields (default values).
impl version_migrate::MigratesTo<WorkspaceV1_1_0> for WorkspaceV1 {
    fn migrate(self) -> WorkspaceV1_1_0 {
        WorkspaceV1_1_0 {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            resources: self.resources,
            project_context: self.project_context,
            last_accessed: 0,   // Default: epoch (will be updated on first access)
            is_favorite: false, // Default: not favorite
        }
    }
}

/// Migration from WorkspaceV1_1_0 to WorkspaceV1_2_0.
/// Added last_active_session_id field for workspace-specific active session management.
impl version_migrate::MigratesTo<WorkspaceV1_2_0> for WorkspaceV1_1_0 {
    fn migrate(self) -> WorkspaceV1_2_0 {
        WorkspaceV1_2_0 {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            resources: self.resources,
            project_context: self.project_context,
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
            last_active_session_id: None, // Default: no previous active session
        }
    }
}

/// Migration from WorkspaceV1_2_0 to WorkspaceV1_3_0.
/// Updated to support UploadedFile V1.4.0 with is_favorite and sort_order.
/// This migration is transparent as the UploadedFile migration is handled automatically.
impl version_migrate::MigratesTo<WorkspaceV1_3_0> for WorkspaceV1_2_0 {
    fn migrate(self) -> WorkspaceV1_3_0 {
        WorkspaceV1_3_0 {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            resources: self.resources,
            project_context: self.project_context,
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
            last_active_session_id: self.last_active_session_id,
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert WorkspaceV1 DTO to domain model.
impl IntoDomain<Workspace> for WorkspaceV1 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            // workspace_dir is calculated from workspace_id, not stored in DTO
            // The caller (load_workspace) must set this field after conversion
            workspace_dir: PathBuf::new(),
            resources: self.resources.into_domain(),
            project_context: self.project_context.into_domain(),
            last_accessed: 0,             // Default for V1
            is_favorite: false,           // Default for V1
            last_active_session_id: None, // V1 doesn't have this field
        }
    }
}

/// Convert WorkspaceV1_1_0 DTO to domain model.
impl IntoDomain<Workspace> for WorkspaceV1_1_0 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            // workspace_dir is calculated from workspace_id, not stored in DTO
            // The caller (load_workspace) must set this field after conversion
            workspace_dir: PathBuf::new(),
            resources: self.resources.into_domain(),
            project_context: self.project_context.into_domain(),
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
            last_active_session_id: None, // V1_1_0 doesn't have this field
        }
    }
}

/// Convert WorkspaceV1_2_0 DTO to domain model.
impl IntoDomain<Workspace> for WorkspaceV1_2_0 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            // workspace_dir is calculated from workspace_id, not stored in DTO
            // The caller (load_workspace) must set this field after conversion
            workspace_dir: PathBuf::new(),
            resources: self.resources.into_domain(),
            project_context: self.project_context.into_domain(),
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
            last_active_session_id: self.last_active_session_id,
        }
    }
}

/// Convert WorkspaceV1_3_0 DTO to domain model.
impl IntoDomain<Workspace> for WorkspaceV1_3_0 {
    fn into_domain(self) -> Workspace {
        Workspace {
            id: self.id,
            name: self.name,
            root_path: self.root_path,
            // workspace_dir is calculated from workspace_id, not stored in DTO
            // The caller (load_workspace) must set this field after conversion
            workspace_dir: PathBuf::new(),
            resources: self.resources.into_domain(),
            project_context: self.project_context.into_domain(),
            last_accessed: self.last_accessed,
            is_favorite: self.is_favorite,
            last_active_session_id: self.last_active_session_id,
        }
    }
}

/// Convert domain model to WorkspaceV1 DTO for persistence.
impl From<&Workspace> for WorkspaceV1 {
    fn from(workspace: &Workspace) -> Self {
        WorkspaceV1 {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            resources: WorkspaceResourcesV1::from(&workspace.resources),
            project_context: ProjectContextV1::from(&workspace.project_context),
        }
    }
}

/// Convert domain model to WorkspaceV1_1_0 DTO for persistence.
impl From<&Workspace> for WorkspaceV1_1_0 {
    fn from(workspace: &Workspace) -> Self {
        WorkspaceV1_1_0 {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            resources: WorkspaceResourcesV1::from(&workspace.resources),
            project_context: ProjectContextV1::from(&workspace.project_context),
            last_accessed: workspace.last_accessed,
            is_favorite: workspace.is_favorite,
        }
    }
}

impl FromDomain<Workspace> for WorkspaceV1_1_0 {
    fn from_domain(domain: Workspace) -> Self {
        WorkspaceV1_1_0 {
            id: domain.id,
            name: domain.name,
            root_path: domain.root_path,
            resources: WorkspaceResourcesV1::from(&domain.resources),
            project_context: ProjectContextV1::from(&domain.project_context),
            last_accessed: domain.last_accessed,
            is_favorite: domain.is_favorite,
        }
    }
}

/// Convert domain model to WorkspaceV1_2_0 DTO for persistence.
impl From<&Workspace> for WorkspaceV1_2_0 {
    fn from(workspace: &Workspace) -> Self {
        WorkspaceV1_2_0 {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            resources: WorkspaceResourcesV1::from(&workspace.resources),
            project_context: ProjectContextV1::from(&workspace.project_context),
            last_accessed: workspace.last_accessed,
            is_favorite: workspace.is_favorite,
            last_active_session_id: workspace.last_active_session_id.clone(),
        }
    }
}

impl FromDomain<Workspace> for WorkspaceV1_2_0 {
    fn from_domain(domain: Workspace) -> Self {
        WorkspaceV1_2_0 {
            id: domain.id,
            name: domain.name,
            root_path: domain.root_path,
            resources: WorkspaceResourcesV1::from(&domain.resources),
            project_context: ProjectContextV1::from(&domain.project_context),
            last_accessed: domain.last_accessed,
            is_favorite: domain.is_favorite,
            last_active_session_id: domain.last_active_session_id,
        }
    }
}

/// Convert domain model to WorkspaceV1_3_0 DTO for persistence.
impl From<&Workspace> for WorkspaceV1_3_0 {
    fn from(workspace: &Workspace) -> Self {
        WorkspaceV1_3_0 {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            resources: WorkspaceResourcesV1::from(&workspace.resources),
            project_context: ProjectContextV1::from(&workspace.project_context),
            last_accessed: workspace.last_accessed,
            is_favorite: workspace.is_favorite,
            last_active_session_id: workspace.last_active_session_id.clone(),
        }
    }
}

impl FromDomain<Workspace> for WorkspaceV1_3_0 {
    fn from_domain(domain: Workspace) -> Self {
        WorkspaceV1_3_0 {
            id: domain.id,
            name: domain.name,
            root_path: domain.root_path,
            resources: WorkspaceResourcesV1::from(&domain.resources),
            project_context: ProjectContextV1::from(&domain.project_context),
            last_accessed: domain.last_accessed,
            is_favorite: domain.is_favorite,
            last_active_session_id: domain.last_active_session_id,
        }
    }
}

/// Convert SessionWorkspaceV1 DTO to domain model.
impl IntoDomain<SessionWorkspace> for SessionWorkspaceV1 {
    fn into_domain(self) -> SessionWorkspace {
        SessionWorkspace {
            workspace_id: self.workspace_id,
            session_id: self.session_id,
            session_temp_files: self
                .session_temp_files
                .into_iter()
                .map(|t| t.into_domain())
                .collect(),
        }
    }
}

/// Convert domain model to SessionWorkspaceV1 DTO for persistence.
impl From<&SessionWorkspace> for SessionWorkspaceV1 {
    fn from(session_workspace: &SessionWorkspace) -> Self {
        SessionWorkspaceV1 {
            workspace_id: session_workspace.workspace_id.clone(),
            session_id: session_workspace.session_id.clone(),
            session_temp_files: session_workspace
                .session_temp_files
                .iter()
                .map(TempFileV1::from)
                .collect(),
        }
    }
}

// ============================================================================
// Migrator factories
// ============================================================================

/// Creates a Migrator for TempFile entities.
///
/// # Migration Path
///
/// - V1.0.0 → TempFile: Converts DTO to domain model (no migration needed yet)
pub fn create_temp_file_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("temp_file" => [TempFileV1, TempFile])
        .expect("Failed to create temp_file migrator")
}

/// Creates a Migrator for ProjectContext entities.
///
/// # Migration Path
///
/// - V1.0.0 → ProjectContext: Converts DTO to domain model (no migration needed yet)
pub fn create_project_context_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("project_context" => [ProjectContextV1, ProjectContext])
        .expect("Failed to create project_context migrator")
}

/// Creates a Migrator for WorkspaceResources entities.
///
/// # Migration Path
///
/// - V1.0.0 → WorkspaceResources: Converts DTO to domain model (no migration needed yet)
pub fn create_workspace_resources_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("workspace_resources" => [WorkspaceResourcesV1, WorkspaceResources])
        .expect("Failed to create workspace_resources migrator")
}

/// Creates a Migrator for Workspace entities.
///
/// # Migration Path
///
/// - V1.0.0 → V1.1.0: Added last_accessed and is_favorite fields
/// - V1.1.0 → V1.2.0: Added last_active_session_id field
/// - V1.2.0 → V1.3.0: Updated to support UploadedFile V1.4.0 (is_favorite, sort_order)
/// - V1.3.0 → Workspace: Converts DTO to domain model
pub fn create_workspace_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("workspace" => [
        WorkspaceV1,
        WorkspaceV1_1_0,
        WorkspaceV1_2_0,
        WorkspaceV1_3_0,
        Workspace
    ], save = true)
    .expect("Failed to create workspace migrator")
}

/// Creates a Migrator for SessionWorkspace entities.
///
/// # Migration Path
///
/// - V1.0.0 → SessionWorkspace: Converts DTO to domain model (no migration needed yet)
pub fn create_session_workspace_migrator() -> version_migrate::Migrator {
    version_migrate::migrator!("session_workspace" => [SessionWorkspaceV1, SessionWorkspace])
        .expect("Failed to create session_workspace migrator")
}
