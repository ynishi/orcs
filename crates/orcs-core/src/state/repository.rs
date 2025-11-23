//! State repository trait.

use async_trait::async_trait;

use crate::error::Result;
use crate::state::model::{AppState, OpenTab};

/// Repository for managing application state.
#[async_trait]
pub trait StateRepository: Send + Sync {
    /// Saves the app state to storage.
    async fn save_state(&self, state: AppState) -> Result<()>;

    async fn get_state(&self) -> Result<AppState>;

    async fn get_last_selected_workspace(&self) -> Option<String>;

    async fn set_last_selected_workspace(&self, workspace_id: String) -> Result<()>;

    async fn clear_last_selected_workspace(&self) -> Result<()>;

    async fn get_default_workspace(&self) -> Option<String>;

    async fn set_default_workspace(&self, workspace_id: String) -> Result<()>;

    async fn get_active_session(&self) -> Option<String>;

    async fn set_active_session(&self, session_id: String) -> Result<()>;

    async fn clear_active_session(&self) -> Result<()>;

    // Tab management methods
    async fn get_open_tabs(&self) -> Vec<OpenTab>;

    async fn get_active_tab_id(&self) -> Option<String>;

    async fn open_tab(&self, session_id: String, workspace_id: String) -> Result<String>;

    async fn close_tab(&self, tab_id: String) -> Result<()>;

    async fn set_active_tab(&self, tab_id: String) -> Result<()>;

    async fn reorder_tabs(&self, from_index: usize, to_index: usize) -> Result<()>;

    /// Updates state in memory without saving to disk.
    /// Used for frequent operations like tab switching.
    async fn update_state_in_memory(&self, state: AppState);
}
