//! Workspace DTOs and domain conversions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use version_migrate::{IntoDomain, Versioned};

use orcs_core::workspace::{
    GeneratedDoc, ProjectContext, SessionWorkspace, TempFile,
    Workspace, WorkspaceResources,
};

use super::uploaded_file::UploadedFileV1_1_0;

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

/// Represents an AI-generated document or artifact (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct GeneratedDocV1 {
    /// Unique identifier for the generated document
    pub id: String,
    /// Title or name of the document
    pub title: String,
    /// Path to the stored document
    pub path: PathBuf,
    /// Type of document (e.g., "summary", "analysis", "diagram")
    pub doc_type: String,
    /// ID of the session that generated this document
    pub session_id: String,
    /// Timestamp when the document was generated
    pub generated_at: i64,
}

/// Project-specific context and metadata (DTO V1).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct ProjectContextV1 {
    /// Programming languages detected in the project
    #[serde(default)]
    pub languages: Vec<String>,
    /// Build system or framework (e.g., "cargo", "npm", "maven")
    #[serde(default)]
    pub build_system: Option<String>,
    /// Project description or purpose
    #[serde(default)]
    pub description: Option<String>,
    /// Git repository URL if available
    #[serde(default)]
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
    pub uploaded_files: Vec<UploadedFileV1_1_0>,
    /// AI-generated documentation and artifacts
    #[serde(default)]
    pub generated_docs: Vec<GeneratedDocV1>,
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

/// Convert GeneratedDocV1 DTO to domain model.
impl IntoDomain<GeneratedDoc> for GeneratedDocV1 {
    fn into_domain(self) -> GeneratedDoc {
        GeneratedDoc {
            id: self.id,
            title: self.title,
            path: self.path,
            doc_type: self.doc_type,
            session_id: self.session_id,
            generated_at: self.generated_at,
        }
    }
}

/// Convert domain model to GeneratedDocV1 DTO for persistence.
impl From<&GeneratedDoc> for GeneratedDocV1 {
    fn from(generated_doc: &GeneratedDoc) -> Self {
        GeneratedDocV1 {
            id: generated_doc.id.clone(),
            title: generated_doc.title.clone(),
            path: generated_doc.path.clone(),
            doc_type: generated_doc.doc_type.clone(),
            session_id: generated_doc.session_id.clone(),
            generated_at: generated_doc.generated_at,
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
            uploaded_files: self.uploaded_files.into_iter()
                .map(|f| f.into_domain())
                .collect(),
            generated_docs: self.generated_docs.into_iter()
                .map(|d| d.into_domain())
                .collect(),
            temp_files: self.temp_files.into_iter()
                .map(|t| t.into_domain())
                .collect(),
        }
    }
}

/// Convert domain model to WorkspaceResourcesV1 DTO for persistence.
impl From<&WorkspaceResources> for WorkspaceResourcesV1 {
    fn from(resources: &WorkspaceResources) -> Self {
        WorkspaceResourcesV1 {
            uploaded_files: resources.uploaded_files.iter()
                .map(UploadedFileV1_1_0::from)
                .collect(),
            generated_docs: resources.generated_docs.iter()
                .map(GeneratedDocV1::from)
                .collect(),
            temp_files: resources.temp_files.iter()
                .map(TempFileV1::from)
                .collect(),
        }
    }
}

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

/// Convert SessionWorkspaceV1 DTO to domain model.
impl IntoDomain<SessionWorkspace> for SessionWorkspaceV1 {
    fn into_domain(self) -> SessionWorkspace {
        SessionWorkspace {
            workspace_id: self.workspace_id,
            session_id: self.session_id,
            session_temp_files: self.session_temp_files.into_iter()
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
            session_temp_files: session_workspace.session_temp_files.iter()
                .map(TempFileV1::from)
                .collect(),
        }
    }
}
