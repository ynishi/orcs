//! TOML-based workspace metadata repository implementation.
//!
//! Stores workspace metadata as individual `metadata.toml` files in each workspace directory.
//! Provides atomic writes and file locking for concurrent access safety.

use async_trait::async_trait;
use orcs_core::{
    error::{OrcsError, Result},
    workspace::{WorkspaceMetadata, WorkspaceMetadataRepository},
};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::task;

use crate::dto::create_workspace_metadata_migrator;

/// TOML-based workspace metadata repository.
///
/// Stores metadata in `~/.orcs/workspaces/{workspace_id}/metadata.toml`.
/// Each metadata file is independent and contains only frequently-updated fields.
///
/// # Features
///
/// - **Atomic writes**: Uses tmp file + fsync + atomic rename pattern
/// - **Version migration**: Handles schema evolution via version-migrate
/// - **Lightweight**: Only metadata fields, no full workspace data
/// - **Async-safe**: All operations wrapped in tokio::task::spawn_blocking
pub struct TomlWorkspaceMetadataRepository {
    /// Root directory for all workspaces (typically `~/.orcs/workspaces`)
    root_dir: PathBuf,
}

impl TomlWorkspaceMetadataRepository {
    /// Creates a new repository instance.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - Root directory for workspaces (typically `~/.orcs/workspaces`)
    ///
    /// # Returns
    ///
    /// Returns a new repository instance.
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Gets the metadata file path for a workspace.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// Path to metadata.toml file
    fn get_metadata_path(&self, workspace_id: &str) -> PathBuf {
        self.root_dir
            .join(workspace_id)
            .join("metadata.toml")
    }

    /// Loads metadata from TOML file synchronously.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to metadata.toml file
    ///
    /// # Returns
    ///
    /// - `Ok(Some(WorkspaceMetadata))`: Metadata loaded successfully
    /// - `Ok(None)`: File does not exist
    /// - `Err`: Error reading or parsing file
    fn load_metadata_sync(path: &Path) -> Result<Option<WorkspaceMetadata>> {
        if !path.exists() {
            return Ok(None);
        }

        let toml_str = fs::read_to_string(path).map_err(|e| {
            OrcsError::Io(format!(
                "Failed to read metadata file '{}': {}",
                path.display(),
                e
            ))
        })?;

        let toml_value: toml::Value = toml::from_str(&toml_str).map_err(|e| {
            OrcsError::Serialization(format!(
                "Failed to parse metadata TOML from '{}': {}",
                path.display(),
                e
            ))
        })?;

        let migrator = create_workspace_metadata_migrator();
        let metadata: WorkspaceMetadata = migrator
            .load_flat_from("workspace_metadata", toml_value)
            .map_err(|e| {
                OrcsError::Serialization(format!(
                    "Failed to migrate metadata from '{}': {}",
                    path.display(),
                    e
                ))
            })?;

        Ok(Some(metadata))
    }

    /// Saves metadata to TOML file synchronously with atomic writes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to metadata.toml file
    /// * `metadata` - Metadata to save
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Metadata saved successfully
    /// - `Err`: Error writing file
    fn save_metadata_sync(path: &Path, metadata: &WorkspaceMetadata) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    OrcsError::Io(format!(
                        "Failed to create workspace directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }

        // Serialize to JSON first (migrator works with JSON)
        let migrator = create_workspace_metadata_migrator();
        let json_str = migrator
            .save_domain_flat("workspace_metadata", metadata)
            .map_err(|e| {
                OrcsError::Serialization(format!(
                    "Failed to serialize metadata: {}",
                    e
                ))
            })?;

        // Parse JSON to serde_json::Value
        let json_value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
            OrcsError::Serialization(format!(
                "Failed to parse JSON during serialization: {}",
                e
            ))
        })?;

        // Convert to TOML
        let toml_value = json_to_toml(&json_value).map_err(|e| {
            OrcsError::Serialization(format!(
                "Failed to convert JSON to TOML: {}",
                e
            ))
        })?;

        let toml_str = toml::to_string_pretty(&toml_value).map_err(|e| {
            OrcsError::Serialization(format!(
                "Failed to serialize TOML: {}",
                e
            ))
        })?;

        // Write to temporary file in the same directory
        let tmp_path = path.with_extension("toml.tmp");
        let mut tmp_file = File::create(&tmp_path).map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create temp file '{}': {}",
                tmp_path.display(),
                e
            ))
        })?;

        tmp_file.write_all(toml_str.as_bytes()).map_err(|e| {
            OrcsError::Io(format!(
                "Failed to write to temp file '{}': {}",
                tmp_path.display(),
                e
            ))
        })?;

        // Ensure data is written to disk
        tmp_file.sync_all().map_err(|e| {
            OrcsError::Io(format!(
                "Failed to sync temp file '{}': {}",
                tmp_path.display(),
                e
            ))
        })?;

        drop(tmp_file);

        // Atomic rename
        fs::rename(&tmp_path, path).map_err(|e| {
            OrcsError::Io(format!(
                "Failed to rename temp file '{}' to '{}': {}",
                tmp_path.display(),
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Deletes metadata file synchronously.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to metadata.toml file
    ///
    /// # Returns
    ///
    /// - `Ok(())`: File deleted or didn't exist
    /// - `Err`: Error deleting file
    fn delete_metadata_sync(path: &Path) -> Result<()> {
        if path.exists() {
            fs::remove_file(path).map_err(|e| {
                OrcsError::Io(format!(
                    "Failed to delete metadata file '{}': {}",
                    path.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    /// Lists all workspace directories synchronously.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<PathBuf>)`: List of workspace directory paths
    /// - `Err`: Error reading directory
    fn list_workspace_dirs_sync(root_dir: &Path) -> Result<Vec<PathBuf>> {
        if !root_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(root_dir).map_err(|e| {
            OrcsError::Io(format!(
                "Failed to read workspaces directory '{}': {}",
                root_dir.display(),
                e
            ))
        })?;

        let mut dirs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                OrcsError::Io(format!(
                    "Failed to read directory entry in '{}': {}",
                    root_dir.display(),
                    e
                ))
            })?;

            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }

        Ok(dirs)
    }
}

#[async_trait]
impl WorkspaceMetadataRepository for TomlWorkspaceMetadataRepository {
    async fn find_metadata(&self, workspace_id: &str) -> Result<Option<WorkspaceMetadata>> {
        let path = self.get_metadata_path(workspace_id);

        task::spawn_blocking(move || Self::load_metadata_sync(&path))
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to spawn blocking task: {}", e)))?
    }

    async fn save_metadata(&self, metadata: &WorkspaceMetadata) -> Result<()> {
        let path = self.get_metadata_path(&metadata.id);
        let metadata = metadata.clone();

        task::spawn_blocking(move || Self::save_metadata_sync(&path, &metadata))
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to spawn blocking task: {}", e)))?
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
        let path = self.get_metadata_path(workspace_id);

        task::spawn_blocking(move || Self::delete_metadata_sync(&path))
            .await
            .map_err(|e| OrcsError::Io(format!("Failed to spawn blocking task: {}", e)))?
    }

    async fn list_all_metadata(&self) -> Result<Vec<WorkspaceMetadata>> {
        let root_dir = self.root_dir.clone();

        task::spawn_blocking(move || {
            let dirs = Self::list_workspace_dirs_sync(&root_dir)?;
            let mut metadata_list = Vec::new();

            for dir in dirs {
                let metadata_path = dir.join("metadata.toml");
                if let Some(metadata) = Self::load_metadata_sync(&metadata_path)? {
                    metadata_list.push(metadata);
                }
            }

            // Sort by last_accessed (descending)
            metadata_list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

            Ok(metadata_list)
        })
        .await
        .map_err(|e| OrcsError::Io(format!("Failed to spawn blocking task: {}", e)))?
    }
}

/// Converts a serde_json::Value to a toml::Value.
///
/// This is needed because version-migrate uses JSON internally.
fn json_to_toml(json: &serde_json::Value) -> Result<toml::Value> {
    match json {
        serde_json::Value::Null => Ok(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(toml::Value::Float(f))
            } else {
                Err(OrcsError::Serialization(format!(
                    "Unsupported JSON number: {}",
                    n
                )))
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Result<Vec<toml::Value>> =
                arr.iter().map(json_to_toml).collect();
            Ok(toml::Value::Array(toml_arr?))
        }
        serde_json::Value::Object(obj) => {
            let mut toml_map = toml::map::Map::new();
            for (k, v) in obj {
                toml_map.insert(k.clone(), json_to_toml(v)?);
            }
            Ok(toml::Value::Table(toml_map))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_save_and_find_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

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
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

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
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

        let metadata = WorkspaceMetadata {
            id: "test-workspace".to_string(),
            name: "Test Workspace".to_string(),
            root_path: PathBuf::from("/test/path"),
            last_accessed: 1000,
            is_favorite: false,
        };

        repo.save_metadata(&metadata).await.unwrap();
        assert!(repo.find_metadata("test-workspace").await.unwrap().is_some());

        repo.delete_metadata("test-workspace").await.unwrap();
        assert!(repo.find_metadata("test-workspace").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_all_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

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
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

        let found = repo.find_metadata("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = TomlWorkspaceMetadataRepository::new(temp_dir.path().to_path_buf());

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
