use super::app_mode::AppMode;
use super::model::Session;
use super::repository::SessionRepository;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Forward declaration - orcs-interaction will provide this
// We use dynamic dispatch to avoid circular dependencies
pub trait InteractionManagerTrait: Send + Sync {
    fn session_id(&self) -> &str;
    fn to_session(
        &self,
        app_mode: AppMode,
        workspace_id: Option<String>,
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
    /// Currently active session ID
    active_session_id: Arc<RwLock<Option<String>>>,
    /// In-memory session cache
    sessions: Arc<RwLock<HashMap<String, Arc<T>>>>,
    /// Persistent storage backend via repository pattern
    repository: Arc<dyn SessionRepository>,
}

impl<T: InteractionManagerTrait + 'static> SessionManager<T> {
    /// Creates a new `SessionManager` with a custom repository backend.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository backend to use for session persistence
    pub fn new(repository: Arc<dyn SessionRepository>) -> Self {
        Self {
            active_session_id: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            repository,
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
        // Load active session ID
        if let Some(session_id) = self.repository.get_active_session_id().await? {
            // Try to load session data
            if let Some(session) = self.repository.find_by_id(&session_id).await? {
                let manager = Arc::new(factory(session));

                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), manager.clone());

                let mut active = self.active_session_id.write().await;
                *active = Some(session_id);

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

        let mut active = self.active_session_id.write().await;
        *active = Some(session_id.clone());

        // Persist active session ID
        self.repository.set_active_session_id(&session_id).await?;

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

        let mut active = self.active_session_id.write().await;
        *active = Some(session_id.clone());

        // Persist active session ID
        self.repository.set_active_session_id(&session_id).await?;

        Ok(manager)
    }

    /// Returns the currently active session.
    ///
    /// # Returns
    ///
    /// `Some(manager)` if there is an active session, `None` otherwise.
    pub async fn active_session(&self) -> Option<Arc<T>> {
        let active_id = self.active_session_id.read().await;
        if let Some(id) = active_id.as_ref() {
            let sessions = self.sessions.read().await;
            sessions.get(id).cloned()
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
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        // Load existing session to preserve workspace_id
        let session_id = manager.session_id();
        let existing_workspace_id = self
            .repository
            .find_by_id(session_id)
            .await?
            .and_then(|s| s.workspace_id);

        let session = manager.to_session(app_mode, existing_workspace_id).await;
        self.repository.save(&session).await?;

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
            let mut active = self.active_session_id.write().await;
            *active = Some(session_id.to_string());
            self.repository.set_active_session_id(session_id).await?;
            return Ok(manager.clone());
        }

        drop(sessions);

        // Load from storage
        let session = self
            .repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
        self.load_session(session, factory).await
    }

    /// Lists all sessions from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage cannot be accessed.
    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        self.repository.list_all().await
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

        // Remove from storage
        self.repository.delete(session_id).await?;

        // Clear active if this was the active session
        let mut active = self.active_session_id.write().await;
        if active.as_ref() == Some(&session_id.to_string()) {
            *active = None;
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
        workspace_id: Option<String>,
        workspace_root: Option<std::path::PathBuf>,
    ) -> Result<()> {
        // Update the in-memory InteractionManager's workspace_id and workspace_root
        let sessions = self.sessions.read().await;
        if let Some(manager) = sessions.get(session_id) {
            manager
                .set_workspace_id(workspace_id.clone(), workspace_root)
                .await;
        }
        drop(sessions);

        // Load existing session from storage
        let mut session = self
            .repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Update workspace_id
        session.workspace_id = workspace_id;

        // Save back to storage
        self.repository.save(&session).await?;

        Ok(())
    }

    /// Returns the ID of the currently active session.
    pub async fn active_session_id(&self) -> Option<String> {
        self.active_session_id.read().await.clone()
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
            .repository
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        // Update the title
        session.title = new_title;

        // Update timestamp
        session.updated_at = chrono::Utc::now().to_rfc3339();

        // Save back to storage
        self.repository.save(&session).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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

        async fn to_session(&self, app_mode: AppMode, workspace_id: Option<String>) -> Session {
            Session {
                id: self.session_id.clone(),
                title: format!("Session {}", self.session_id),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                current_persona_id: "mai".to_string(),
                persona_histories: HashMap::new(),
                app_mode,
                workspace_id,
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
        active_session_id: Mutex<Option<String>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
                active_session_id: Mutex::new(None),
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

        async fn get_active_session_id(&self) -> Result<Option<String>> {
            let active = self.active_session_id.lock().unwrap();
            Ok(active.clone())
        }

        async fn set_active_session_id(&self, session_id: &str) -> Result<()> {
            let mut active = self.active_session_id.lock().unwrap();
            *active = Some(session_id.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let repository = Arc::new(MockSessionRepository::new());
        let manager: SessionManager<MockInteractionManager> = SessionManager::new(repository);

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
        let repository = Arc::new(MockSessionRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(repository.clone());

        // Create and save
        let _session = manager
            .create_session("test-save".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        // Create new manager and restore
        let manager2: SessionManager<MockInteractionManager> = SessionManager::new(repository);

        let restored = manager2
            .restore_last_session(MockInteractionManager::from_data)
            .await
            .unwrap();

        assert!(restored.is_some());
        assert_eq!(restored.unwrap().session_id(), "test-save");
    }

    #[tokio::test]
    async fn test_switch_session() {
        let repository = Arc::new(MockSessionRepository::new());
        let manager: SessionManager<MockInteractionManager> = SessionManager::new(repository);

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
        let repository = Arc::new(MockSessionRepository::new());
        let manager: SessionManager<MockInteractionManager> = SessionManager::new(repository);

        manager
            .create_session("to-delete".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.delete_session("to-delete").await.unwrap();

        assert_eq!(manager.active_session_id().await, None);
    }
}
