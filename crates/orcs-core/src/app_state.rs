//! Application state domain models.
//!
//! Contains domain models for application-level state that persists across sessions.

use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Application state that persists across restarts.
///
/// This struct contains application-level state information that should be
/// preserved across application restarts, such as the last selected workspace.
///
/// # Fields
///
/// * `last_selected_workspace_id` - The ID of the last workspace the user selected.
///   This is used to restore the workspace on application startup.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Queryable)]
#[queryable(entity = "app_state")]
pub struct AppState {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    pub last_selected_workspace_id: Option<String>,
}

impl AppState {
    /// Creates a new AppState with no last selected workspace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new AppState with a specific last selected workspace ID.
    pub fn with_last_selected_workspace(workspace_id: String) -> Self {
        Self {
            last_selected_workspace_id: Some(workspace_id),
        }
    }

    /// Sets the last selected workspace ID.
    pub fn set_last_selected_workspace(&mut self, workspace_id: String) {
        self.last_selected_workspace_id = Some(workspace_id);
    }

    /// Clears the last selected workspace ID.
    pub fn clear_last_selected_workspace(&mut self) {
        self.last_selected_workspace_id = None;
    }

    /// Returns the last selected workspace ID, if any.
    pub fn last_selected_workspace_id(&self) -> Option<&str> {
        self.last_selected_workspace_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = AppState::new();
        assert!(state.last_selected_workspace_id.is_none());
    }

    #[test]
    fn test_with_last_selected_workspace() {
        let state = AppState::with_last_selected_workspace("ws-123".to_string());
        assert_eq!(state.last_selected_workspace_id(), Some("ws-123"));
    }

    #[test]
    fn test_set_last_selected_workspace() {
        let mut state = AppState::new();
        state.set_last_selected_workspace("ws-456".to_string());
        assert_eq!(state.last_selected_workspace_id(), Some("ws-456"));
    }

    #[test]
    fn test_clear_last_selected_workspace() {
        let mut state = AppState::with_last_selected_workspace("ws-789".to_string());
        state.clear_last_selected_workspace();
        assert!(state.last_selected_workspace_id.is_none());
    }
}
