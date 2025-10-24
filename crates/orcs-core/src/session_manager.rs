use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use orcs_types::{SessionData, AppMode};
use crate::session_storage::SessionStorage;

// Forward declaration - orcs-interaction will provide this
// We use dynamic dispatch to avoid circular dependencies
pub trait InteractionManagerTrait: Send + Sync {
    fn session_id(&self) -> &str;
    fn to_session_data(&self, app_mode: AppMode) -> impl std::future::Future<Output = SessionData> + Send;
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
    /// Persistent storage backend
    storage: Arc<SessionStorage>,
}

impl<T: InteractionManagerTrait + 'static> SessionManager<T> {
    /// Creates a new `SessionManager` with default storage location.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created.
    pub fn new() -> Result<Self> {
        let storage = SessionStorage::default_location()?;
        Ok(Self::with_storage(storage))
    }

    /// Creates a new `SessionManager` with a custom storage backend.
    ///
    /// # Arguments
    ///
    /// * `storage` - The storage backend to use
    pub fn with_storage(storage: SessionStorage) -> Self {
        Self {
            active_session_id: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage: Arc::new(storage),
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
        F: FnOnce(SessionData) -> T,
    {
        // Load active session ID
        if let Some(session_id) = self.storage.load_active_session_id()? {
            // Try to load session data
            if let Ok(session_data) = self.storage.load_session(&session_id) {
                let manager = Arc::new(factory(session_data));

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
        self.storage.save_active_session_id(&session_id)?;

        Ok(manager)
    }

    /// Loads a session from storage and sets it as active.
    ///
    /// # Arguments
    ///
    /// * `data` - The session data to load
    /// * `factory` - Function to create the InteractionManager from SessionData
    ///
    /// # Errors
    ///
    /// Returns an error if the active session ID cannot be persisted.
    pub async fn load_session<F>(&self, data: SessionData, factory: F) -> Result<Arc<T>>
    where
        F: FnOnce(SessionData) -> T,
    {
        let session_id = data.id.clone();
        let manager = Arc::new(factory(data));

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), manager.clone());

        let mut active = self.active_session_id.write().await;
        *active = Some(session_id.clone());

        // Persist active session ID
        self.storage.save_active_session_id(&session_id)?;

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
        let manager = self.active_session()
            .await
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        let session_data = manager.to_session_data(app_mode).await;
        self.storage.save_session(&session_data)?;

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
        F: FnOnce(SessionData) -> T,
    {
        let sessions = self.sessions.read().await;

        // Check if already in memory
        if let Some(manager) = sessions.get(session_id) {
            let mut active = self.active_session_id.write().await;
            *active = Some(session_id.to_string());
            self.storage.save_active_session_id(session_id)?;
            return Ok(manager.clone());
        }

        drop(sessions);

        // Load from storage
        let session_data = self.storage.load_session(session_id)?;
        self.load_session(session_data, factory).await
    }

    /// Lists all sessions from storage.
    ///
    /// # Errors
    ///
    /// Returns an error if the storage cannot be accessed.
    pub fn list_sessions(&self) -> Result<Vec<SessionData>> {
        self.storage.list_sessions()
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
        self.storage.delete_session(session_id)?;

        // Clear active if this was the active session
        let mut active = self.active_session_id.write().await;
        if active.as_ref() == Some(&session_id.to_string()) {
            *active = None;
        }

        Ok(())
    }

    /// Returns the ID of the currently active session.
    pub async fn active_session_id(&self) -> Option<String> {
        self.active_session_id.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // Mock InteractionManager for testing
    struct MockInteractionManager {
        session_id: String,
    }

    impl MockInteractionManager {
        fn new(session_id: String) -> Self {
            Self { session_id }
        }

        fn from_data(data: SessionData) -> Self {
            Self {
                session_id: data.id,
            }
        }
    }

    impl InteractionManagerTrait for MockInteractionManager {
        fn session_id(&self) -> &str {
            &self.session_id
        }

        async fn to_session_data(&self, app_mode: AppMode) -> SessionData {
            SessionData {
                id: self.session_id.clone(),
                name: format!("Session {}", self.session_id),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                current_persona_id: "mai".to_string(),
                persona_histories: HashMap::new(),
                app_mode,
            }
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();
        let manager: SessionManager<MockInteractionManager> = SessionManager::with_storage(storage);

        let _session = manager
            .create_session("test-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        assert_eq!(manager.active_session_id().await, Some("test-1".to_string()));
        assert_eq!(manager.active_session_id().await, Some("test-1".to_string()));
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();
        let manager: SessionManager<MockInteractionManager> = SessionManager::with_storage(storage);

        // Create and save
        let session = manager
            .create_session("test-save".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        // Create new manager and restore
        let storage2 = SessionStorage::new(temp_dir.path()).unwrap();
        let manager2: SessionManager<MockInteractionManager> = SessionManager::with_storage(storage2);

        let restored = manager2
            .restore_last_session(MockInteractionManager::from_data)
            .await
            .unwrap();

        assert!(restored.is_some());
        assert_eq!(restored.unwrap().session_id(), "test-save");
    }

    #[tokio::test]
    async fn test_switch_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();
        let manager: SessionManager<MockInteractionManager> = SessionManager::with_storage(storage);

        // Create two sessions
        manager
            .create_session("session-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager
            .create_session("session-2".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        // Active should be session-2
        assert_eq!(manager.active_session_id().await, Some("session-2".to_string()));

        // Switch back to session-1
        manager
            .switch_session("session-1", MockInteractionManager::from_data)
            .await
            .unwrap();

        assert_eq!(manager.active_session_id().await, Some("session-1".to_string()));
    }

    #[tokio::test]
    async fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();
        let manager: SessionManager<MockInteractionManager> = SessionManager::with_storage(storage);

        manager
            .create_session("to-delete".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.delete_session("to-delete").await.unwrap();

        assert_eq!(manager.active_session_id().await, None);
    }
}
