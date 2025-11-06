//! Workspace repository trait.
//!
//! Defines the interface for workspace persistence operations.

use super::model::Workspace;
use crate::error::Result;
use async_trait::async_trait;

/// Repository for workspace persistence.
///
/// This trait defines the contract for persisting and retrieving complete workspace data.
///
/// # Design Goals
///
/// - **Simplicity**: Single repository for all workspace data
/// - **Flexibility**: Can be implemented with different storage backends
/// - **Performance**: AsyncDirStorage provides efficient file I/O with ACID guarantees
///
/// # Implementation Notes
///
/// Implementations should:
/// - Handle concurrent access safely
/// - Provide atomic updates for workspace data
/// - Support efficient listing/filtering operations
#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    /// Finds a workspace by its ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Workspace))`: Workspace found
    /// - `Ok(None)`: Workspace not found
    /// - `Err(_)`: Error occurred during retrieval
    async fn find_by_id(&self, workspace_id: &str) -> Result<Option<Workspace>>;

    /// Saves a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Workspace saved successfully
    /// - `Err(_)`: Error occurred during save
    async fn save(&self, workspace: &Workspace) -> Result<()>;

    /// Updates a workspace with a function.
    ///
    /// This provides a convenient way to load, modify, and save a workspace atomically.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to update
    /// * `f` - Function that modifies the workspace
    ///
    /// # Returns
    ///
    /// - `Ok(Workspace)`: Updated workspace
    /// - `Err(_)`: Workspace not found or error occurred
    ///
    /// # Example
    ///
    /// ```ignore
    /// repository.update("workspace-123", |ws| {
    ///     ws.is_favorite = !ws.is_favorite;
    /// }).await?;
    /// ```
    async fn update<F>(&self, workspace_id: &str, f: F) -> Result<Workspace>
    where
        F: FnOnce(&mut Workspace) + Send;

    /// Deletes a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Workspace deleted successfully (or didn't exist)
    /// - `Err(_)`: Error occurred during deletion
    async fn delete(&self, workspace_id: &str) -> Result<()>;

    /// Lists all workspaces.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Workspace>)`: All workspaces, sorted by last_accessed (desc)
    /// - `Err(_)`: Error occurred during listing
    async fn list_all(&self) -> Result<Vec<Workspace>>;

    /// Checks if a workspace exists.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// - `Ok(true)`: Workspace exists
    /// - `Ok(false)`: Workspace does not exist
    /// - `Err(_)`: Error occurred during check
    async fn exists(&self, workspace_id: &str) -> Result<bool> {
        Ok(self.find_by_id(workspace_id).await?.is_some())
    }
}

