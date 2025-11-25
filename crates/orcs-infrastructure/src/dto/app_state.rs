//! AppState DTOs and migrations
//!
//! This module defines versioned DTOs for application state that persists
//! across sessions, such as the last selected workspace ID.

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, Versioned};

use orcs_core::state::model::{AppState, OpenTab};

/// Application state configuration V1.0.0 (initial version).
///
/// Contains application-level state that persists across restarts,
/// such as the last selected workspace ID.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.0.0")]
pub struct AppStateV1_0 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,
}

/// Application state configuration V1.1.0.
///
/// Added default_workspace_id field for system workspace (ConfigDir).
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.1.0")]
pub struct AppStateV1_1 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (ConfigDir as workspace).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,
}

/// Application state configuration V1.2.0.
///
/// Added active_session_id field to track the currently active session.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.2.0")]
pub struct AppStateV1_2 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (ConfigDir as workspace).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,
}

/// Application state configuration V1.3.0.
///
/// Reverted default_workspace_id back to optional.
/// Using placeholder pattern was a bad practice - if a value can be invalid, it should be Option.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct AppStateV1_3 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,
}

/// Open tab DTO for V1.4.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenTabDTO {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub last_accessed_at: i32,
    pub order: i32,
}

/// Application state configuration V1.4.0.
///
/// Added open_tabs and active_tab_id for tab management.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.4.0")]
pub struct AppStateV1_4 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,

    /// List of currently open tabs.
    #[serde(default)]
    pub open_tabs: Vec<OpenTabDTO>,

    /// ID of the currently active tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_id: Option<String>,
}

/// Application state configuration V1.5.0.
///
/// Migrated to camelCase for JSON serialization to match TypeScript conventions.
/// This ensures consistency between Rust persistence and TypeScript API.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.5.0")]
#[serde(rename_all = "camelCase")]
pub struct AppStateV1_5 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,

    /// List of currently open tabs.
    #[serde(default)]
    pub open_tabs: Vec<OpenTabDTO>,

    /// ID of the currently active tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_id: Option<String>,
}

/// Open tab DTO for V1.6 with UI state support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenTabDTOV1_6 {
    pub id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub last_accessed_at: i32,
    pub order: i32,

    // UI State fields (added in V1.6)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_file_paths: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_mode: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_chat_iteration: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_dirty: Option<bool>,
}

/// Application state configuration V1.6.0.
///
/// Added UI state fields to OpenTab for tab state persistence across app restarts.
/// This allows restoring input text, attached files, AutoChat state, etc.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.6.0")]
#[serde(rename_all = "camelCase")]
pub struct AppStateV1_6 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// None if not yet initialized.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_workspace_id: Option<String>,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,

    /// List of currently open tabs (with UI state support).
    #[serde(default)]
    pub open_tabs: Vec<OpenTabDTOV1_6>,

    /// ID of the currently active tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tab_id: Option<String>,
}

/// Type alias for the latest AppState version.
pub type AppStateDTO = AppStateV1_6;

impl Default for AppStateV1_0 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
        }
    }
}

impl Default for AppStateV1_1 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
        }
    }
}

impl Default for AppStateV1_2 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
        }
    }
}

impl Default for AppStateV1_3 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
        }
    }
}

impl Default for AppStateV1_4 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

impl Default for AppStateV1_5 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

impl Default for AppStateV1_6 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
            default_workspace_id: None,
            active_session_id: None,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

// ============================================================================
// Migration implementations
// ============================================================================

/// Migration from AppStateV1_0 to AppStateV1_1.
/// Adds default_workspace_id field with default value (None).
impl version_migrate::MigratesTo<AppStateV1_1> for AppStateV1_0 {
    fn migrate(self) -> AppStateV1_1 {
        AppStateV1_1 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: None, // Default: no default workspace set
        }
    }
}

/// Migration from AppStateV1_1 to AppStateV1_2.
/// Adds active_session_id field with default value (None).
impl version_migrate::MigratesTo<AppStateV1_2> for AppStateV1_1 {
    fn migrate(self) -> AppStateV1_2 {
        AppStateV1_2 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: None, // Default: no active session
        }
    }
}

/// Migration from AppStateV1_2 to AppStateV1_3.
/// Simple copy - both versions now have the same structure.
impl version_migrate::MigratesTo<AppStateV1_3> for AppStateV1_2 {
    fn migrate(self) -> AppStateV1_3 {
        AppStateV1_3 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
        }
    }
}

/// Migration from AppStateV1_3 to AppStateV1_4.
/// Adds open_tabs and active_tab_id fields with default values.
impl version_migrate::MigratesTo<AppStateV1_4> for AppStateV1_3 {
    fn migrate(self) -> AppStateV1_4 {
        AppStateV1_4 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: Vec::new(), // Default: no tabs open
            active_tab_id: None,   // Default: no active tab
        }
    }
}

/// Migration from AppStateV1_4 to AppStateV1_5.
/// Simple copy - migrating to camelCase serialization format.
impl version_migrate::MigratesTo<AppStateV1_5> for AppStateV1_4 {
    fn migrate(self) -> AppStateV1_5 {
        AppStateV1_5 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: self.open_tabs,
            active_tab_id: self.active_tab_id,
        }
    }
}

/// Migration from AppStateV1_5 to AppStateV1_6.
/// Migrates OpenTabDTO to OpenTabDTOV1_6 by adding UI state fields with default values (None).
impl version_migrate::MigratesTo<AppStateV1_6> for AppStateV1_5 {
    fn migrate(self) -> AppStateV1_6 {
        AppStateV1_6 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: self
                .open_tabs
                .into_iter()
                .map(|tab| OpenTabDTOV1_6 {
                    id: tab.id,
                    session_id: tab.session_id,
                    workspace_id: tab.workspace_id,
                    last_accessed_at: tab.last_accessed_at,
                    order: tab.order,
                    // Default UI state values
                    input: None,
                    attached_file_paths: None,
                    auto_mode: None,
                    auto_chat_iteration: None,
                    is_dirty: None,
                })
                .collect(),
            active_tab_id: self.active_tab_id,
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert AppStateV1_1 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_1 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: None,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

/// Convert domain model to AppStateV1_1 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_1 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_1 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
        }
    }
}

/// Convert AppStateV1_2 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_2 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

/// Convert domain model to AppStateV1_2 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_2 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_2 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
            active_session_id: state.active_session_id,
        }
    }
}

/// Convert AppStateV1_3 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_3 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: Vec::new(),
            active_tab_id: None,
        }
    }
}

/// Convert domain model to AppStateV1_3 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_3 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_3 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
            active_session_id: state.active_session_id,
        }
    }
}

/// Convert OpenTabDTO to OpenTab domain model.
/// OpenTabDTO is the old format (V1.4, V1.5) without UI state fields.
/// UI state fields are initialized to None.
impl From<OpenTabDTO> for OpenTab {
    fn from(dto: OpenTabDTO) -> Self {
        OpenTab {
            id: dto.id,
            session_id: dto.session_id,
            workspace_id: dto.workspace_id,
            last_accessed_at: dto.last_accessed_at,
            order: dto.order,
            // UI state fields default to None (backward compatibility)
            input: None,
            attached_file_paths: None,
            auto_mode: None,
            auto_chat_iteration: None,
            is_dirty: None,
        }
    }
}

/// Convert OpenTab domain model to OpenTabDTO.
impl From<OpenTab> for OpenTabDTO {
    fn from(tab: OpenTab) -> Self {
        OpenTabDTO {
            id: tab.id,
            session_id: tab.session_id,
            workspace_id: tab.workspace_id,
            last_accessed_at: tab.last_accessed_at,
            order: tab.order,
        }
    }
}

/// Convert AppStateV1_4 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_4 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: self.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: self.active_tab_id,
        }
    }
}

/// Convert domain model to AppStateV1_4 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_4 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_4 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
            active_session_id: state.active_session_id,
            open_tabs: state.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: state.active_tab_id,
        }
    }
}

/// Convert AppStateV1_5 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_5 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: self.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: self.active_tab_id,
        }
    }
}

/// Convert domain model to AppStateV1_5 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_5 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_5 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
            active_session_id: state.active_session_id,
            open_tabs: state.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: state.active_tab_id,
        }
    }
}

/// Convert OpenTabDTOV1_6 to OpenTab domain model.
impl From<OpenTabDTOV1_6> for OpenTab {
    fn from(dto: OpenTabDTOV1_6) -> Self {
        OpenTab {
            id: dto.id,
            session_id: dto.session_id,
            workspace_id: dto.workspace_id,
            last_accessed_at: dto.last_accessed_at,
            order: dto.order,
            input: dto.input,
            attached_file_paths: dto.attached_file_paths,
            auto_mode: dto.auto_mode,
            auto_chat_iteration: dto.auto_chat_iteration,
            is_dirty: dto.is_dirty,
        }
    }
}

/// Convert OpenTab domain model to OpenTabDTOV1_6.
impl From<OpenTab> for OpenTabDTOV1_6 {
    fn from(tab: OpenTab) -> Self {
        OpenTabDTOV1_6 {
            id: tab.id,
            session_id: tab.session_id,
            workspace_id: tab.workspace_id,
            last_accessed_at: tab.last_accessed_at,
            order: tab.order,
            input: tab.input,
            attached_file_paths: tab.attached_file_paths,
            auto_mode: tab.auto_mode,
            auto_chat_iteration: tab.auto_chat_iteration,
            is_dirty: tab.is_dirty,
        }
    }
}

/// Convert AppStateV1_6 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_6 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self.default_workspace_id,
            active_session_id: self.active_session_id,
            open_tabs: self.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: self.active_tab_id,
        }
    }
}

/// Convert domain model to AppStateV1_6 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_6 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_6 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: state.default_workspace_id,
            active_session_id: state.active_session_id,
            open_tabs: state.open_tabs.into_iter().map(Into::into).collect(),
            active_tab_id: state.active_tab_id,
        }
    }
}

// ============================================================================
// Migrator factory
// ============================================================================

/// Creates and configures a Migrator instance for AppState entities.
///
/// The migrator handles automatic schema migration and conversion to the domain model.
///
/// # Migration Path
///
/// - V1.0 → V1.1: Adds `default_workspace_id` field with default value (None)
/// - V1.1 → V1.2: Adds `active_session_id` field with default value (None)
/// - V1.2 → V1.3: Reverts default_workspace_id back to optional
/// - V1.3 → V1.4: Adds `open_tabs` and `active_tab_id` for tab management
/// - V1.4 → V1.5: Migrates to camelCase serialization format
/// - V1.5 → V1.6: Adds UI state fields to OpenTab (input, attached_file_paths, auto_mode, auto_chat_iteration, is_dirty)
/// - V1.6 → AppState: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_app_state_migrator();
/// let state: AppState = migrator.load_flat_from("app_state", toml_value)?;
/// ```
pub fn create_app_state_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0 -> V1.1 -> V1.2 -> V1.3 -> V1.4 -> V1.5 -> V1.6 -> AppState
    let app_state_path = version_migrate::Migrator::define("app_state")
        .from::<AppStateV1_0>()
        .step::<AppStateV1_1>()
        .step::<AppStateV1_2>()
        .step::<AppStateV1_3>()
        .step::<AppStateV1_4>()
        .step::<AppStateV1_5>()
        .step::<AppStateV1_6>()
        .into_with_save::<AppState>();

    migrator
        .register(app_state_path)
        .expect("Failed to register app_state migration path");

    migrator
}
