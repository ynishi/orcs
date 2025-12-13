//! Application state service implementation.
//!
//! This module provides a service for managing application-level state that persists
//! across sessions, such as the last selected workspace ID.

use crate::dto::create_app_state_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::error::{OrcsError, Result};
use orcs_core::state::model::{AppState, OpenTab};
use orcs_core::state::repository::StateRepository;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
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
        Self::with_base_dir(None).await
    }

    /// Creates a new AppStateService with a custom base directory (for testing).
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Optional base directory. If None, uses the default system path.
    pub async fn with_base_dir(base_dir: Option<&std::path::Path>) -> Result<Self> {
        use std::fs;

        // Get file path for AppState via centralized path management
        let orcs_paths = OrcsPaths::new(base_dir);
        let path_type = orcs_paths.get_path(ServiceType::AppState)?;
        let file_path = path_type.into_path_buf(); // app_state.json

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| OrcsError::io(format!("Failed to create parent directory: {}", e)))?;
        }

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
    async fn get_default_workspace(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.default_workspace_id.clone()
    }

    /// Sets the default workspace ID.
    async fn set_default_workspace(&self, workspace_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();
        state.default_workspace_id = Some(workspace_id);
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

    // Tab management methods
    async fn get_open_tabs(&self) -> Vec<OpenTab> {
        let state = self.state.lock().await;
        state.open_tabs.clone()
    }

    async fn get_active_tab_id(&self) -> Option<String> {
        let state = self.state.lock().await;
        state.active_tab_id.clone()
    }

    async fn open_tab(&self, session_id: String, workspace_id: String) -> Result<String> {
        let mut state = self.state.lock().await.clone();

        // Check if tab for this session already exists
        if let Some(existing_tab) = state
            .open_tabs
            .iter()
            .find(|tab| tab.session_id == session_id)
        {
            // Update last_accessed_at and set as active
            let tab_id = existing_tab.id.clone();
            state.open_tabs.iter_mut().for_each(|tab| {
                if tab.id == tab_id {
                    tab.last_accessed_at = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i32;
                }
            });
            state.active_tab_id = Some(tab_id.clone());
            let cloned_state = state.clone();
            drop(state);
            self.save_state(cloned_state).await?;
            return Ok(tab_id);
        }

        // Create new tab
        let tab_id = Uuid::new_v4().to_string();
        let new_tab = OpenTab {
            id: tab_id.clone(),
            session_id,
            workspace_id,
            last_accessed_at: (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32),
            order: state.open_tabs.len() as i32,
            // UI state fields initialized to None
            input: None,
            attached_file_paths: None,
            auto_mode: None,
            auto_chat_iteration: None,
            is_dirty: None,
        };

        state.open_tabs.push(new_tab);
        state.active_tab_id = Some(tab_id.clone());
        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await?;
        Ok(tab_id)
    }

    async fn close_tab(&self, tab_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();

        // Find tab index
        let tab_index = state
            .open_tabs
            .iter()
            .position(|tab| tab.id == tab_id)
            .ok_or_else(|| OrcsError::not_found("Tab", &tab_id))?;

        // Remove tab
        state.open_tabs.remove(tab_index);

        // If active tab was closed, set next tab as active
        if state.active_tab_id.as_ref() == Some(&tab_id) {
            state.active_tab_id = if !state.open_tabs.is_empty() {
                let next_index = tab_index.min(state.open_tabs.len() - 1);
                Some(state.open_tabs[next_index].id.clone())
            } else {
                None
            };
        }

        // Reorder remaining tabs
        for (i, tab) in state.open_tabs.iter_mut().enumerate() {
            tab.order = i as i32;
        }

        let cloned_state = state.clone();
        drop(state);
        self.save_state(cloned_state).await
    }

    async fn set_active_tab(&self, tab_id: String) -> Result<()> {
        let mut state = self.state.lock().await.clone();

        // Verify tab exists
        if !state.open_tabs.iter().any(|tab| tab.id == tab_id) {
            return Err(OrcsError::not_found("Tab", &tab_id));
        }

        // Update last_accessed_at
        state.open_tabs.iter_mut().for_each(|tab| {
            if tab.id == tab_id {
                tab.last_accessed_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i32;
            }
        });

        state.active_tab_id = Some(tab_id);

        // Memory-only update (no disk write)
        // Will be saved on app shutdown or when important operation occurs
        self.update_state_in_memory(state).await;
        Ok(())
    }

    async fn reorder_tabs(&self, from_index: usize, to_index: usize) -> Result<()> {
        let mut state = self.state.lock().await.clone();

        if from_index >= state.open_tabs.len() || to_index >= state.open_tabs.len() {
            return Err(OrcsError::internal(format!(
                "Invalid tab indices: from={}, to={}, len={}",
                from_index,
                to_index,
                state.open_tabs.len()
            )));
        }

        let tab = state.open_tabs.remove(from_index);
        state.open_tabs.insert(to_index, tab);

        // Update order field
        for (i, tab) in state.open_tabs.iter_mut().enumerate() {
            tab.order = i as i32;
        }

        // Memory-only update (no disk write)
        // Will be saved on app shutdown or when important operation occurs
        self.update_state_in_memory(state).await;
        Ok(())
    }

    async fn update_state_in_memory(&self, state: AppState) {
        let mut state_lock = self.state.lock().await;
        *state_lock = state;
    }

    async fn update_tab_ui_state(
        &self,
        tab_id: String,
        input: Option<String>,
        attached_file_paths: Option<Vec<String>>,
        auto_mode: Option<bool>,
        auto_chat_iteration: Option<i32>,
        is_dirty: Option<bool>,
    ) -> Result<()> {
        let mut state = self.state.lock().await.clone();

        // Find the tab and update its UI state
        let tab = state
            .open_tabs
            .iter_mut()
            .find(|t| t.id == tab_id)
            .ok_or_else(|| OrcsError::not_found("Tab", &tab_id))?;

        // Update only the provided fields (partial update)
        if let Some(v) = input {
            tab.input = Some(v);
        }
        if let Some(v) = attached_file_paths {
            tab.attached_file_paths = Some(v);
        }
        if let Some(v) = auto_mode {
            tab.auto_mode = Some(v);
        }
        if let Some(v) = auto_chat_iteration {
            tab.auto_chat_iteration = Some(v);
        }
        if let Some(v) = is_dirty {
            tab.is_dirty = Some(v);
        }

        // Memory-only update (no disk write)
        // Will be saved on app shutdown or when important operation occurs
        self.update_state_in_memory(state).await;
        Ok(())
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
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let service = AppStateService::with_base_dir(Some(temp_file.path()))
            .await
            .unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        // Default should be None
        assert!(workspace_id.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_last_selected_workspace() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let service = AppStateService::with_base_dir(Some(temp_file.path()))
            .await
            .unwrap();
        service
            .set_last_selected_workspace("ws-123".to_string())
            .await
            .unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        assert_eq!(workspace_id, Some("ws-123".to_string()));
    }

    #[tokio::test]
    async fn test_clear_last_selected_workspace() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let service = AppStateService::with_base_dir(Some(temp_file.path()))
            .await
            .unwrap();
        service
            .set_last_selected_workspace("ws-456".to_string())
            .await
            .unwrap();
        service.clear_last_selected_workspace().await.unwrap();
        let workspace_id = service.get_last_selected_workspace().await;
        assert!(workspace_id.is_none());
    }
}
