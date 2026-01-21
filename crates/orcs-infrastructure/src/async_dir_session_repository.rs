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
    pub async fn default() -> Result<Self> {
        Self::new(None).await
    }

    /// Creates a new AsyncDirSessionRepository with custom base directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for sessions
    /// * `persona_repository` - Required for persona ID migration
    pub async fn new(base_dir: Option<&Path>) -> Result<Self> {
        use crate::paths::OrcsPaths;

        let migrator = create_session_migrator();
        let orcs_paths = OrcsPaths::new(base_dir);
        let storage = orcs_paths
            .create_async_dir_storage(Self::SERVICE_TYPE, migrator)
            .await?;

        Ok(Self { storage })
    }

    /// Fallback implementation that loads sessions individually, skipping corrupt files.
    async fn list_all_with_fallback(&self) -> Result<Vec<Session>> {
        use tokio::fs;

        let sessions_dir = self.storage.base_path().join("sessions");

        if !sessions_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(&sessions_dir).await?;
        let mut sessions = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Only process .toml files
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            // Extract session ID from filename (without .toml extension)
            let session_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            // Try to load the session, skip if it fails
            match self
                .storage
                .load::<Session>(Self::ENTITY_NAME, &session_id)
                .await
            {
                Ok(session) => {
                    tracing::debug!(
                        "[AsyncDirSessionRepository] Loaded session via fallback: id={}, title={}",
                        session.id,
                        session.title
                    );
                    sessions.push(session);
                }
                Err(e) => {
                    tracing::warn!(
                        "[AsyncDirSessionRepository] Skipping corrupt session file {}: {:?}",
                        session_id,
                        e
                    );
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        tracing::debug!(
            "[AsyncDirSessionRepository] list_all_with_fallback() returning {} sessions",
            sessions.len()
        );

        Ok(sessions)
    }
}

#[async_trait]
impl SessionRepository for AsyncDirSessionRepository {
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        match self
            .storage
            .load::<Session>(Self::ENTITY_NAME, session_id)
            .await
        {
            Ok(session) => Ok(Some(session)),
            Err(e) => {
                let orcs_err: orcs_core::OrcsError = e.into();
                tracing::debug!(
                    "find_by_id error for session_id={}: {:?}, is_not_found_or_missing={}",
                    session_id,
                    orcs_err,
                    orcs_err.is_not_found_or_missing()
                );
                // Check if it's a NotFound error or an IO error indicating file not found
                if orcs_err.is_not_found_or_missing() {
                    Ok(None)
                } else {
                    Err(orcs_err)
                }
            }
        }
    }

    async fn save(&self, session: &Session) -> Result<()> {
        tracing::debug!(
            "[AsyncDirSessionRepository] save() called: id={}, title={}, is_favorite={}",
            session.id,
            session.title,
            session.is_favorite
        );
        self.storage
            .save(Self::ENTITY_NAME, &session.id, session)
            .await?;
        tracing::debug!(
            "[AsyncDirSessionRepository] save() completed: id={}",
            session.id
        );
        Ok(())
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.storage.delete(session_id).await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Session>> {
        // Try the fast path first using load_all
        match self.storage.load_all::<Session>(Self::ENTITY_NAME).await {
            Ok(sessions_with_ids) => {
                tracing::debug!(
                    "[AsyncDirSessionRepository] list_all() loaded {} sessions from storage",
                    sessions_with_ids.len()
                );

                let mut sessions: Vec<Session> = sessions_with_ids
                    .into_iter()
                    .map(|(file_id, session)| {
                        tracing::debug!(
                            "[AsyncDirSessionRepository] Loaded session: file_id={}, session.id={}, title={}, is_favorite={}",
                            file_id,
                            session.id,
                            session.title,
                            session.is_favorite
                        );
                        session
                    })
                    .collect();

                // Sort by updated_at descending (most recent first)
                sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                tracing::debug!(
                    "[AsyncDirSessionRepository] list_all() returning {} sessions",
                    sessions.len()
                );

                Ok(sessions)
            }
            Err(e) => {
                // If load_all fails (e.g., one corrupt file), fall back to individual loading
                tracing::warn!(
                    "[AsyncDirSessionRepository] load_all failed: {:?}, falling back to individual loading",
                    e
                );
                self.list_all_with_fallback().await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_toolkit::agent::dialogue::ExecutionModel;
    use orcs_core::session::{AppMode, ConversationMessage, MessageMetadata, MessageRole};
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
                    metadata: MessageMetadata::default(),
                    attachments: vec![],
                },
                ConversationMessage {
                    role: MessageRole::Assistant,
                    content: "Hi there!".to_string(),
                    timestamp: "2024-01-01T00:00:01Z".to_string(),
                    metadata: MessageMetadata::default(),
                    attachments: vec![],
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
            participant_backends: HashMap::new(),
            participant_models: HashMap::new(),
            conversation_mode: Default::default(),
            talk_style: None,
            is_favorite: false,
            is_archived: false,
            sort_order: None,
            auto_chat_config: None,
            is_muted: false,
            context_mode: Default::default(),
            sandbox_state: None,
            last_memory_sync_at: None,
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
        // current_persona_id remains as "mai" (persona migration is handled elsewhere)
        assert_eq!(loaded.current_persona_id, "mai");
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
