//! AppState DTOs and migrations
//!
//! This module defines versioned DTOs for application state that persists
//! across sessions, such as the last selected workspace ID.

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, Versioned};

use orcs_core::state::model::{AppState, PLACEHOLDER_DEFAULT_WORKSPACE_ID};

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
/// Made default_workspace_id field required.
/// The default workspace (~/orcs) is now mandatory and always initialized.
#[derive(Debug, Clone, Serialize, Deserialize, Versioned)]
#[versioned(version = "1.3.0")]
pub struct AppStateV1_3 {
    /// ID of the last selected workspace.
    /// This is used to restore the workspace on application startup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_selected_workspace_id: Option<String>,

    /// ID of the default system workspace (~/orcs).
    /// This is a fallback workspace that is always available.
    /// Must be initialized during bootstrap.
    pub default_workspace_id: String,

    /// ID of the currently active session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_session_id: Option<String>,
}

/// Type alias for the latest AppState version.
pub type AppStateDTO = AppStateV1_3;

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
            default_workspace_id: PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string(),
            active_session_id: None,
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
/// Makes default_workspace_id required by setting placeholder if None.
impl version_migrate::MigratesTo<AppStateV1_3> for AppStateV1_2 {
    fn migrate(self) -> AppStateV1_3 {
        AppStateV1_3 {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self
                .default_workspace_id
                .unwrap_or_else(|| PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string()),
            active_session_id: self.active_session_id,
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
            default_workspace_id: self
                .default_workspace_id
                .unwrap_or_else(|| PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string()),
            active_session_id: None,
        }
    }
}

/// Convert domain model to AppStateV1_1 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_1 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_1 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: Some(state.default_workspace_id),
        }
    }
}

/// Convert AppStateV1_2 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_2 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
            default_workspace_id: self
                .default_workspace_id
                .unwrap_or_else(|| PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string()),
            active_session_id: self.active_session_id,
        }
    }
}

/// Convert domain model to AppStateV1_2 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_2 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_2 {
            last_selected_workspace_id: state.last_selected_workspace_id,
            default_workspace_id: Some(state.default_workspace_id),
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
/// - V1.2 → V1.3: Makes `default_workspace_id` required (sets placeholder if None)
/// - V1.3 → AppState: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_app_state_migrator();
/// let state: AppState = migrator.load_flat_from("app_state", toml_value)?;
/// ```
pub fn create_app_state_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0 -> V1.1 -> V1.2 -> V1.3 -> AppState
    let app_state_path = version_migrate::Migrator::define("app_state")
        .from::<AppStateV1_0>()
        .step::<AppStateV1_1>()
        .step::<AppStateV1_2>()
        .step::<AppStateV1_3>()
        .into_with_save::<AppState>();

    migrator
        .register(app_state_path)
        .expect("Failed to register app_state migration path");

    migrator
}
