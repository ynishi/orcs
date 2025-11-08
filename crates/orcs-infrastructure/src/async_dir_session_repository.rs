//! AsyncDirStorage-based SessionRepository implementation
//!
//! This replaces TomlSessionRepository with a version-migrate AsyncDirStorage-based implementation.
//! Benefits:
//! - No manual Migrator management
//! - Built-in ACID guarantees
//! - Fully async I/O (no spawn_blocking)
//! - ~75% code reduction

use crate::dto::create_session_migrator;
use crate::storage_repository::StorageRepository;
use async_trait::async_trait;
use orcs_core::error::Result;
use orcs_core::repository::SessionRepository;
use orcs_core::session::Session;
use std::path::Path;
use version_migrate::AsyncDirStorage;

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
}

impl StorageRepository for AsyncDirSessionRepository {
    const SERVICE_TYPE: crate::paths::ServiceType = crate::paths::ServiceType::Session;
    const ENTITY_NAME: &'static str = "session";

    fn storage(&self) -> &AsyncDirStorage {
        &self.storage
    }
}

impl AsyncDirSessionRepository {
    /// Creates an AsyncDirSessionRepository instance at the default location.
    ///
    /// Uses centralized path management and storage creation via `OrcsPaths`.
    ///
    /// # Arguments
    ///
    /// * `persona_repository` - Required for persona ID migration
    ///
    /// # Errors
    ///
    /// Returns an error if the storage cannot be created.
    pub async fn default(
    ) -> Result<Self> {
        Self::new(None).await
    }

    /// Creates a new AsyncDirSessionRepository with custom base directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for sessions
    /// * `persona_repository` - Required for persona ID migration
    pub async fn new(
        base_dir: Option<&Path>,
    ) -> Result<Self> {
        use crate::paths::OrcsPaths;

        let migrator = create_session_migrator();
        let orcs_paths = OrcsPaths::new(base_dir);
        let storage = orcs_paths
            .create_async_dir_storage(Self::SERVICE_TYPE, migrator)
            .await?;

        Ok(Self {
            storage,
        })
    }
}

#[async_trait]
impl SessionRepository for AsyncDirSessionRepository {
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        match self.storage.load::<Session>(Self::ENTITY_NAME, session_id).await {
            Ok(session) => Ok(Some(session)),
            Err(e) => {
                let orcs_err = e.into();
                tracing::debug!(
                    "find_by_id error for session_id={}: {:?}, is_not_found={}",
                    session_id,
                    orcs_err,
                    orcs_core::OrcsError::is_not_found(&orcs_err)
                );
                if orcs_core::OrcsError::is_not_found(&orcs_err) {
                    Ok(None)
                } else {
                    Err(orcs_err)
                }
            }
        }
    }

    async fn save(&self, session: &Session) -> Result<()> {
        self.storage
            .save(Self::ENTITY_NAME, &session.id, session)
            .await?;
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.storage
            .delete(session_id)
            .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Session>> {
        let mut sessions = self
            .storage
            .load_all::<Session>(Self::ENTITY_NAME)
            .await?
            .into_iter()
            .map(|(_id, session)| session)
            .collect::<Vec<Session>>();

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_toolkit::agent::dialogue::ExecutionModel;
    use orcs_core::persona::{Persona, PersonaBackend, PersonaRepository, PersonaSource};
    use orcs_core::session::{AppMode, ConversationMessage, MessageMetadata, MessageRole};
    use std::collections::HashMap;
    use std::sync::Mutex;
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
                    icon: None,
                    base_color: None,
                }]),
            }
        }
    }

    #[async_trait]
    impl PersonaRepository for MockPersonaRepository {
        async fn get_all(&self) -> Result<Vec<Persona>> {
            Ok(self.personas.lock().unwrap().clone())
        }

        async fn save_all(&self, _configs: &[Persona]) -> Result<()> {
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
            workspace_id: String::new(),
            active_participant_ids: vec![],
            execution_strategy: ExecutionModel::Broadcast,
            system_messages: vec![],
            participants: HashMap::new(),
            participant_icons: HashMap::new(),
            participant_colors: HashMap::new(),
            conversation_mode: Default::default(),
            talk_style: None,
        }
    }

    #[tokio::test]
    async fn test_save_and_find_by_id() {
        let temp_dir = TempDir::new().unwrap();
        let repository = AsyncDirSessionRepository::new(Some(temp_dir.path()))
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
        let repository = AsyncDirSessionRepository::new(Some(temp_dir.path()))
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
        let repository = AsyncDirSessionRepository::new(Some(temp_dir.path()))
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
    async fn test_find_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let repository = AsyncDirSessionRepository::new(Some(temp_dir.path()))
            .await
            .unwrap();

        let result = repository.find_by_id("nonexistent-session").await.unwrap();
        assert!(result.is_none());
    }
}
