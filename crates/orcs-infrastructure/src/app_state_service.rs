//! Application state service implementation.
//!
//! This module provides a service for managing application-level state that persists
//! across sessions, such as the last selected workspace ID.

use crate::dto::create_app_state_migrator;
use crate::paths::{OrcsPaths, ServiceType};
use orcs_core::app_state::AppState;
use std::sync::{Arc, Mutex, RwLock};
use version_migrate::{FileStorage, FileStorageStrategy, FormatStrategy, LoadBehavior};

/// Service for managing application state.
///
/// This implementation reads and writes application state using FileStorage
/// and caches it to avoid repeated file I/O operations.
///
/// # Example
///
/// ```ignore
/// use orcs_infrastructure::app_state_service::AppStateService;
///
/// let service = AppStateService::new()?;
/// service.set_last_selected_workspace("ws-123".to_string())?;
/// let workspace_id = service.get_last_selected_workspace();
/// ```
#[derive(Clone)]
pub struct AppStateService {
    /// Cached app state loaded from storage.
    /// Uses RwLock for thread-safe lazy loading.
    state: Arc<RwLock<Option<AppState>>>,
    /// FileStorage instance for persistence.
    /// Wrapped in Mutex for interior mutability.
    storage: Arc<Mutex<FileStorage>>,
}

impl AppStateService {
    /// Creates a new AppStateService.
    ///
    /// This method ensures that the app_state file exists. If it doesn't exist,
    /// it creates the file with default values via LoadBehavior::CreateIfMissing.
    ///
    /// Uses the centralized path management via `ServiceType::AppState`.
    pub fn new() -> Result<Self, String> {
        // Get file path for AppState via centralized path management
        let path_type = OrcsPaths::get_path(ServiceType::AppState).map_err(|e| e.to_string())?;
        let file_path = path_type.into_path_buf(); // app_state.toml

        // Setup migrator
        let migrator = create_app_state_migrator();

        // Setup storage strategy: TOML format, CreateIfMissing
        let strategy = FileStorageStrategy::new()
            .with_format(FormatStrategy::Toml)
            .with_load_behavior(LoadBehavior::CreateIfMissing);

        // Create FileStorage (automatically loads or creates empty config)
        let storage = FileStorage::new(file_path, migrator, strategy)
            .map_err(|e| format!("Failed to create FileStorage: {}", e))?;

        Ok(Self {
            state: Arc::new(RwLock::new(None)),
            storage: Arc::new(Mutex::new(storage)),
        })
    }

    /// Loads the app state from storage if not already cached.
    fn load_state(&self) -> AppState {
        // Check if already cached
        {
            let read_lock = self.state.read().unwrap();
            if let Some(ref cached) = *read_lock {
                return cached.clone();
            }
        }

        // Load from FileStorage
        let loaded = Self::load_from_storage(&self.storage).unwrap_or_else(|_| AppState::default());

        // Cache it
        {
            let mut write_lock = self.state.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        loaded
    }

    /// Saves the app state to storage.
    fn save_state(&self, state: &AppState) -> Result<(), String> {
        let mut storage = self.storage.lock().unwrap();

        // Update and save atomically
        storage
            .update_and_save("app_state", vec![state.clone()])
            .map_err(|e| format!("Failed to save app_state: {}", e))?;

        // Update cache
        {
            let mut write_lock = self.state.write().unwrap();
            *write_lock = Some(state.clone());
        }

        Ok(())
    }

    /// Loads AppState from FileStorage.
    fn load_from_storage(storage: &Mutex<FileStorage>) -> Result<AppState, String> {
        let storage = storage.lock().unwrap();

        // Query from storage
        let states: Vec<AppState> = storage
            .query("app_state")
            .map_err(|e| format!("Failed to query app_state: {}", e))?;

        // app_state is a single object, take first or return default
        Ok(states.into_iter().next().unwrap_or_default())
    }

    /// Gets the last selected workspace ID.
    pub fn get_last_selected_workspace(&self) -> Option<String> {
        let state = self.load_state();
        state.last_selected_workspace_id
    }

    /// Sets the last selected workspace ID.
    pub fn set_last_selected_workspace(&self, workspace_id: String) -> Result<(), String> {
        let mut state = self.load_state();
        state.set_last_selected_workspace(workspace_id);
        self.save_state(&state)
    }

    /// Clears the last selected workspace ID.
    pub fn clear_last_selected_workspace(&self) -> Result<(), String> {
        let mut state = self.load_state();
        state.clear_last_selected_workspace();
        self.save_state(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_service_creation() {
        let service = AppStateService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_get_last_selected_workspace_default() {
        let service = AppStateService::new().unwrap();
        let workspace_id = service.get_last_selected_workspace();
        // Default should be None
        assert!(workspace_id.is_none());
    }
}
