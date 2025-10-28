//! Workspace management trait definition.
//!
//! This module defines the `WorkspaceManager` trait, which provides an abstraction
//! for managing workspaces, files, and their associations with sessions.

use async_trait::async_trait;
use std::path::Path;

use crate::error::Result;
use crate::workspace::model::{SessionWorkspace, TempFile, UploadedFile, Workspace};

/// Trait for managing workspaces and their associated files.
///
/// The `WorkspaceManager` provides a high-level interface for:
/// - Creating and retrieving workspaces
/// - Managing files within workspaces
/// - Creating temporary files
/// - Reading file contents
/// - Managing session-workspace associations
///
/// Implementations should ensure thread-safety and asynchronous operation.
#[async_trait]
pub trait WorkspaceManager: Send + Sync {
    /// Gets the workspace for the current working directory.
    ///
    /// This method determines the current working directory and returns the workspace
    /// associated with it, creating one if necessary.
    ///
    /// # Returns
    ///
    /// Returns the workspace for the current working directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current directory cannot be determined
    /// - The workspace cannot be created or retrieved
    async fn get_current_workspace(&self) -> Result<Workspace>;

    /// Gets an existing workspace or creates a new one for the given repository path.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the repository
    ///
    /// # Returns
    ///
    /// Returns the workspace associated with the repository path, creating it if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be created or retrieved.
    async fn get_or_create_workspace(&self, repo_path: &Path) -> Result<Workspace>;

    /// Retrieves a workspace by its ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The unique identifier of the workspace
    ///
    /// # Returns
    ///
    /// Returns `Some(Workspace)` if found, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_workspace(&self, workspace_id: &str) -> Result<Option<Workspace>>;

    /// Adds a file from the filesystem to a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to add the file to
    /// * `source_path` - The path to the source file
    ///
    /// # Returns
    ///
    /// Returns the `UploadedFile` record representing the added file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file cannot be read
    /// - The file cannot be copied to the workspace storage
    /// - The database operation fails
    async fn add_file_to_workspace(
        &self,
        workspace_id: &str,
        source_path: &Path,
    ) -> Result<UploadedFile>;

    /// Adds a file from byte data to a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to add the file to
    /// * `filename` - The name of the file
    /// * `data` - The file content as bytes
    /// * `session_id` - Optional session ID if file was saved from a chat message
    /// * `message_timestamp` - Optional message timestamp if file was saved from a chat message
    /// * `author` - Optional author identifier (user ID, persona ID, or "system")
    ///
    /// # Returns
    ///
    /// Returns the `UploadedFile` record representing the added file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file cannot be written to storage
    /// - The database operation fails
    async fn add_file_from_bytes(
        &self,
        workspace_id: &str,
        filename: &str,
        data: &[u8],
        session_id: Option<String>,
        message_timestamp: Option<String>,
        author: Option<String>,
    ) -> Result<UploadedFile>;

    /// Deletes a file from a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    /// * `file_id` - The ID of the file to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file does not exist
    /// - The file cannot be deleted from storage
    /// - The database operation fails
    async fn delete_file_from_workspace(&self, workspace_id: &str, file_id: &str) -> Result<()>;

    /// Renames a file in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    /// * `file_id` - The ID of the file to rename
    /// * `new_name` - The new name for the file
    ///
    /// # Returns
    ///
    /// Returns the updated `UploadedFile` record.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file does not exist
    /// - A file with the new name already exists
    /// - The file cannot be renamed
    /// - The database operation fails
    async fn rename_file_in_workspace(
        &self,
        workspace_id: &str,
        file_id: &str,
        new_name: &str,
    ) -> Result<UploadedFile>;

    /// Creates a temporary file associated with a session and workspace.
    ///
    /// Temporary files are typically used for intermediate data during a session
    /// and may be cleaned up when the session ends.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session creating the file
    /// * `workspace_id` - The ID of the workspace the file belongs to
    /// * `filename` - The name of the temporary file
    /// * `content` - The content to write to the file
    ///
    /// # Returns
    ///
    /// Returns the `TempFile` record representing the created temporary file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file cannot be written to storage
    /// - The database operation fails
    async fn create_temp_file(
        &self,
        session_id: &str,
        workspace_id: &str,
        filename: &str,
        content: &[u8],
    ) -> Result<TempFile>;

    /// Reads the content of a file in a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace containing the file
    /// * `relative_path` - The relative path of the file within the workspace
    ///
    /// # Returns
    ///
    /// Returns the file content as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The file does not exist
    /// - The file cannot be read
    /// - The file content is not valid UTF-8
    async fn read_file_content(&self, workspace_id: &str, relative_path: &str) -> Result<String>;

    /// Retrieves the workspace association for a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session
    ///
    /// # Returns
    ///
    /// Returns `Some(SessionWorkspace)` if the session has an associated workspace,
    /// `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    async fn get_session_workspace(&self, session_id: &str) -> Result<Option<SessionWorkspace>>;

    /// Lists all registered workspaces.
    ///
    /// Returns a list of all workspaces sorted by last accessed time (most recent first).
    /// This is useful for UI displays of available workspaces.
    ///
    /// # Returns
    ///
    /// Returns a vector of all registered workspaces.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace listing fails.
    async fn list_all_workspaces(&self) -> Result<Vec<Workspace>>;

    /// Toggles the favorite status of a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The update operation fails
    async fn toggle_favorite(&self, workspace_id: &str) -> Result<()>;

    /// Updates the last accessed timestamp of a workspace.
    ///
    /// This should be called when a workspace is accessed or switched to.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The update operation fails
    async fn touch_workspace(&self, workspace_id: &str) -> Result<()>;

    /// Saves a workspace to persistent storage.
    ///
    /// This method updates the workspace data in storage, including all fields
    /// such as last_accessed, is_favorite, last_active_session_id, etc.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace to save
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    async fn save_workspace(&self, workspace: &Workspace) -> Result<()>;

    /// Deletes a workspace from persistent storage.
    ///
    /// This method removes the workspace metadata and all associated files.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The deletion operation fails
    async fn delete_workspace(&self, workspace_id: &str) -> Result<()>;
}
