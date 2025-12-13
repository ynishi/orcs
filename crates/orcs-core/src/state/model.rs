//! Application state domain models.
//!
//! Contains domain models for application-level state that persists across sessions.

use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Represents an open tab in the application.
///
/// Tabs are views of sessions. This struct tracks which sessions are currently
/// open as tabs, their display order, when they were last accessed, and their UI state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SchemaBridge)]
#[serde(rename_all = "camelCase")]
pub struct OpenTab {
    /// Unique tab identifier (UUID format)
    pub id: String,
    /// Associated session ID
    pub session_id: String,
    /// Workspace ID this tab belongs to
    pub workspace_id: String,
    /// Last access timestamp (Unix timestamp in milliseconds as i32 for TypeScript compatibility)
    /// Note: Using milliseconds/1000 to fit in i32 range
    pub last_accessed_at: i32,
    /// Display order (lower numbers appear first)
    pub order: i32,

    // ============================================================================
    // UI State (persisted to restore tab state across app restarts)
    // ============================================================================
    /// Input text being typed (for message input field)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,

    /// Attached file paths (File objects cannot be serialized, so we store paths)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_file_paths: Option<Vec<String>>,

    /// AutoChat mode enabled flag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_mode: Option<bool>,

    /// Current AutoChat iteration number (null = not running)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_iteration: Option<i32>,

    /// Dirty flag (has unsaved changes)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_dirty: Option<bool>,
}

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
/// * `active_session_id` - The ID of the currently active session.
/// * `open_tabs` - List of currently open tabs.
/// * `active_tab_id` - The ID of the currently active tab.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, SchemaBridge, Default)]
#[queryable(entity = "app_state")]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    pub active_session_id: Option<String>,

    /// List of currently open tabs.
    #[serde(default)]
    pub open_tabs: Vec<OpenTab>,

    /// ID of the currently active tab.
    pub active_tab_id: Option<String>,
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
        assert!(state.open_tabs.is_empty());
        assert!(state.active_tab_id.is_none());
    }

    #[test]
    fn test_default() {
        let state = AppState::default();
        assert!(state.last_selected_workspace_id.is_none());
        assert!(state.default_workspace_id.is_none());
        assert!(state.active_session_id.is_none());
        assert!(state.open_tabs.is_empty());
        assert!(state.active_tab_id.is_none());
    }
}
