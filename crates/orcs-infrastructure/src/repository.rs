use orcs_core::config::PersonaConfig;
use orcs_core::repository::{PersonaRepository, SessionRepository};
use orcs_core::session::Session;
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use crate::dto::SessionV1;

/// A repository implementation for storing persona configurations in a TOML file.
pub struct TomlPersonaRepository;

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<PersonaConfig>, String> {
        crate::toml_storage::load_personas()
    }

    fn save_all(&self, configs: &[PersonaConfig]) -> Result<(), String> {
        crate::toml_storage::save_personas(configs)
    }
}

/// A repository implementation for storing session data in TOML files.
///
/// This implementation follows Clean Architecture principles:
/// - Uses DTOs (SessionV1) for persistence
/// - Handles migration from older formats
/// - Converts between DTOs and domain models
/// - Stores sessions as individual TOML files in a sessions directory
pub struct TomlSessionRepository {
    base_dir: PathBuf,
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
    ///
    /// # Errors
    ///
    /// Returns an error if the directory structure cannot be created.
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Create directory structure
        let sessions_dir = base_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;

        Ok(Self { base_dir })
    }

    /// Creates a `TomlSessionRepository` instance at the default location (~/.orcs).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined or if
    /// the directory structure cannot be created.
    pub fn default_location() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        let base_dir = home_dir.join(".orcs");
        Self::new(base_dir)
    }

    /// Returns the file path for a given session ID.
    fn session_file_path(&self, session_id: &str) -> PathBuf {
        self.base_dir
            .join("sessions")
            .join(format!("{}.toml", session_id))
    }

    /// Loads a session from a specific file path.
    ///
    /// This method handles:
    /// 1. Reading the TOML file
    /// 2. Deserializing to SessionV1 DTO
    /// 3. Converting DTO to domain model
    fn load_session_from_path(&self, path: &Path) -> Result<Session> {
        let toml_content = fs::read_to_string(path)
            .context(format!("Failed to read session file: {:?}", path))?;

        // Deserialize DTO from TOML
        let dto: SessionV1 = toml::from_str(&toml_content)
            .context("Failed to deserialize session data from TOML")?;

        // Convert DTO to domain model using the From trait
        Ok(Session::from(dto))
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

        // Convert domain model to DTO
        let dto: SessionV1 = SessionV1::from(session);

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
    use orcs_types::{AppMode, MessageRole, ConversationMessage};
    use std::collections::HashMap;
    use tempfile::TempDir;

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
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let repository = TomlSessionRepository::new(temp_dir.path()).unwrap();

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
        let repository = TomlSessionRepository::new(temp_dir.path()).unwrap();

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
        let repository = TomlSessionRepository::new(temp_dir.path()).unwrap();

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
        let repository = TomlSessionRepository::new(temp_dir.path()).unwrap();

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
