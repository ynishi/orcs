//! AsyncDirStorage-based PersonaRepository implementation
//!
//! This replaces TomlPersonaRepository with a version-migrate AsyncDirStorage-based implementation.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - 1 persona = 1 file (scalable for large prompts)

use crate::dto::create_persona_migrator;
use anyhow::{Context, Result};
use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;
use std::path::{Path, PathBuf};
use tokio::fs;
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// AsyncDirStorage-based persona repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// └── personas/
///     ├── <uuid-1>.toml
///     ├── <uuid-2>.toml
///     └── <uuid-3>.toml
/// ```
pub struct AsyncDirPersonaRepository {
    storage: AsyncDirStorage,
    #[allow(dead_code)]
    base_dir: PathBuf,
}

impl AsyncDirPersonaRepository {
    /// Creates an AsyncDirPersonaRepository instance at the default location (~/.config/orcs).
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default_location() -> Result<Self> {
        use crate::paths::OrcsPaths;
        let base_dir = OrcsPaths::config_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get config directory: {}", e))?;
        Self::new(base_dir).await
    }

    /// Creates a new AsyncDirPersonaRepository.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for personas (e.g., ~/.config/orcs)
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
            .context("Failed to create base directory")?;

        // Setup AppPaths with CustomBase strategy to use our base_dir
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base_dir.clone()));

        // Setup migrator
        let migrator = create_persona_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        let storage = AsyncDirStorage::new(paths, "personas", migrator, strategy)
            .await
            .context("Failed to create AsyncDirStorage")?;

        Ok(Self { storage, base_dir })
    }

    /// Returns the actual config file path.
    ///
    /// This returns the real path where the config.toml is stored.
    pub fn config_file_path(&self) -> PathBuf {
        self.base_dir.join("config.toml")
    }

    /// Returns the actual personas directory path.
    ///
    /// This returns the real path where persona files are stored,
    /// which is determined by the AsyncDirStorage's path resolution strategy.
    pub fn personas_dir(&self) -> &Path {
        self.storage.base_path()
    }
}

impl PersonaRepository for AsyncDirPersonaRepository {
    fn get_all(&self) -> Result<Vec<Persona>, String> {
        // Block on async operation (since trait is sync)
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let all_personas = self
                    .storage
                    .load_all::<Persona>("persona")
                    .await
                    .map_err(|e| format!("Failed to load all personas: {}", e))?;

                // Extract values from Vec<(String, Persona)>
                let personas: Vec<Persona> = all_personas.into_iter().map(|(_, p)| p).collect();
                Ok(personas)
            })
        })
    }

    fn save_all(&self, personas: &[Persona]) -> Result<(), String> {
        // Block on async operation (since trait is sync)
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // 1. Load all existing personas to identify orphaned files
                let existing = self
                    .storage
                    .load_all::<Persona>("persona")
                    .await
                    .map_err(|e| format!("Failed to load existing personas: {}", e))?;

                let existing_ids: std::collections::HashSet<String> =
                    existing.iter().map(|(id, _)| id.clone()).collect();
                let new_ids: std::collections::HashSet<String> =
                    personas.iter().map(|p| p.id.clone()).collect();

                // 2. Delete orphaned personas (exist in storage but not in new list)
                for orphaned_id in existing_ids.difference(&new_ids) {
                    self.storage
                        .delete(orphaned_id)
                        .await
                        .map_err(|e| {
                            format!("Failed to delete orphaned persona {}: {}", orphaned_id, e)
                        })?;
                }

                // 3. Save each persona individually (1 persona = 1 file)
                for persona in personas {
                    self.storage
                        .save("persona", &persona.id, persona)
                        .await
                        .map_err(|e| format!("Failed to save persona {}: {}", persona.id, e))?;
                }
                Ok(())
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::persona::{PersonaBackend, PersonaSource};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load_personas() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirPersonaRepository::new(temp_dir.path())
            .await
            .unwrap();

        let persona = Persona {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Test Persona".to_string(),
            role: "Tester".to_string(),
            background: "Test background".to_string(),
            communication_style: "Test style".to_string(),
            default_participant: true,
            source: PersonaSource::User,
            backend: PersonaBackend::ClaudeCli,
        };

        // Save
        repo.save_all(&[persona.clone()]).unwrap();

        // Load
        let personas = repo.get_all().unwrap();
        assert_eq!(personas.len(), 1);
        assert_eq!(personas[0].name, "Test Persona");
        assert_eq!(personas[0].id, persona.id);
    }

    #[tokio::test]
    async fn test_save_multiple_personas() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirPersonaRepository::new(temp_dir.path())
            .await
            .unwrap();

        let persona1 = Persona {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Persona 1".to_string(),
            role: "Role 1".to_string(),
            background: "Background 1".to_string(),
            communication_style: "Style 1".to_string(),
            default_participant: true,
            source: PersonaSource::System,
            backend: PersonaBackend::ClaudeCli,
        };

        let persona2 = Persona {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Persona 2".to_string(),
            role: "Role 2".to_string(),
            background: "Background 2".to_string(),
            communication_style: "Style 2".to_string(),
            default_participant: false,
            source: PersonaSource::User,
            backend: PersonaBackend::GeminiCli,
        };

        // Save multiple
        repo.save_all(&[persona1.clone(), persona2.clone()])
            .unwrap();

        // Load all
        let personas = repo.get_all().unwrap();
        assert_eq!(personas.len(), 2);

        // Verify both exist
        let names: Vec<String> = personas.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"Persona 1".to_string()));
        assert!(names.contains(&"Persona 2".to_string()));
    }

    #[tokio::test]
    async fn test_empty_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirPersonaRepository::new(temp_dir.path())
            .await
            .unwrap();

        let personas = repo.get_all().unwrap();
        assert_eq!(personas.len(), 0);
    }
}
