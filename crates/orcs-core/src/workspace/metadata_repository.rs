//! Workspace metadata repository trait.
//!
//! Defines the interface for workspace metadata persistence operations.
//! This is separate from the full workspace data to allow for:
//! - Lightweight, frequent updates (last_accessed, is_favorite)
//! - Different storage strategies (TOML, SQLite, etc.)
//! - Performance optimization

use super::model::Workspace;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Lightweight workspace metadata (frequently updated).
///
/// This contains only the fields that change frequently or are needed
/// for listing/filtering workspaces without loading full workspace data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceMetadata {
    /// Unique identifier for the workspace
    pub id: String,
    /// Name of the workspace (typically derived from project name)
    pub name: String,
    /// Root directory path of the project
    pub root_path: PathBuf,
    /// Last accessed timestamp (UNIX timestamp in seconds)
    pub last_accessed: i64,
    /// Whether this workspace is marked as favorite
    pub is_favorite: bool,
}

impl WorkspaceMetadata {
    /// Creates metadata from a full Workspace
    pub fn from_workspace(workspace: &Workspace) -> Self {
        WorkspaceMetadata {
            id: workspace.id.clone(),
            name: workspace.name.clone(),
            root_path: workspace.root_path.clone(),
            last_accessed: workspace.last_accessed,
            is_favorite: workspace.is_favorite,
        }
    }

    /// Applies metadata updates to a Workspace
    pub fn apply_to_workspace(&self, workspace: &mut Workspace) {
        workspace.id = self.id.clone();
        workspace.name = self.name.clone();
        workspace.root_path = self.root_path.clone();
        workspace.last_accessed = self.last_accessed;
        workspace.is_favorite = self.is_favorite;
    }
}

/// Repository for workspace metadata persistence.
///
/// This trait defines the contract for persisting and retrieving workspace metadata,
/// which is a lightweight subset of the full Workspace data.
///
/// # Design Goals
///
/// - **Performance**: Metadata is small and changes frequently
/// - **Flexibility**: Can be implemented with different storage backends
/// - **Isolation**: Metadata updates don't require loading full workspace data
///
/// # Implementation Notes
///
/// Implementations should:
/// - Handle concurrent access safely
/// - Provide atomic updates for metadata
/// - Support efficient listing/filtering operations
#[async_trait]
pub trait WorkspaceMetadataRepository: Send + Sync {
    /// Finds workspace metadata by its ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace
    ///
    /// # Returns
    ///
    /// - `Ok(Some(WorkspaceMetadata))`: Metadata found
    /// - `Ok(None)`: Workspace not found
    /// - `Err(_)`: Error occurred during retrieval
    async fn find_metadata(&self, workspace_id: &str) -> Result<Option<WorkspaceMetadata>>;

    /// Saves workspace metadata.
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Metadata saved successfully
    /// - `Err(_)`: Error occurred during save
    async fn save_metadata(&self, metadata: &WorkspaceMetadata) -> Result<()>;

    /// Updates workspace metadata with a function.
    ///
    /// This provides a convenient way to load, modify, and save metadata atomically.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to update
    /// * `f` - Function that modifies the metadata
    ///
    /// # Returns
    ///
    /// - `Ok(WorkspaceMetadata)`: Updated metadata
    /// - `Err(_)`: Workspace not found or error occurred
    ///
    /// # Example
    ///
    /// ```ignore
    /// repository.update_metadata("workspace-123", |meta| {
    ///     meta.is_favorite = !meta.is_favorite;
    /// }).await?;
    /// ```
    async fn update_metadata<F>(&self, workspace_id: &str, f: F) -> Result<WorkspaceMetadata>
    where
        F: FnOnce(&mut WorkspaceMetadata) + Send;

    /// Deletes workspace metadata.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Metadata deleted successfully (or didn't exist)
    /// - `Err(_)`: Error occurred during deletion
    async fn delete_metadata(&self, workspace_id: &str) -> Result<()>;

    /// Lists all workspace metadata.
    ///
    /// This is optimized for listing workspaces without loading full data.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<WorkspaceMetadata>)`: All workspace metadata, sorted by last_accessed (desc)
    /// - `Err(_)`: Error occurred during listing
    async fn list_all_metadata(&self) -> Result<Vec<WorkspaceMetadata>>;

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
        Ok(self.find_metadata(workspace_id).await?.is_some())
    }
}
