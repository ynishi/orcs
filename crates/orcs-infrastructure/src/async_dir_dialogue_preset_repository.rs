//! AsyncDirStorage-based DialoguePresetRepository implementation
//!
//! This provides a version-migrate AsyncDirStorage-based implementation for dialogue presets.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - 1 preset = 1 file (scalable)
//!
//! Directory structure:
//! ```text
//! base_dir/
//! └── dialogue_presets/
//!     ├── <preset-id-1>.toml
//!     ├── <preset-id-2>.toml
//!     └── <preset-id-3>.toml
//! ```
//!
//! # System vs User Presets
//!
//! System presets are provided by `get_default_presets()` and are merged
//! with user-created presets from storage. System presets cannot be modified
//! or deleted.

use crate::OrcsPaths;
use crate::dto::create_dialogue_preset_migrator;
use crate::storage_repository::StorageRepository;
use orcs_core::dialogue::{
    DialoguePreset, DialoguePresetRepository, PresetSource, get_default_presets,
};
use orcs_core::error::Result;
use std::path::Path;
use version_migrate::AsyncDirStorage;

/// AsyncDirStorage-based dialogue preset repository.
pub struct AsyncDirDialoguePresetRepository {
    storage: AsyncDirStorage,
}

impl StorageRepository for AsyncDirDialoguePresetRepository {
    const SERVICE_TYPE: crate::paths::ServiceType = crate::paths::ServiceType::DialoguePreset;
    const ENTITY_NAME: &'static str = "dialogue_preset";

    fn storage(&self) -> &AsyncDirStorage {
        &self.storage
    }
}

impl AsyncDirDialoguePresetRepository {
    /// Creates an AsyncDirDialoguePresetRepository instance at the default location.
    pub async fn default() -> Result<Self> {
        Self::new(None).await
    }

    /// Creates a new AsyncDirDialoguePresetRepository with custom base directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for dialogue presets
    pub async fn new(base_dir: Option<&Path>) -> Result<Self> {
        let migrator = create_dialogue_preset_migrator();
        let orcs_paths = OrcsPaths::new(base_dir);
        let storage = orcs_paths
            .create_async_dir_storage(Self::SERVICE_TYPE, migrator)
            .await?;
        Ok(Self { storage })
    }

    /// Helper to check if a preset is a system preset (cannot be modified/deleted).
    fn is_system_preset(preset_id: &str) -> bool {
        preset_id.starts_with("preset-")
    }
}

#[async_trait::async_trait]
impl DialoguePresetRepository for AsyncDirDialoguePresetRepository {
    async fn find_by_id(&self, preset_id: &str) -> Result<Option<DialoguePreset>> {
        // Check system presets first
        if let Some(system_preset) = get_default_presets()
            .into_iter()
            .find(|p| p.id == preset_id)
        {
            return Ok(Some(system_preset));
        }

        // Then check user presets from storage
        match self
            .storage
            .load::<DialoguePreset>(Self::ENTITY_NAME, preset_id)
            .await
        {
            Ok(preset) => Ok(Some(preset)),
            Err(e) => {
                let orcs_err: orcs_core::OrcsError = e.into();
                // Check if it's a NotFound error or an IO error with "File not found" message
                if orcs_err.is_not_found()
                    || (orcs_err.is_io() && orcs_err.to_string().contains("File not found"))
                {
                    Ok(None)
                } else {
                    Err(orcs_err)
                }
            }
        }
    }

    async fn save(&self, preset: &DialoguePreset) -> Result<()> {
        // Prevent saving/modifying system presets
        if preset.source == PresetSource::System || Self::is_system_preset(&preset.id) {
            return Err(orcs_core::OrcsError::config(
                "Cannot save system presets. System presets are read-only.",
            ));
        }

        self.storage
            .save(Self::ENTITY_NAME, &preset.id, preset)
            .await?;
        Ok(())
    }

    async fn delete(&self, preset_id: &str) -> Result<()> {
        // Prevent deleting system presets
        if Self::is_system_preset(preset_id) {
            return Err(orcs_core::OrcsError::config(
                "Cannot delete system presets. System presets are read-only.",
            ));
        }

        self.storage.delete(preset_id).await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<DialoguePreset>> {
        // Get system presets
        let mut all_presets = get_default_presets();

        // Get user presets from storage
        let user_presets_with_ids = self
            .storage
            .load_all::<DialoguePreset>(Self::ENTITY_NAME)
            .await?;

        // Extract user presets and append to result
        let user_presets: Vec<DialoguePreset> =
            user_presets_with_ids.into_iter().map(|(_, p)| p).collect();

        all_presets.extend(user_presets);

        Ok(all_presets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
    use orcs_core::session::ConversationMode;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_get_all_includes_system_presets() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let all_presets = repo.get_all().await.unwrap();

        // Should have at least the system presets
        assert!(
            all_presets.len() >= 7,
            "Should have at least 7 system presets"
        );

        // Verify system presets are included
        let has_brainstorm = all_presets.iter().any(|p| p.id == "preset-brainstorm");
        assert!(has_brainstorm, "Should include brainstorm system preset");
    }

    #[tokio::test]
    async fn test_find_system_preset() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let preset = repo.find_by_id("preset-brainstorm").await.unwrap();
        assert!(preset.is_some(), "Should find system preset");

        let preset = preset.unwrap();
        assert_eq!(preset.name, "アイデア出し");
        assert_eq!(preset.source, PresetSource::System);
    }

    #[tokio::test]
    async fn test_save_user_preset() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let user_preset = DialoguePreset {
            id: uuid::Uuid::new_v4().to_string(),
            name: "My Custom Preset".to_string(),
            icon: Some("🎯".to_string()),
            description: Some("Custom preset description".to_string()),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::Casual),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::User,
        };

        // Save user preset
        repo.save(&user_preset).await.unwrap();

        // Verify it was saved
        let loaded = repo.find_by_id(&user_preset.id).await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "My Custom Preset");
        assert_eq!(loaded.source, PresetSource::User);
    }

    #[tokio::test]
    async fn test_cannot_save_system_preset() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let system_preset = DialoguePreset {
            id: "preset-brainstorm".to_string(),
            name: "Modified Brainstorm".to_string(),
            icon: Some("💡".to_string()),
            description: Some("Trying to modify system preset".to_string()),
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Brief,
            talk_style: Some(TalkStyle::Brainstorm),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::System,
        };

        // Attempt to save system preset should fail
        let result = repo.save(&system_preset).await;
        assert!(result.is_err(), "Should not allow saving system presets");
    }

    #[tokio::test]
    async fn test_cannot_delete_system_preset() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        // Attempt to delete system preset should fail
        let result = repo.delete("preset-brainstorm").await;
        assert!(result.is_err(), "Should not allow deleting system presets");
    }

    #[tokio::test]
    async fn test_delete_user_preset() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let user_preset = DialoguePreset {
            id: uuid::Uuid::new_v4().to_string(),
            name: "To Delete".to_string(),
            icon: Some("🗑️".to_string()),
            description: None,
            execution_strategy: ExecutionModel::Sequential,
            conversation_mode: ConversationMode::Normal,
            talk_style: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::User,
        };

        // Save
        repo.save(&user_preset).await.unwrap();

        // Verify exists
        assert!(repo.find_by_id(&user_preset.id).await.unwrap().is_some());

        // Delete
        repo.delete(&user_preset.id).await.unwrap();

        // Verify deleted
        assert!(repo.find_by_id(&user_preset.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_all_combines_system_and_user_presets() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirDialoguePresetRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        // Save a user preset
        let user_preset = DialoguePreset {
            id: uuid::Uuid::new_v4().to_string(),
            name: "User Preset".to_string(),
            icon: Some("👤".to_string()),
            description: None,
            execution_strategy: ExecutionModel::Broadcast,
            conversation_mode: ConversationMode::Concise,
            talk_style: Some(TalkStyle::Planning),
            created_at: chrono::Utc::now().to_rfc3339(),
            source: PresetSource::User,
        };
        repo.save(&user_preset).await.unwrap();

        // Get all presets
        let all_presets = repo.get_all().await.unwrap();

        // Should have system presets + user preset
        assert!(
            all_presets.len() >= 8,
            "Should have at least 7 system + 1 user preset"
        );

        // Verify both types are included
        let has_system = all_presets.iter().any(|p| p.source == PresetSource::System);
        let has_user = all_presets.iter().any(|p| p.source == PresetSource::User);
        assert!(has_system, "Should include system presets");
        assert!(has_user, "Should include user presets");
    }
}
