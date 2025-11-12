use orcs_core::session::{AppMode, Session, SessionRepository, InteractionManagerTrait};
use orcs_core::error::{OrcsError, Result};
use orcs_core::state::repository::StateRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::updater::SessionUpdater;

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
            .unwrap_or_else(|| orcs_core::session::PLACEHOLDER_WORKSPACE_ID.to_string());

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
            .ok_or_else(|| {
                OrcsError::NotFound {
                    entity_type: "Session",
                    id: session_id.to_string(),
                }
            })?;
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
        if self.state_repository.get_active_session().await.as_ref()
            == Some(&session_id.to_string())
        {
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
    /// * `workspace_id` - The new workspace ID
    /// * `workspace_root` - The workspace root path (optional)
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

        // Update workspace_id in storage using SessionUpdater
        let updater = SessionUpdater::new(self.session_repository.clone());
        updater
            .update(session_id, |session| {
                session.workspace_id = workspace_id;
                Ok(())
            })
            .await?;

        Ok(())
    }

    /// Returns the ID of the currently active session.
    pub async fn active_session_id(&self) -> Option<String> {
        self.state_repository.get_active_session().await
    }
}

#[cfg(test)]
#[path = "manager_test.rs"]
mod tests;

