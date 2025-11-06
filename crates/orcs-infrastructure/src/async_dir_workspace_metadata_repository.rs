//! AsyncDirStorage-based WorkspaceMetadataRepository implementation
//!
//! This replaces TomlWorkspaceMetadataRepository with a version-migrate AsyncDirStorage-based implementation.

use crate::dto::create_workspace_metadata_migrator;
use async_trait::async_trait;
use orcs_core::{
    error::{OrcsError, Result},
    workspace::{WorkspaceMetadata, WorkspaceMetadataRepository},
};
use std::path::Path;
use tokio::fs;

#[cfg(test)]
use std::path::PathBuf;
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// AsyncDirStorage-based workspace metadata repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// └── workspaces/
///     ├── workspace-id-1.toml
///     ├── workspace-id-2.toml
///     └── workspace-id-3.toml
/// ```
///
/// Note: This uses a flat structure (`{id}.toml`) instead of the old structure (`{id}/metadata.toml`).
/// Data migration will be needed for existing workspaces.
pub struct AsyncDirWorkspaceMetadataRepository {
    storage: AsyncDirStorage,
}

impl AsyncDirWorkspaceMetadataRepository {
    /// Creates an AsyncDirWorkspaceMetadataRepository instance at the default location.
    ///
    /// Uses centralized path management via `ServiceType::WorkspaceMetadata`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default_location() -> Result<Self> {
        use crate::paths::{OrcsPaths, ServiceType};
        let path_type = OrcsPaths::get_path(ServiceType::WorkspaceMetadata)
            .map_err(|e| OrcsError::Io(format!("Failed to get workspace metadata directory: {}", e)))?;
        let base_dir = path_type.into_path_buf();
        Self::new(base_dir).await
    }

    /// Creates a new AsyncDirWorkspaceMetadataRepository.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory (e.g., ~/.config/orcs)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Directory creation fails
    /// - AsyncDirStorage initialization fails
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure base directory exists
        fs::create_dir_all(&base_dir)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create base directory: {}", e)))?;

        // Setup AppPaths with CustomBase strategy
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base_dir));

        // Setup migrator
        let migrator = create_workspace_metadata_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage for "workspaces" subdirectory
        let storage = AsyncDirStorage::new(paths, "workspaces", migrator, strategy)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create AsyncDirStorage: {}", e)))?;

        Ok(Self { storage })
    }
}

#[async_trait]
impl WorkspaceMetadataRepository for AsyncDirWorkspaceMetadataRepository {
    async fn find_metadata(&self, workspace_id: &str) -> Result<Option<WorkspaceMetadata>> {
        match self
            .storage
            .load::<WorkspaceMetadata>("workspace_metadata", workspace_id)
            .await
        {
            Ok(metadata) => Ok(Some(metadata)),
            Err(e) => {
                // Check if it's a "not found" error
                let error_str = e.to_string();
                if error_str.contains("No such file or directory")
                    || error_str.contains("not found")
                    || error_str.contains("cannot find")
                {
                    return Ok(None);
                }
                Err(OrcsError::Io(e.to_string()))
            }
        }
    }

    async fn save_metadata(&self, metadata: &WorkspaceMetadata) -> Result<()> {
        self.storage
            .save("workspace_metadata", &metadata.id, metadata)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to save metadata: {}", e)))?;
        Ok(())
    }

    async fn update_metadata<F>(&self, workspace_id: &str, f: F) -> Result<WorkspaceMetadata>
    where
        F: FnOnce(&mut WorkspaceMetadata) + Send,
    {
        let mut metadata = self
            .find_metadata(workspace_id)
            .await?
            .ok_or_else(|| OrcsError::Io(format!("Workspace '{}' not found", workspace_id)))?;

        f(&mut metadata);

        self.save_metadata(&metadata).await?;

        Ok(metadata)
    }

    async fn delete_metadata(&self, workspace_id: &str) -> Result<()> {
        self.storage
            .delete(workspace_id)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to delete metadata: {}", e)))?;
        Ok(())
    }

    async fn list_all_metadata(&self) -> Result<Vec<WorkspaceMetadata>> {
        let all_metadata = self
            .storage
            .load_all::<WorkspaceMetadata>("workspace_metadata")
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to load all metadata: {}", e)))?;

        let mut metadata_list: Vec<WorkspaceMetadata> = all_metadata
            .into_iter()
            .map(|(_id, metadata)| metadata)
            .collect();

        // Sort by last_accessed (descending)
        metadata_list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

        Ok(metadata_list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_find_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        let metadata = WorkspaceMetadata {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            is_favorite: true,
        };

        // Save metadata
        repo.save_metadata(&metadata).await.unwrap();

        // Find metadata
        let found = repo.find_metadata("test-workspace").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, metadata.id);
        assert_eq!(found.name, metadata.name);
        assert_eq!(found.root_path, metadata.root_path);
        assert_eq!(found.last_accessed, metadata.last_accessed);
        assert_eq!(found.is_favorite, metadata.is_favorite);
    }

    #[tokio::test]
    async fn test_update_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        let metadata = WorkspaceMetadata {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            last_accessed: 1000,
            is_favorite: false,
        };

        repo.save_metadata(&metadata).await.unwrap();

        // Update metadata
        let updated = repo
            .update_metadata("test-workspace", |meta| {
                meta.is_favorite = true;
                meta.last_accessed = 2000;
            })
            .await
            .unwrap();

        assert_eq!(updated.is_favorite, true);
        assert_eq!(updated.last_accessed, 2000);

        // Verify persistence
        let found = repo.find_metadata("test-workspace").await.unwrap().unwrap();
        assert_eq!(found.is_favorite, true);
        assert_eq!(found.last_accessed, 2000);
    }

    #[tokio::test]
    async fn test_delete_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        let metadata = WorkspaceMetadata {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            last_accessed: 1000,
            is_favorite: false,
        };

        repo.save_metadata(&metadata).await.unwrap();
        assert!(
            repo.find_metadata("test-workspace")
                .await
                .unwrap()
                .is_some()
        );

        repo.delete_metadata("test-workspace").await.unwrap();
        assert!(
            repo.find_metadata("test-workspace")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_list_all_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        let metadata1 = WorkspaceMetadata {
            id: "workspace-1".to_string(),
            name: "Workspace 1".to_string(),
            root_path: PathBuf::from("/test/path1"),
            last_accessed: 1000,
            is_favorite: false,
        };

        let metadata2 = WorkspaceMetadata {
            id: "workspace-2".to_string(),
            name: "Workspace 2".to_string(),
            root_path: PathBuf::from("/test/path2"),
            last_accessed: 2000,
            is_favorite: true,
        };

        repo.save_metadata(&metadata1).await.unwrap();
        repo.save_metadata(&metadata2).await.unwrap();

        let all = repo.list_all_metadata().await.unwrap();
        assert_eq!(all.len(), 2);

        // Should be sorted by last_accessed (descending)
        assert_eq!(all[0].id, "workspace-2");
        assert_eq!(all[1].id, "workspace-1");
    }

    #[tokio::test]
    async fn test_find_nonexistent_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        let found = repo.find_metadata("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceMetadataRepository::new(temp_dir.path())
            .await
            .unwrap();

        assert!(!repo.exists("test-workspace").await.unwrap());

        let metadata = WorkspaceMetadata {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            last_accessed: 1000,
            is_favorite: false,
        };

        repo.save_metadata(&metadata).await.unwrap();
        assert!(repo.exists("test-workspace").await.unwrap());
    }
}
