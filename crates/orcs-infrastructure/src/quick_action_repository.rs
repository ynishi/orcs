//! Quick Action repository implementation.
//!
//! Stores quick action configuration as versioned JSON files within workspace directories.
//! Uses version-migrate for automatic schema migration.
//!
//! File location: `{workspace_dir}/quick_actions.json`

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use version_migrate::Migrator;

use orcs_core::OrcsError;
use orcs_core::error::Result;
use orcs_core::quick_action::{QuickActionConfig, QuickActionRepository};

use crate::dto::create_quick_action_migrator;
use crate::paths::{OrcsPaths, PathType, ServiceType};

/// File-based quick action repository with version migration support.
///
/// Stores configuration in `{workspace_storage_dir}/{workspace_id}/quick_actions.json`.
pub struct FileQuickActionRepository {
    /// Base path for workspace storage.
    workspace_storage_base: PathBuf,
    /// Migrator for version handling.
    migrator: Migrator,
}

impl FileQuickActionRepository {
    const CONFIG_FILENAME: &'static str = "quick_actions.json";

    /// Creates a new FileQuickActionRepository with default paths.
    pub async fn new() -> Result<Self> {
        let orcs_paths = OrcsPaths::new(None);
        let path_type = orcs_paths
            .get_path(ServiceType::WorkspaceStorage)
            .map_err(|e| OrcsError::Config(e.to_string()))?;

        let workspace_storage_base = match path_type {
            PathType::Dir(p) => p,
            PathType::File(p) => p.parent().unwrap_or(&p).to_path_buf(),
        };

        let migrator = create_quick_action_migrator();

        Ok(Self {
            workspace_storage_base,
            migrator,
        })
    }

    /// Creates a new FileQuickActionRepository with a custom base path (for testing).
    pub fn with_base_path(workspace_storage_base: PathBuf) -> Self {
        Self {
            workspace_storage_base,
            migrator: create_quick_action_migrator(),
        }
    }

    /// Returns the path to the quick actions config file for a workspace.
    fn config_path(&self, workspace_id: &str) -> PathBuf {
        self.workspace_storage_base
            .join(workspace_id)
            .join(Self::CONFIG_FILENAME)
    }
}

#[async_trait]
impl QuickActionRepository for FileQuickActionRepository {
    async fn load(&self, workspace_id: &str) -> Result<QuickActionConfig> {
        let path = self.config_path(workspace_id);

        if !path.exists() {
            // Return default config if file doesn't exist
            return Ok(QuickActionConfig::default());
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(|e| OrcsError::io(format!("Failed to read quick actions config: {}", e)))?;

        // Parse JSON and migrate to latest version
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| OrcsError::Config(format!("Failed to parse quick actions JSON: {}", e)))?;

        let config: QuickActionConfig = self
            .migrator
            .load_flat_from("quick_action", json_value)
            .map_err(|e| {
                OrcsError::Migration(format!("Failed to migrate quick actions config: {}", e))
            })?;

        Ok(config)
    }

    async fn save(&self, workspace_id: &str, config: &QuickActionConfig) -> Result<()> {
        let path = self.config_path(workspace_id);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| OrcsError::io(format!("Failed to create directory: {}", e)))?;
        }

        // Serialize using migrator (includes version info)
        let serialized = self
            .migrator
            .save_domain_flat("quick_action", config.clone())
            .map_err(|e| {
                OrcsError::Config(format!("Failed to serialize quick actions config: {}", e))
            })?;

        fs::write(&path, serialized)
            .await
            .map_err(|e| OrcsError::io(format!("Failed to write quick actions config: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_default_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileQuickActionRepository::with_base_path(temp_dir.path().to_path_buf());

        let config = repo.load("test-workspace").await.unwrap();
        assert_eq!(config.slots.len(), 10);
        assert!(config.slots.iter().all(|s| s.command_name.is_none()));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileQuickActionRepository::with_base_path(temp_dir.path().to_path_buf());

        let mut config = QuickActionConfig::default();
        config.set_slot_command("A", Some("summary".to_string()));
        config.set_slot_command("B", Some("review".to_string()));

        repo.save("test-workspace", &config).await.unwrap();

        let loaded = repo.load("test-workspace").await.unwrap();
        assert_eq!(
            loaded.get_slot("A").unwrap().command_name,
            Some("summary".to_string())
        );
        assert_eq!(
            loaded.get_slot("B").unwrap().command_name,
            Some("review".to_string())
        );
        assert_eq!(loaded.get_slot("C").unwrap().command_name, None);
    }
}
