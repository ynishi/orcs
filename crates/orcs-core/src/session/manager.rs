use super::app_mode::AppMode;
use super::model::Session;
use super::repository::SessionRepository;
use crate::error::{OrcsError, Result};
use crate::state::repository::StateRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
use super::app_mode::ConversationMode;

// Forward declaration - orcs-interaction will provide this
// We use dynamic dispatch to avoid circular dependencies
pub trait InteractionManagerTrait: Send + Sync {
    fn session_id(&self) -> &str;
    fn to_session(
        &self,
        app_mode: AppMode,
        workspace_id: String,
    ) -> impl std::future::Future<Output = Session> + Send;
    fn set_workspace_id(
        &self,
        workspace_id: Option<String>,
        workspace_root: Option<std::path::PathBuf>,
    ) -> impl std::future::Future<Output = ()> + Send;
}

/// Manages multiple sessions and their lifecycle.
///
/// `SessionManager` is responsible for:
/// - Creating new sessions
/// - Loading sessions from storage
/// - Switching between sessions
/// - Persisting session state
/// - Managing the active session
pub struct SessionManager<T: InteractionManagerTrait> {
    /// In-memory session cache
    sessions: Arc<RwLock<HashMap<String, Arc<T>>>>,
    /// Persistent storage backend for session data
    session_repository: Arc<dyn SessionRepository>,
    /// Application state repository for global state (e.g., active session ID)
    state_repository: Arc<dyn StateRepository>,
}

impl<T: InteractionManagerTrait + 'static> SessionManager<T> {
    /// Creates a new `SessionManager` with repository backends.
    ///
    /// # Arguments
    ///
    /// * `session_repository` - The repository backend for session data persistence
    /// * `state_repository` - The repository backend for application state (e.g., active session ID)
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        state_repository: Arc<dyn StateRepository>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_repository,
            state_repository,
        }
    }

    /// Attempts to restore the last active session on startup.
    ///
    /// # Returns
    ///
    /// `Some(manager)` if a session was restored, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if storage access fails.
    pub async fn restore_last_session<F>(&self, factory: F) -> Result<Option<Arc<T>>>
    where
        F: FnOnce(Session) -> T,
    {
        // Load active session ID from state repository
        if let Some(session_id) = self.state_repository.get_active_session().await {
            // Try to load session data
            if let Some(session) = self.session_repository.find_by_id(&session_id).await? {
                let manager = Arc::new(factory(session));

                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), manager.clone());

                return Ok(Some(manager));
            }
        }

        Ok(None)
    }

    /// Creates a new session and sets it as active.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the new session
    /// * `factory` - Function to create the InteractionManager
    ///
    /// # Errors
    ///
    /// Returns an error if the active session ID cannot be persisted.
    pub async fn create_session<F>(&self, session_id: String, factory: F) -> Result<Arc<T>>
    where
        F: FnOnce(String) -> T,
    {
        let manager = Arc::new(factory(session_id.clone()));

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), manager.clone());

        // Persist active session ID to state repository
        self.state_repository
            .set_active_session(session_id)
            .await
            .map_err(|e| OrcsError::Internal(e.to_string()))?;

        Ok(manager)
    }

    /// Loads a session from storage and sets it as active.
    ///
    /// # Arguments
    ///
    /// * `data` - The session data to load
    /// * `factory` - Function to create the InteractionManager from Session
    ///
    /// # Errors
    ///
    /// Returns an error if the active session ID cannot be persisted.
    pub async fn load_session<F>(&self, data: Session, factory: F) -> Result<Arc<T>>
    where
        F: FnOnce(Session) -> T,
    {
        let session_id = data.id.clone();
        let manager = Arc::new(factory(data));

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), manager.clone());

        // Persist active session ID to state repository
        self.state_repository
            .set_active_session(session_id)
            .await
            .map_err(|e| OrcsError::Internal(e.to_string()))?;

        Ok(manager)
    }

    /// Returns the currently active session.
    ///
    /// # Returns
    ///
    /// `Some(manager)` if there is an active session, `None` otherwise.
    pub async fn active_session(&self) -> Option<Arc<T>> {
        if let Some(id) = self.state_repository.get_active_session().await {
            let sessions = self.sessions.read().await;
            sessions.get(&id).cloned()
        } else {
            None
        }
    }

    /// Saves the active session to storage.
    ///
    /// # Arguments
    ///
    /// * `app_mode` - The current application mode
    ///
    /// # Errors
    ///
    /// Returns an error if there is no active session or if storage fails.
    pub async fn save_active_session(&self, app_mode: AppMode) -> Result<()> {
        let manager = self
            .active_session()
            .await
            .ok_or_else(|| OrcsError::Internal("No active session".to_string()))?;

        // Load existing session to preserve workspace_id
        // If the session doesn't exist yet (newly created), use placeholder
        let session_id = manager.session_id();
        let existing_workspace_id = self
            .session_repository
            .find_by_id(session_id)
            .await
            .ok()
            .flatten()
            .map(|s| s.workspace_id)
            .unwrap_or_else(|| crate::session::model::PLACEHOLDER_WORKSPACE_ID.to_string());

        let session = manager.to_session(app_mode, existing_workspace_id).await;
        self.session_repository.save(&session).await?;

        Ok(())
    }

    /// Switches to a different session.
    ///
    /// If the session is already loaded in memory, it will be activated immediately.
    /// Otherwise, it will be loaded from storage.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to switch to
    /// * `factory` - Function to create InteractionManager if loading from storage
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be loaded.
    pub async fn switch_session<F>(&self, session_id: &str, factory: F) -> Result<Arc<T>>
    where
        F: FnOnce(Session) -> T,
    {
        let sessions = self.sessions.read().await;

        // Check if already in memory
        if let Some(manager) = sessions.get(session_id) {
            let manager = manager.clone();
            drop(sessions);
            self.state_repository
                .set_active_session(session_id.to_string())
                .await
                .map_err(|e| OrcsError::Internal(e.to_string()))?;
            return Ok(manager);
        }

        drop(sessions);

        // Load from storage
        let session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;
        self.load_session(session, factory).await
    }

    /// Lists all sessions from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage cannot be accessed.
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        self.session_repository.list_all().await
    }

    /// Deletes a session from both memory and storage.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the storage deletion fails.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        // Remove from memory
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        drop(sessions);

        // Remove from storage
        self.session_repository.delete(session_id).await?;

        // Clear active if this was the active session
        if self.state_repository.get_active_session().await.as_ref() == Some(&session_id.to_string()) {
            self.state_repository
                .clear_active_session()
                .await
                .map_err(|e| OrcsError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    /// Updates the workspace ID for a session.
    ///
    /// This method allows changing which workspace a session is associated with.
    /// This is useful when switching workspaces during a session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to update
    /// * `workspace_id` - The new workspace ID (None to clear the association)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The session does not exist
    /// - The storage update fails
    pub async fn update_workspace_id(
        &self,
        session_id: &str,
        workspace_id: String,
        workspace_root: Option<std::path::PathBuf>,
    ) -> Result<()> {
        // Update the in-memory InteractionManager's workspace_id and workspace_root
        let sessions = self.sessions.read().await;
        if let Some(manager) = sessions.get(session_id) {
            manager
                .set_workspace_id(Some(workspace_id.clone()), workspace_root)
                .await;
        }
        drop(sessions);

        // Load existing session from storage
        let mut session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;

        // Update workspace_id
        session.workspace_id = workspace_id;

        // Save back to storage
        self.session_repository.save(&session).await?;

        Ok(())
    }

    /// Returns the ID of the currently active session.
    pub async fn active_session_id(&self) -> Option<String> {
        self.state_repository.get_active_session().await
    }

    /// Renames a session by updating its title.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to rename
    /// * `new_title` - The new title for the session
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist or cannot be saved.
    pub async fn rename_session(&self, session_id: &str, new_title: String) -> Result<()> {
        // Load the session from storage
        let mut session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;

        // Update the title
        session.title = new_title;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.session_repository.save(&session).await?;

        Ok(())
    }

    /// Toggles the favorite status of a session
    pub async fn toggle_favorite(&self, session_id: &str) -> Result<()> {
        // Load the session from storage
        let mut session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;

        // Toggle favorite status
        session.is_favorite = !session.is_favorite;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.session_repository.save(&session).await?;

        Ok(())
    }

    /// Toggles the archive status of a session
    pub async fn toggle_archive(&self, session_id: &str) -> Result<()> {
        // Load the session from storage
        let mut session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;

        // Toggle archive status
        session.is_archived = !session.is_archived;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.session_repository.save(&session).await?;

        Ok(())
    }

    /// Updates the manual sort order of a session
    pub async fn update_sort_order(&self, session_id: &str, sort_order: Option<i32>) -> Result<()> {
        // Load the session from storage
        let mut session = self
            .session_repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| OrcsError::Internal(format!("Session not found: {}", session_id)))?;

        // Update sort order
        session.sort_order = sort_order;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.session_repository.save(&session).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use llm_toolkit::agent::dialogue::ExecutionModel;

    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock InteractionManager for testing
    struct MockInteractionManager {
        session_id: String,
    }

    impl MockInteractionManager {
        fn new(session_id: String) -> Self {
            Self { session_id }
        }

        fn from_data(data: Session) -> Self {
            Self {
                session_id: data.id,
            }
        }
    }

    impl InteractionManagerTrait for MockInteractionManager {
        fn session_id(&self) -> &str {
            &self.session_id
        }

        async fn to_session(&self, app_mode: AppMode, workspace_id: String) -> Session {
            Session {
                id: self.session_id.clone(),
                title: format!("Session {}", self.session_id),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                current_persona_id: "mai".to_string(),
                persona_histories: HashMap::new(),
                app_mode,
                workspace_id,
                active_participant_ids: Vec::new(),
                execution_strategy: ExecutionModel::Broadcast,
                system_messages: Vec::new(),
                participants: HashMap::new(),
                participant_icons: HashMap::new(),
                conversation_mode: ConversationMode::Normal,
                talk_style: None,
                participant_colors: HashMap::new(),
                is_favorite: false,
                is_archived: false,
                sort_order: None,
            }
        }

        async fn set_workspace_id(
            &self,
            _workspace_id: Option<String>,
            _workspace_root: Option<std::path::PathBuf>,
        ) {
            // Mock implementation - no-op
        }
    }

    // Mock SessionRepository for testing
    struct MockSessionRepository {
        sessions: Mutex<HashMap<String, Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.get(session_id).cloned())
        }

        async fn save(&self, session: &Session) -> Result<()> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session.id.clone(), session.clone());
            Ok(())
        }

        async fn delete(&self, session_id: &str) -> Result<()> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id);
            Ok(())
        }

        async fn list_all(&self) -> Result<Vec<Session>> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.values().cloned().collect())
        }
    }

    // Mock StateRepository for testing
    struct MockStateRepository {
        active_session_id: Mutex<Option<String>>,
    }

    impl MockStateRepository {
        fn new() -> Self {
            Self {
                active_session_id: Mutex::new(None),
            }
        }
    }

    #[async_trait::async_trait]
    impl StateRepository for MockStateRepository {
        async fn save_state(&self, _state: crate::state::model::AppState) -> Result<()> {
            Ok(())
        }

        async fn get_state(&self) -> Result<crate::state::model::AppState> {
            Ok(crate::state::model::AppState::default())
        }

        async fn get_last_selected_workspace(&self) -> Option<String> {
            None
        }

        async fn set_last_selected_workspace(&self, _workspace_id: String) -> Result<()> {
            Ok(())
        }

        async fn clear_last_selected_workspace(&self) -> Result<()> {
            Ok(())
        }

        async fn get_default_workspace(&self) -> String {
            crate::state::model::PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string()
        }

        async fn set_default_workspace(&self, _workspace_id: String) -> Result<()> {
            Ok(())
        }

        async fn get_active_session(&self) -> Option<String> {
            self.active_session_id.lock().unwrap().clone()
        }

        async fn set_active_session(&self, session_id: String) -> Result<()> {
            *self.active_session_id.lock().unwrap() = Some(session_id);
            Ok(())
        }

        async fn clear_active_session(&self) -> Result<()> {
            *self.active_session_id.lock().unwrap() = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        let _session = manager
            .create_session("test-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        assert_eq!(
            manager.active_session_id().await,
            Some("test-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository.clone(), state_repository.clone());

        // Create and save
        let _session = manager
            .create_session("test-save".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        // Create new manager and restore
        let manager2: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        let restored = manager2
            .restore_last_session(MockInteractionManager::from_data)
            .await
            .unwrap();

        assert!(restored.is_some());
        assert_eq!(restored.unwrap().session_id(), "test-save");
    }

    #[tokio::test]
    async fn test_switch_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        // Create two sessions
        manager
            .create_session("session-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        manager
            .create_session("session-2".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        // Active should be session-2
        assert_eq!(
            manager.active_session_id().await,
            Some("session-2".to_string())
        );

        // Switch back to session-1
        manager
            .switch_session("session-1", MockInteractionManager::from_data)
            .await
            .unwrap();

        assert_eq!(
            manager.active_session_id().await,
            Some("session-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        manager
            .create_session("to-delete".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.delete_session("to-delete").await.unwrap();

        assert_eq!(manager.active_session_id().await, None);
    }
}
