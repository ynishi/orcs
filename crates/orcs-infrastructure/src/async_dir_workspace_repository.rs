//! AsyncDirStorage-based Workspace repository implementation.
//!
//! This provides a repository for full Workspace data (including resources metadata)
//! using AsyncDirStorage from version-migrate for ACID guarantees and async I/O.

use crate::dto::create_workspace_migrator;
use orcs_core::{
    error::{OrcsError, Result},
    workspace::Workspace,
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

impl AsyncDirWorkspaceRepository {
    /// Creates an AsyncDirWorkspaceRepository instance at the default location.
    ///
    /// Uses `~/.config/orcs` as the base directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default_location() -> Result<Self> {
        use crate::paths::OrcsPaths;
        let base_dir = OrcsPaths::config_dir()
            .map_err(|e| OrcsError::Io(format!("Failed to get config directory: {}", e)))?;
        Self::new(base_dir).await
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
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure base directory exists
        fs::create_dir_all(&base_dir)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create base directory: {}", e)))?;

        // Setup AppPaths with CustomBase strategy
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base_dir));

        // Setup migrator
        let migrator = create_workspace_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage for "workspace_data" subdirectory
        let storage = AsyncDirStorage::new(paths, "workspace_data", migrator, strategy)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to create AsyncDirStorage: {}", e)))?;

        Ok(Self { storage })
    }

    /// Loads a workspace by ID.
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
    pub async fn find_by_id(&self, workspace_id: &str) -> Result<Option<Workspace>> {
        match self
            .storage
            .load::<Workspace>("workspace", workspace_id)
            .await
        {
            Ok(workspace) => Ok(Some(workspace)),
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
    pub async fn save(&self, workspace: &Workspace) -> Result<()> {
        self.storage
            .save("workspace", &workspace.id, workspace)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to save workspace: {}", e)))?;
        Ok(())
    }

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
    pub async fn delete(&self, workspace_id: &str) -> Result<()> {
        self.storage
            .delete(workspace_id)
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to delete workspace: {}", e)))?;
        Ok(())
    }

    /// Lists all workspaces.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Workspace>)`: All workspaces, sorted by last_accessed (desc)
    /// - `Err(_)`: Error occurred during listing
    pub async fn list_all(&self) -> Result<Vec<Workspace>> {
        let all_workspaces = self
            .storage
            .load_all::<Workspace>("workspace")
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to load all workspaces: {}", e)))?;

        let mut workspaces: Vec<Workspace> =
            all_workspaces.into_iter().map(|(_id, ws)| ws).collect();

        // Sort by last_accessed (descending)
        workspaces.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

        Ok(workspaces)
    }

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
    pub async fn exists(&self, workspace_id: &str) -> Result<bool> {
        Ok(self.find_by_id(workspace_id).await?.is_some())
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
        };

        repo.save(&workspace).await.unwrap();
        assert!(repo.exists("test-workspace").await.unwrap());
    }
}
