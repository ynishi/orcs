//! State repository trait.

use async_trait::async_trait;

use crate::error::Result;
use crate::state::model::AppState;

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
}
