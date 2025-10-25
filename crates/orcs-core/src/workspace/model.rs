use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a project-level workspace containing all resources and context
/// associated with a specific project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResources,
    /// Project-specific context and metadata
    pub project_context: ProjectContext,
}

/// Collection of all resources managed within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceResources {
    /// Files uploaded by the user or system
    pub uploaded_files: Vec<UploadedFile>,
    /// AI-generated documentation and artifacts
    pub generated_docs: Vec<GeneratedDoc>,
    /// Temporary files created during session operations
    pub temp_files: Vec<TempFile>,
}

/// Represents a file uploaded to the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    /// Unique identifier for the uploaded file
    pub id: String,
    /// Original filename
    pub name: String,
    /// Path to the stored file
    pub path: PathBuf,
    /// MIME type of the file
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: i64,
}

/// Represents an AI-generated document or artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedDoc {
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

/// Project-specific context and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    /// Programming languages detected in the project
    pub languages: Vec<String>,
    /// Build system or framework (e.g., "cargo", "npm", "maven")
    pub build_system: Option<String>,
    /// Project description or purpose
    pub description: Option<String>,
    /// Git repository URL if available
    pub repository_url: Option<String>,
    /// Additional metadata as key-value pairs
    pub metadata: std::collections::HashMap<String, String>,
}

/// Session-specific workspace view that references the parent workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWorkspace {
    /// ID of the parent workspace
    pub workspace_id: String,
    /// ID of the current session
    pub session_id: String,
    /// Temporary files specific to this session
    pub session_temp_files: Vec<TempFile>,
}

/// Represents a temporary file created during operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempFile {
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
