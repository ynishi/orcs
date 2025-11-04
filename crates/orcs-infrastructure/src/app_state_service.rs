//! Application state service implementation.
//!
//! This module provides a service for managing application-level state that persists
//! across sessions, such as the last selected workspace ID.

use crate::dto::create_app_state_migrator;
use crate::paths::OrcsPaths;
use orcs_core::app_state::AppState;
use std::sync::{Arc, RwLock};
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// Service for managing application state.
///
/// This implementation reads and writes application state using AsyncDirStorage
/// and caches it to avoid repeated file I/O operations.
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
pub struct AppStateService {
    /// Cached app state loaded from storage.
    /// Uses RwLock for thread-safe lazy loading.
    state: Arc<RwLock<Option<AppState>>>,
    /// AsyncDirStorage instance for persistence.
    storage: Arc<AsyncDirStorage>,
}

impl AppStateService {
    /// Creates a new AppStateService.
    ///
    /// The state is loaded lazily on first access to avoid blocking
    /// during initialization.
    pub async fn new() -> Result<Self, String> {
        let config_dir = OrcsPaths::config_dir().map_err(|e| e.to_string())?;

        // Setup AppPaths with CustomBase strategy
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(config_dir));

        // Setup migrator
        let migrator = create_app_state_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage for app_state (single file: app_state.toml)
        let storage = AsyncDirStorage::new(paths, ".", migrator, strategy)
            .await
            .map_err(|e| format!("Failed to create AsyncDirStorage: {}", e))?;

        Ok(Self {
            state: Arc::new(RwLock::new(None)),
            storage: Arc::new(storage),
        })
    }

    /// Loads the app state from storage if not already cached.
    async fn load_state(&self) -> AppState {
        // Check if already cached
        {
            let read_lock = self.state.read().unwrap();
            if let Some(ref cached) = *read_lock {
                return cached.clone();
            }
        }

        // Load from AsyncDirStorage
        let loaded = Self::load_from_storage(&self.storage)
            .await
            .unwrap_or_else(|_| AppState::default());

        // Cache it
        {
            let mut write_lock = self.state.write().unwrap();
            *write_lock = Some(loaded.clone());
        }

        loaded
    }

    /// Saves the app state to storage.
    async fn save_state(&self, state: &AppState) -> Result<(), String> {
        // Save via AsyncDirStorage using fixed entity_name "app_state" and id "state"
        self.storage
            .save("app_state", "state", state.clone())
            .await
            .map_err(|e| format!("Failed to save app_state: {}", e))?;

        // Update cache
        {
            let mut write_lock = self.state.write().unwrap();
            *write_lock = Some(state.clone());
        }

        Ok(())
    }

    /// Loads AppState from AsyncDirStorage.
    async fn load_from_storage(storage: &AsyncDirStorage) -> Result<AppState, String> {
        // Load from storage using fixed entity_name "app_state" and id "state"
        match storage.load::<AppState>("app_state", "state").await {
            Ok(state) => Ok(state),
            Err(_) => {
                // If not found, return default
                Ok(AppState::default())
            }
        }
    }

    /// Gets the last selected workspace ID.
    pub async fn get_last_selected_workspace(&self) -> Option<String> {
        let state = self.load_state().await;
        state.last_selected_workspace_id
    }

    /// Sets the last selected workspace ID.
    pub async fn set_last_selected_workspace(&self, workspace_id: String) -> Result<(), String> {
        let mut state = self.load_state().await;
        state.set_last_selected_workspace(workspace_id);
        self.save_state(&state).await
    }

    /// Clears the last selected workspace ID.
    pub async fn clear_last_selected_workspace(&self) -> Result<(), String> {
        let mut state = self.load_state().await;
        state.clear_last_selected_workspace();
        self.save_state(&state).await
    }

    /// Synchronous wrapper for get_last_selected_workspace (for non-async contexts)
    pub fn get_last_selected_workspace_sync(&self) -> Option<String> {
        tokio::runtime::Handle::current().block_on(self.get_last_selected_workspace())
    }

    /// Synchronous wrapper for set_last_selected_workspace (for non-async contexts)
    pub fn set_last_selected_workspace_sync(&self, workspace_id: String) -> Result<(), String> {
        tokio::runtime::Handle::current().block_on(self.set_last_selected_workspace(workspace_id))
    }
}

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
}
