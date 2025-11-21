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
    /// Directory where workspace data is stored (e.g., ~/.orcs/workspaces/{id})
    pub workspace_dir: PathBuf,
    /// Collection of all workspace resources
    pub resources: WorkspaceResources,
    /// Project-specific context and metadata
    pub project_context: ProjectContext,
    /// Last accessed timestamp (UNIX timestamp in seconds)
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite
    pub is_favorite: bool,
    /// ID of the last active session in this workspace
    pub last_active_session_id: Option<String>,
}

/// Collection of all resources managed within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceResources {
    /// Files uploaded by the user or system
    pub uploaded_files: Vec<UploadedFile>,
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
    /// Session ID if this file was saved from a chat message
    pub session_id: Option<String>,
    /// Message timestamp if this file was saved from a chat message (ISO 8601)
    pub message_timestamp: Option<String>,
    /// Author of the file (user ID, persona ID, or "system")
    pub author: Option<String>,
    /// Whether this file is archived (hidden by default)
    #[serde(default)]
    pub is_archived: bool,
    /// Whether this file is marked as favorite (pinned to top)
    #[serde(default)]
    pub is_favorite: bool,
    /// Manual sort order (optional, for custom ordering within favorites)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
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
