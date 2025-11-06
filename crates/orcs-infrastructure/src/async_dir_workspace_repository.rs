//! AsyncDirStorage-based Workspace repository implementation.
//!
//! This provides a repository for full Workspace data (including resources metadata)
//! using AsyncDirStorage from version-migrate for ACID guarantees and async I/O.

use crate::{ServiceType, dto::create_workspace_migrator, storage_repository::{StorageRepository, is_not_found}};
use async_trait::async_trait;
use orcs_core::{
    error::{OrcsError, Result},
    workspace::{Workspace, WorkspaceRepository},
};
use std::path::Path;
use tokio::fs;
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// AsyncDirStorage-based workspace repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// └── workspace_data/
///     ├── workspace-id-1.toml
///     ├── workspace-id-2.toml
///     └── workspace-id-3.toml
/// ```
///
/// Note: This stores full Workspace data (resources metadata) in TOML format.
/// Actual files (uploaded_files, temp_files) are managed separately by FileSystemWorkspaceManager.
pub struct AsyncDirWorkspaceRepository {
    storage: AsyncDirStorage,
}

impl StorageRepository for AsyncDirWorkspaceRepository {
    const SERVICE_TYPE: crate::paths::ServiceType = ServiceType::Workspace;
    const ENTITY_NAME: &'static str = "workspace";

    fn storage(&self) -> &AsyncDirStorage {
        &self.storage
    }
}

impl AsyncDirWorkspaceRepository {
    /// Creates an AsyncDirWorkspaceRepository instance at the default location.
    ///
    /// Uses centralized path management via `ServiceType::Workspace`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default() -> Result<Self> {
        Self::new(None).await
    }

    /// Creates a new AsyncDirWorkspaceRepository.
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
    pub async fn new(base_dir: Option<&Path>) -> Result<Self> {
        use crate::paths::{OrcsPaths, ServiceType};

        // Create AsyncDirStorage via centralized helper
        let migrator = create_workspace_migrator();
        let orcs_paths = OrcsPaths::new(None);
        let storage = orcs_paths
            .create_async_dir_storage(ServiceType::Workspace, migrator)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create workspace storage: {}", e)))?;

        Ok(Self { storage })
    }
}

#[async_trait]
impl WorkspaceRepository for AsyncDirWorkspaceRepository {
    async fn find_by_id(&self, workspace_id: &str) -> Result<Option<Workspace>> {
        match self
            .storage
            .load::<Workspace>(Self::ENTITY_NAME, workspace_id)
            .await
        {
            Ok(workspace) => Ok(Some(workspace)),
            Err(e) => {
                if is_not_found(&e) {
                    Ok(None)
                } else {
                     Err(OrcsError::Io(e.to_string()))
                }
            }
        }
    }

    async fn save(&self, workspace: &Workspace) -> Result<()> {
        self.storage
            .save(Self::ENTITY_NAME, &workspace.id, workspace)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to save workspace: {}", e)))?;
        Ok(())
    }

    async fn update<F>(&self, workspace_id: &str, f: F) -> Result<Workspace>
    where
        F: FnOnce(&mut Workspace) + Send,
    {
        let mut workspace = self
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| OrcsError::Io(format!("Workspace '{}' not found", workspace_id)))?;

        f(&mut workspace);

        self.save(&workspace).await?;

        Ok(workspace)
    }

    async fn delete(&self, workspace_id: &str) -> Result<()> {
        self.storage
            .delete(workspace_id)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to delete workspace: {}", e)))?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Workspace>> {
        let all_workspaces = self
            .storage
            .load_all::<Workspace>(Self::ENTITY_NAME)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to load all workspaces: {}", e)))?;

        let mut workspaces: Vec<Workspace> =
            all_workspaces.into_iter().map(|(_id, ws)| ws).collect();

        // Sort by last_accessed (descending)
        workspaces.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

        Ok(workspaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::workspace::{ProjectContext, WorkspaceResources};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceRepository::new(temp_dir.path())
            .await
            .unwrap();

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            workspace_dir: PathBuf::from("/test/workspace"),
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: 1000,
            is_favorite: true,
            last_active_session_id: None,
        };

        // Save workspace
        repo.save(&workspace).await.unwrap();

        // Find workspace
        let found = repo.find_by_id("test-workspace").await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, workspace.id);
        assert_eq!(found.name, workspace.name);
        assert_eq!(found.root_path, workspace.root_path);
        assert_eq!(found.last_accessed, workspace.last_accessed);
        assert_eq!(found.is_favorite, workspace.is_favorite);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceRepository::new(temp_dir.path())
            .await
            .unwrap();

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            workspace_dir: PathBuf::from("/test/workspace"),
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: 1000,
            is_favorite: false,
            last_active_session_id: None,
        };

        repo.save(&workspace).await.unwrap();
        assert!(repo.find_by_id("test-workspace").await.unwrap().is_some());

        repo.delete("test-workspace").await.unwrap();
        assert!(repo.find_by_id("test-workspace").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_all() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceRepository::new(temp_dir.path())
            .await
            .unwrap();

        let workspace1 = Workspace {
            id: "workspace-1".to_string(),
            name: "Workspace 1".to_string(),
            root_path: PathBuf::from("/test/path1"),
            workspace_dir: PathBuf::from("/test/workspace1"),
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: 1000,
            is_favorite: false,
            last_active_session_id: None,
        };

        let workspace2 = Workspace {
            id: "workspace-2".to_string(),
            name: "Workspace 2".to_string(),
            root_path: PathBuf::from("/test/path2"),
            workspace_dir: PathBuf::from("/test/workspace2"),
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: 2000,
            is_favorite: true,
            last_active_session_id: None,
        };

        repo.save(&workspace1).await.unwrap();
        repo.save(&workspace2).await.unwrap();

        let all = repo.list_all().await.unwrap();
        assert_eq!(all.len(), 2);

        // Should be sorted by last_accessed (descending)
        assert_eq!(all[0].id, "workspace-2");
        assert_eq!(all[1].id, "workspace-1");
    }

    #[tokio::test]
    async fn test_find_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceRepository::new(temp_dir.path())
            .await
            .unwrap();

        let found = repo.find_by_id("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirWorkspaceRepository::new(temp_dir.path())
            .await
            .unwrap();

        assert!(!repo.exists("test-workspace").await.unwrap());

        let workspace = Workspace {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            workspace_dir: PathBuf::from("/test/workspace"),
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: 1000,
            is_favorite: false,
            last_active_session_id: None,
        };

        repo.save(&workspace).await.unwrap();
        assert!(repo.exists("test-workspace").await.unwrap());
    }
}
