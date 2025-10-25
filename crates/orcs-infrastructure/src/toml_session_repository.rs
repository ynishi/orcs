//! TOML-based SessionRepository implementation

use orcs_core::repository::{PersonaRepository, SessionRepository};
use orcs_core::session::{Session, ConversationMessage};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::dto::{SessionV1_0_0, SessionV1_1_0, SessionV2_0_0};
use version_migrate::{MigratesTo, IntoDomain};

/// A repository implementation for storing session data in TOML files.
///
/// This implementation follows Clean Architecture principles:
/// - Uses DTOs (SessionV2_0_0) for persistence
/// - Handles migration from older formats (V1.0.0→V1.1.0→V2.0.0)
/// - Converts between DTOs and domain models
/// - Stores sessions as individual TOML files in a sessions directory
pub struct TomlSessionRepository {
    base_dir: PathBuf,
    persona_repository: std::sync::Arc<dyn PersonaRepository>,
}

impl TomlSessionRepository {
    /// Creates a new `TomlSessionRepository` with the specified base directory.
    ///
    /// The directory structure will be created if it doesn't exist:
    /// ```text
    /// base_dir/
    /// ├── sessions/
    /// │   ├── session-id-1.toml
    /// │   └── session-id-2.toml
    /// └── active_session.txt
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory for storing session data
    /// * `persona_repository` - Required for V0→V1 migration (persona name→UUID resolution)
    ///
    /// # Errors
    ///
    /// Returns an error if the directory structure cannot be created.
    pub fn new(
        base_dir: impl AsRef<Path>,
        persona_repository: std::sync::Arc<dyn PersonaRepository>,
    ) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Create directory structure
        let sessions_dir = base_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;

        Ok(Self {
            base_dir,
            persona_repository,
        })
    }

    /// Creates a `TomlSessionRepository` instance at the default location (~/.orcs).
    ///
    /// # Arguments
    ///
    /// * `persona_repository` - Required for V0→V1 migration
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined or if
    /// the directory structure cannot be created.
    pub fn default_location(
        persona_repository: std::sync::Arc<dyn PersonaRepository>,
    ) -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        let base_dir = home_dir.join(".orcs");
        Self::new(base_dir, persona_repository)
    }

    /// Returns the file path for a given session ID.
    fn session_file_path(&self, session_id: &str) -> PathBuf {
        self.base_dir
            .join("sessions")
            .join(format!("{}.toml", session_id))
    }

    /// Migrates a persona ID from old format (name) to UUID.
    fn migrate_persona_id(&self, id: &str) -> Result<String> {
        // Check if already a UUID
        if uuid::Uuid::parse_str(id).is_ok() {
            return Ok(id.to_string());
        }

        // Try to find persona by name
        let personas = self.persona_repository.get_all()
            .map_err(|e| anyhow::anyhow!("Failed to load personas: {}", e))?;

        let id_lower = id.to_lowercase();
        personas
            .iter()
            .find(|p| p.name.to_lowercase() == id_lower || p.name == id)
            .map(|p| p.id.clone())
            .ok_or_else(|| anyhow::anyhow!("Could not resolve persona ID '{}' to UUID", id))
    }

    /// Migrates persona_histories keys from old IDs/names to UUIDs.
    fn migrate_persona_history_keys(
        &self,
        histories: HashMap<String, Vec<ConversationMessage>>,
    ) -> Result<HashMap<String, Vec<ConversationMessage>>> {
        let personas = self.persona_repository.get_all()
            .map_err(|e| anyhow::anyhow!("Failed to load personas: {}", e))?;

        let mut migrated = HashMap::new();

        for (key, messages) in histories {
            // Check if key is already a valid UUID
            if uuid::Uuid::parse_str(&key).is_ok() {
                migrated.insert(key, messages);
                continue;
            }

            // Try to find persona by old ID or name (case-insensitive)
            let key_lower = key.to_lowercase();
            let matching_persona = personas.iter().find(|p| {
                let name_matches = p.name.to_lowercase() == key_lower || p.name == key;
                name_matches
            });

            let final_key = if let Some(persona) = matching_persona {
                tracing::debug!(
                    "Migrated persona_histories key: '{}' -> '{}' ({})",
                    key,
                    persona.id,
                    persona.name
                );
                persona.id.clone()
            } else {
                // Unknown key - keep as is (might be "user" or other special keys)
                tracing::debug!(
                    "Preserved non-persona persona_histories key: '{}'",
                    key
                );
                key
            };

            migrated.insert(final_key, messages);
        }

        Ok(migrated)
    }

    /// Loads a session from a specific file path.
    ///
    /// This method handles:
    /// 1. Reading the TOML file
    /// 2. Auto-detecting version and migrating if necessary
    /// 3. Converting DTO to domain model
    fn load_session_from_path(&self, path: &Path) -> Result<Session> {
        let toml_content = fs::read_to_string(path)
            .context(format!("Failed to read session file: {:?}", path))?;

        // Try to load as SessionV2_0_0 first (latest version)
        let mut dto: SessionV2_0_0 = if let Ok(v2_0_0) = toml::from_str::<SessionV2_0_0>(&toml_content) {
            v2_0_0
        } else if let Ok(v1_1_0) = toml::from_str::<SessionV1_1_0>(&toml_content) {
            // V1.1.0 format - migrate to V2.0.0
            tracing::info!("Migrating session from V1.1.0 to V2.0.0: {:?}", path);
            v1_1_0.migrate()
        } else if let Ok(v1_0_0) = toml::from_str::<SessionV1_0_0>(&toml_content) {
            // V1.0.0 format - migrate to V1.1.0 then V2.0.0
            tracing::info!("Migrating session from V1.0.0 to V2.0.0: {:?}", path);
            let v1_1_0: SessionV1_1_0 = v1_0_0.migrate();
            v1_1_0.migrate()
        } else {
            return Err(anyhow::anyhow!("Failed to parse session file: {:?}", path));
        };

        // Additional migration: migrate persona_histories keys from names to UUIDs if needed
        dto.persona_histories = self.migrate_persona_history_keys(dto.persona_histories)?;
        dto.current_persona_id = self.migrate_persona_id(&dto.current_persona_id)?;

        // Convert DTO to domain model
        Ok(dto.into_domain())
    }
}

#[async_trait]
impl SessionRepository for TomlSessionRepository {
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        let file_path = self.session_file_path(session_id);

        if !file_path.exists() {
            return Ok(None);
        }

        match self.load_session_from_path(&file_path) {
            Ok(session) => Ok(Some(session)),
            Err(e) => {
                // Check if it's a "not found" error
                if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                    if io_error.kind() == std::io::ErrorKind::NotFound {
                        return Ok(None);
                    }
                }
                Err(e)
            }
        }
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let file_path = self.session_file_path(&session.id);

        // Convert domain model to DTO (always use latest version)
        let dto: SessionV2_0_0 = SessionV2_0_0::from(session);

        // Serialize DTO to TOML
        let toml_content = toml::to_string_pretty(&dto)
            .context("Failed to serialize session data to TOML")?;

        fs::write(&file_path, toml_content)
            .context(format!("Failed to write session file: {:?}", file_path))?;

        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        let file_path = self.session_file_path(session_id);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .context(format!("Failed to delete session file: {:?}", file_path))?;
        }

        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Session>> {
        let sessions_dir = self.base_dir.join("sessions");
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&sessions_dir)
            .context("Failed to read sessions directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(session) = self.load_session_from_path(&path) {
                    sessions.push(session);
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    async fn get_active_session_id(&self) -> Result<Option<String>> {
        let active_file = self.base_dir.join("active_session.txt");

        if !active_file.exists() {
            return Ok(None);
        }

        let session_id = fs::read_to_string(&active_file)
            .context("Failed to read active session ID")?;

        Ok(Some(session_id.trim().to_string()))
    }

    async fn set_active_session_id(&self, session_id: &str) -> Result<()> {
        let active_file = self.base_dir.join("active_session.txt");
        fs::write(&active_file, session_id)
            .context("Failed to write active session ID")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::persona::{Persona, PersonaSource};
    use orcs_core::session::{AppMode, ConversationMessage, MessageRole};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    // Mock PersonaRepository for testing
    struct MockPersonaRepository {
        personas: Mutex<Vec<Persona>>,
    }

    impl MockPersonaRepository {
        fn new() -> Self {
            Self {
                personas: Mutex::new(vec![
                    Persona {
                        id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
                        name: "Mai".to_string(),
                        role: "Engineer".to_string(),
                        background: "".to_string(),
                        communication_style: "".to_string(),
                        default_participant: true,
                        source: PersonaSource::System,
                    },
                ]),
            }
        }
    }

    impl PersonaRepository for MockPersonaRepository {
        fn get_all(&self) -> Result<Vec<Persona>, String> {
            Ok(self.personas.lock().unwrap().clone())
        }

        fn save_all(&self, _configs: &[Persona]) -> Result<(), String> {
            Ok(())
        }
    }

    fn create_test_session(id: &str) -> Session {
        let mut persona_histories = HashMap::new();
        persona_histories.insert(
            "mai".to_string(),
            vec![
                ConversationMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    timestamp: "2024-01-01T00:00:00Z".to_string(),
                },
                ConversationMessage {
                    role: MessageRole::Assistant,
                    content: "Hi there!".to_string(),
                    timestamp: "2024-01-01T00:00:01Z".to_string(),
                },
            ],
        );

        Session {
            id: id.to_string(),
            title: format!("Test Session {}", id),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            current_persona_id: "mai".to_string(),
            persona_histories,
            app_mode: AppMode::Idle,
            workspace_id: None,
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = TomlSessionRepository::new(temp_dir.path(), persona_repo).unwrap();

        let session = create_test_session("test-session-1");

        // Save
        repository.save(&session).await.unwrap();

        // Find by ID
        let loaded = repository.find_by_id("test-session-1").await.unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.title, session.title);
        assert_eq!(loaded.current_persona_id, session.current_persona_id);
    }

    #[tokio::test]
    async fn test_list_all() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = TomlSessionRepository::new(temp_dir.path(), persona_repo).unwrap();

        // Save multiple sessions
        repository.save(&create_test_session("session-1")).await.unwrap();
        repository.save(&create_test_session("session-2")).await.unwrap();
        repository.save(&create_test_session("session-3")).await.unwrap();

        // List
        let sessions = repository.list_all().await.unwrap();

        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = TomlSessionRepository::new(temp_dir.path(), persona_repo).unwrap();

        let session = create_test_session("session-to-delete");
        repository.save(&session).await.unwrap();

        // Before delete
        assert!(repository.find_by_id("session-to-delete").await.unwrap().is_some());

        // Delete
        repository.delete("session-to-delete").await.unwrap();

        // After delete
        assert!(repository.find_by_id("session-to-delete").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_active_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = TomlSessionRepository::new(temp_dir.path(), persona_repo).unwrap();

        // Initial state
        assert_eq!(repository.get_active_session_id().await.unwrap(), None);

        // Set
        repository.set_active_session_id("active-session").await.unwrap();

        // Get
        assert_eq!(
            repository.get_active_session_id().await.unwrap(),
            Some("active-session".to_string())
        );
    }
}
