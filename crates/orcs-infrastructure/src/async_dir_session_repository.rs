//! AsyncDirStorage-based SessionRepository implementation
//!
//! This replaces TomlSessionRepository with a version-migrate AsyncDirStorage-based implementation.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - ~75% code reduction

use crate::dto::create_session_migrator;
use anyhow::{Context, Result};
use async_trait::async_trait;
use orcs_core::repository::{PersonaRepository, SessionRepository};
use orcs_core::session::{ConversationMessage, Session};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// AsyncDirStorage-based session repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// ├── sessions/
/// │   ├── session-id-1.toml
/// │   └── session-id-2.toml
/// └── active_session.txt
/// ```
pub struct AsyncDirSessionRepository {
    storage: AsyncDirStorage,
    base_dir: PathBuf,
    persona_repository: std::sync::Arc<dyn PersonaRepository>,
}

impl AsyncDirSessionRepository {
    /// Creates an AsyncDirSessionRepository instance at the default location (~/.config/orcs).
    ///
    /// # Arguments
    ///
    /// * `persona_repository` - Required for persona ID migration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default_location(
        persona_repository: std::sync::Arc<dyn PersonaRepository>,
    ) -> Result<Self> {
        use crate::paths::OrcsPaths;
        let base_dir = OrcsPaths::config_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get config directory: {}", e))?;
        Self::new(base_dir, persona_repository).await
    }

    /// Creates a new AsyncDirSessionRepository.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for sessions (e.g., ~/.config/orcs)
    /// * `persona_repository` - Required for persona ID migration
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Directory creation fails
    /// - AsyncDirStorage initialization fails
    pub async fn new(
        base_dir: impl AsRef<Path>,
        persona_repository: std::sync::Arc<dyn PersonaRepository>,
    ) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure base directory exists
        fs::create_dir_all(&base_dir)
            .await
            .context("Failed to create base directory")?;

        // Setup AppPaths with CustomBase strategy to use our base_dir
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base_dir.clone()));

        // Setup migrator
        let migrator = create_session_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        let storage = AsyncDirStorage::new(paths, "sessions", migrator, strategy)
            .await
            .context("Failed to create AsyncDirStorage")?;

        Ok(Self {
            storage,
            base_dir,
            persona_repository,
        })
    }

    /// Returns the actual sessions directory path.
    ///
    /// This returns the real path where session files are stored,
    /// which is determined by the AsyncDirStorage's path resolution strategy.
    pub fn sessions_dir(&self) -> &Path {
        self.storage.base_path()
    }

    /// Migrates a persona ID from old format (name) to UUID.
    fn migrate_persona_id(&self, id: &str) -> Result<String> {
        // Check if already a UUID
        if uuid::Uuid::parse_str(id).is_ok() {
            return Ok(id.to_string());
        }

        // Try to find persona by name
        let personas = self
            .persona_repository
            .get_all()
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
        let personas = self
            .persona_repository
            .get_all()
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
                tracing::debug!("Preserved non-persona persona_histories key: '{}'", key);
                key
            };

            migrated.insert(final_key, messages);
        }

        Ok(migrated)
    }

    /// Post-processes a loaded session to migrate persona IDs.
    fn post_process_session(&self, mut session: Session) -> Result<Session> {
        session.persona_histories = self.migrate_persona_history_keys(session.persona_histories)?;
        session.current_persona_id = self.migrate_persona_id(&session.current_persona_id)?;
        Ok(session)
    }
}

#[async_trait]
impl SessionRepository for AsyncDirSessionRepository {
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        match self.storage.load::<Session>("session", session_id).await {
            Ok(session) => {
                let session = self.post_process_session(session)?;
                Ok(Some(session))
            }
            Err(e) => {
                // Check if it's a "not found" error
                let error_str = e.to_string();
                if error_str.contains("No such file or directory")
                    || error_str.contains("not found")
                    || error_str.contains("cannot find")
                {
                    return Ok(None);
                }
                Err(anyhow::anyhow!(e))
            }
        }
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let result = self.storage
            .save("session", &session.id, session)
            .await;
        eprintln!("Save result: {:?}", result);
        result.context("Failed to save session")?;
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.storage
            .delete(session_id)
            .await
            .context("Failed to delete session")?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Session>> {
        let all_sessions = self
            .storage
            .load_all::<Session>("session")
            .await
            .context("Failed to load all sessions")?;

        // Post-process all sessions
        let mut sessions = Vec::new();
        for (_id, session) in all_sessions {
            match self.post_process_session(session) {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    tracing::warn!("Failed to post-process session: {}", e);
                    // Continue loading other sessions
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    async fn get_active_session_id(&self) -> Result<Option<String>> {
        let active_file = self.base_dir.join("active_session.txt");

        if !fs::try_exists(&active_file).await? {
            return Ok(None);
        }

        let session_id = fs::read_to_string(&active_file)
            .await
            .context("Failed to read active session ID")?;

        Ok(Some(session_id.trim().to_string()))
    }

    async fn set_active_session_id(&self, session_id: &str) -> Result<()> {
        let active_file = self.base_dir.join("active_session.txt");
        fs::write(&active_file, session_id)
            .await
            .context("Failed to write active session ID")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::persona::{Persona, PersonaBackend, PersonaSource};
    use orcs_core::session::{AppMode, ConversationMessage, MessageMetadata, MessageRole};
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
                personas: Mutex::new(vec![Persona {
                    id: "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c".to_string(),
                    name: "Mai".to_string(),
                    role: "Engineer".to_string(),
                    background: "".to_string(),
                    communication_style: "".to_string(),
                    default_participant: true,
                    source: PersonaSource::System,
                    backend: PersonaBackend::ClaudeCli,
                    model_name: None,
                }]),
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
                    metadata: MessageMetadata::default(),
                },
                ConversationMessage {
                    role: MessageRole::Assistant,
                    content: "Hi there!".to_string(),
                    timestamp: "2024-01-01T00:00:01Z".to_string(),
                    metadata: MessageMetadata::default(),
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
            active_participant_ids: vec![],
            execution_strategy: "broadcast".to_string(),
            system_messages: vec![],
            participants: HashMap::new(),
            conversation_mode: Default::default(),
            talk_style: None,
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = AsyncDirSessionRepository::new(temp_dir.path(), persona_repo)
            .await
            .unwrap();

        let session = create_test_session("test-session-1");

        // Save
        repository.save(&session).await.unwrap();

        // Find by ID
        let loaded = repository.find_by_id("test-session-1").await.unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.title, session.title);
        // current_persona_id is migrated from "mai" to MockPersonaRepository's UUID
        assert_eq!(
            loaded.current_persona_id,
            "8c6f3e4a-7b2d-5f1e-9a3c-4d8b6e2f1a5c"
        );
    }

    #[tokio::test]
    async fn test_list_all() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = AsyncDirSessionRepository::new(temp_dir.path(), persona_repo)
            .await
            .unwrap();

        // Save multiple sessions
        repository
            .save(&create_test_session("session-1"))
            .await
            .unwrap();
        repository
            .save(&create_test_session("session-2"))
            .await
            .unwrap();
        repository
            .save(&create_test_session("session-3"))
            .await
            .unwrap();

        // List
        let sessions = repository.list_all().await.unwrap();

        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = AsyncDirSessionRepository::new(temp_dir.path(), persona_repo)
            .await
            .unwrap();

        let session = create_test_session("session-to-delete");
        repository.save(&session).await.unwrap();

        // Before delete
        assert!(
            repository
                .find_by_id("session-to-delete")
                .await
                .unwrap()
                .is_some()
        );

        // Delete
        repository.delete("session-to-delete").await.unwrap();

        // After delete
        assert!(
            repository
                .find_by_id("session-to-delete")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_active_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = AsyncDirSessionRepository::new(temp_dir.path(), persona_repo)
            .await
            .unwrap();

        // Initial state
        assert_eq!(repository.get_active_session_id().await.unwrap(), None);

        // Set
        repository
            .set_active_session_id("active-session")
            .await
            .unwrap();

        // Get
        assert_eq!(
            repository.get_active_session_id().await.unwrap(),
            Some("active-session".to_string())
        );
    }

    #[tokio::test]
    async fn test_find_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let persona_repo = Arc::new(MockPersonaRepository::new());
        let repository = AsyncDirSessionRepository::new(temp_dir.path(), persona_repo)
            .await
            .unwrap();

        let result = repository.find_by_id("nonexistent-session").await.unwrap();
        assert!(result.is_none());
    }
}
