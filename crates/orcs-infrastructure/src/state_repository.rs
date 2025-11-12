//! Application state service implementation.
//!
//! This module provides a service for managing application-level state that persists
//! across sessions, such as the last selected workspace ID.

use crate::dto::create_app_state_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::error::{OrcsError, Result};
use orcs_core::state::model::AppState;
use orcs_core::state::repository::StateRepository;
use std::sync::Arc;
use tokio::sync::Mutex;
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior};

/// Service for managing application state.
///
/// This implementation reads and writes application state using FileStorage
/// and caches it in memory to avoid repeated file I/O operations.
///
/// All methods are async to support non-blocking I/O in async contexts.
///
/// # Example
///
/// ```ignore
/// use orcs_infrastructure::app_state_service::AppStateService;
///
/// let service = AppStateService::new().await?;
/// service.set_last_selected_workspace("ws-123".to_string()).await?;
/// let workspace_id = service.get_last_selected_workspace().await;
/// ```
#[derive(Clone)]
pub struct StateRepositoryImpl {
    /// Cached app state loaded from storage.
    /// Uses Mutex for thread-safe access.
    state: Arc<Mutex<AppState>>,
    /// FileStorage instance for persistence.
    /// Wrapped in Mutex for interior mutability.
    storage: Arc<Mutex<FileStorage>>,
}

impl StateRepositoryImpl {
    /// Creates a new AppStateService and loads the initial state.
    ///
    /// This method ensures that the app_state file exists. If it doesn't exist,
    /// it creates the file with default values via LoadBehavior::CreateIfMissing.
    ///
    /// Uses the centralized path management via `ServiceType::AppState`.
    pub async fn new() -> Result<Self> {
        // Get file path for AppState via centralized path management
        let orcs_paths = OrcsPaths::new(None);
        let path_type = orcs_paths.get_path(ServiceType::AppState)?;
        let file_path = path_type.into_path_buf(); // app_state.toml

        // Setup migrator
        let migrator = create_app_state_migrator();

        // Setup storage strategy: TOML format, SaveIfMissing with default value
        let default_state = serde_json::to_value(AppState::default()).map_err(|e| {
            OrcsError::Config(format!("Failed to serialize default AppState: {}", e))
        })?;

        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Json)
            .with_load_behavior(LoadBehavior::SaveIfMissing)
            .with_default_value(default_state);

        // Create FileStorage (automatically loads or creates with default)
        let storage = FileStorage::new(file_path, migrator, strategy)?;

        let storage = Arc::new(Mutex::new(storage));

        // Load initial state from storage
        let initial_state = {
            let storage_lock = storage.lock().await;
            let states: Vec<AppState> = storage_lock.query("app_state")?;
            states.into_iter().next().unwrap_or_default()
        };

        Ok(Self {
            state: Arc::new(Mutex::new(initial_state)),
            storage,
        })
    }
}

#[async_trait::async_trait]
impl StateRepository for StateRepositoryImpl {
    /// Saves the app state to storage.
    async fn save_state(&self, state: AppState) -> Result<()> {
        // Update in-memory cache first
        {
            let mut state_lock = self.state.lock().await;
            *state_lock = state.clone();
        }

        // Save to file storage in blocking context
        let storage = self.storage.clone();
        let state_for_save = state.clone();
        tokio::task::spawn_blocking(move || {
            let mut storage = storage.blocking_lock();
            storage
                .update_and_save("app_state", vec![state_for_save])
                .map_err(|e| OrcsError::internal(format!("Failed to save app_state: {}", e)))
        })
        .await
        .map_err(|e| OrcsError::internal(format!("Failed to join task: {}", e)))??;

        Ok(())
    }

    /// Gets the last selected workspace ID.
    async fn get_last_selected_workspace(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.last_selected_workspace_id.clone()
    }

    /// Sets the last selected workspace ID.
    async fn set_last_selected_workspace(&self, workspace_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.last_selected_workspace_id = Some(workspace_id);
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    /// Clears the last selected workspace ID.
    async fn clear_last_selected_workspace(&self) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.last_selected_workspace_id = None;
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    /// Gets the default workspace ID.
    async fn get_default_workspace(&self) -> String {
        let state = self.state.lock().await;
        state.default_workspace_id.clone()
    }

    /// Sets the default workspace ID.
    async fn set_default_workspace(&self, workspace_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.default_workspace_id = workspace_id;
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    /// Gets the active session ID.
    async fn get_active_session(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.active_session_id.clone()
    }

    /// Sets the active session ID.
    async fn set_active_session(&self, session_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.active_session_id = Some(session_id);
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    /// Clears the active session ID.
    async fn clear_active_session(&self) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.active_session_id = None;
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    async fn get_state(&self) -> Result<AppState> {
        Ok(self.state.lock().await.clone())
    }
}

// Type alias for backward compatibility
pub type AppStateService = StateRepositoryImpl;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_service_creation() {
        let service = AppStateService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_get_last_selected_workspace_default() {
        let service = AppStateService::new().await.unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        // Default should be None
        assert!(workspace_id.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_last_selected_workspace() {
        let service = AppStateService::new().await.unwrap();
        service
            .set_last_selected_workspace("ws-123".to_string())
            .await
            .unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        assert_eq!(workspace_id, Some("ws-123".to_string()));
    }

    #[tokio::test]
    async fn test_clear_last_selected_workspace() {
        let service = AppStateService::new().await.unwrap();
        service
            .set_last_selected_workspace("ws-456".to_string())
            .await
            .unwrap();
        service.clear_last_selected_workspace().await.unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        assert!(workspace_id.is_none());
    }
}
