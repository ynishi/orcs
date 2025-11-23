//! Application state domain models.
//!
//! Contains domain models for application-level state that persists across sessions.

use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Application state that persists across restarts.
///
/// This struct contains application-level state information that should be
/// preserved across application restarts.
///
/// # File Location
///
/// - macOS: `~/Library/Preferences/com.orcs-app/state.toml`
/// - Linux: `~/.config/com.orcs-app/state.toml`
/// - Windows: `%APPDATA%\com.orcs-app\state.toml`
///
/// # Fields
///
/// * `last_selected_workspace_id` - The ID of the last workspace the user selected.
///   This is used to restore the workspace on application startup.
/// * `default_workspace_id` - The ID of the system's default workspace (~/orcs).
///   None if not yet initialized.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, SchemaBridge)]
#[queryable(entity = "app_state")]
pub struct AppState {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    pub default_workspace_id: Option<String>,

    pub active_session_id: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
        }
    }
}

impl AppState {
    /// Creates a new AppState with default values.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let state = AppState::new();
        assert!(state.last_selected_workspace_id.is_none());
        assert!(state.default_workspace_id.is_none());
        assert!(state.active_session_id.is_none());
    }

    #[test]
    fn test_default() {
        let state = AppState::default();
        assert!(state.last_selected_workspace_id.is_none());
        assert!(state.default_workspace_id.is_none());
        assert!(state.active_session_id.is_none());
    }
}
