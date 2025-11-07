//! AsyncDirStorage-based PersonaRepository implementation
//!
//! This replaces TomlPersonaRepository with a version-migrate AsyncDirStorage-based implementation.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - 1 persona = 1 file (scalable for large prompts)

use crate::{dto::create_persona_migrator, storage_repository::StorageRepository};
use crate::OrcsPaths;
use orcs_core::error::Result;
use orcs_core::persona::Persona;
use orcs_core::repository::PersonaRepository;
use std::path::Path;
use version_migrate::AsyncDirStorage;

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
}

impl StorageRepository for AsyncDirPersonaRepository {
    const SERVICE_TYPE: crate::paths::ServiceType = crate::paths::ServiceType::Persona;
    const ENTITY_NAME: &'static str = "persona";

    fn storage(&self) -> &AsyncDirStorage {
        &self.storage
    }
}

impl AsyncDirPersonaRepository {
    pub async fn default() -> Result<Self> {
        Self::new(None).await
    }

    /// Creates a new AsyncDirPersonaRepository with custom base directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for personas
    pub async fn new(base_dir: Option<&Path>) -> Result<Self> {
        let migrator = create_persona_migrator();
        let orcs_paths = OrcsPaths::new(base_dir);
        let storage = orcs_paths
            .create_async_dir_storage(Self::SERVICE_TYPE, migrator)
            .await?;
        Ok(Self { storage })
    }
}

#[async_trait::async_trait]
impl PersonaRepository for AsyncDirPersonaRepository {
    async fn get_all(&self) -> Result<Vec<Persona>> {
        let all_personas = self
            .storage
            .load_all::<Persona>(Self::ENTITY_NAME)
            .await?;

        // Extract values from Vec<(String, Persona)>
        let personas: Vec<Persona> = all_personas.into_iter().map(|(_, p)| p).collect();
        Ok(personas)
    }

    async fn save_all(&self, personas: &[Persona]) -> Result<()> {
        let existing = self
            .storage
            .load_all::<Persona>(Self::ENTITY_NAME)
            .await?;

        let existing_ids: std::collections::HashSet<String> =
            existing.iter().map(|(id, _)| id.clone()).collect();
        let new_ids: std::collections::HashSet<String> =
            personas.iter().map(|p| p.id.clone()).collect();

        // Delete orphaned personas (exist in storage but not in new list)
        for orphaned_id in existing_ids.difference(&new_ids) {
            self.storage.delete(orphaned_id).await?;
        }

        // Save each persona individually (1 persona = 1 file)
        for persona in personas {
            self.storage
                .save(Self::ENTITY_NAME, &persona.id, persona)
                .await?;
        }
        Ok(())
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
        let repo = AsyncDirPersonaRepository::new(Some(temp_dir.path()))
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
            model_name: None,
            icon: None,
            base_color: None,
        };

        // Save
        repo.save_all(&[persona.clone()]).await.unwrap();

        // Load
        let personas = repo.get_all().await.unwrap();
        assert_eq!(personas.len(), 1);
        assert_eq!(personas[0].name, "Test Persona");
        assert_eq!(personas[0].id, persona.id);
    }

    #[tokio::test]
    async fn test_save_multiple_personas() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirPersonaRepository::new(Some(temp_dir.path()))
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
            model_name: None,
            icon: None,
            base_color: None,
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
            model_name: None,
            icon: None,
            base_color: None,
        };

        // Save multiple
        repo.save_all(&[persona1.clone(), persona2.clone()]).await
            .unwrap();

        // Load all
        let personas = repo.get_all().await.unwrap();
        assert_eq!(personas.len(), 2);

        // Verify both exist
        let names: Vec<String> = personas.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"Persona 1".to_string()));
        assert!(names.contains(&"Persona 2".to_string()));
    }

    #[tokio::test]
    async fn test_empty_repository() {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirPersonaRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let personas = repo.get_all().await.unwrap();
        assert_eq!(personas.len(), 0);
    }
}
