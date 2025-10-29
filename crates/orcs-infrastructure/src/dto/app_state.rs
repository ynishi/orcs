//! AppState DTOs and migrations
//!
//! This module defines versioned DTOs for application state that persists
//! across sessions, such as the last selected workspace ID.

use serde::{Deserialize, Serialize};
use version_migrate::{IntoDomain, Versioned};

use orcs_core::app_state::AppState;

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

/// Type alias for the latest AppState version.
pub type AppStateDTO = AppStateV1_0;

impl Default for AppStateV1_0 {
    fn default() -> Self {
        Self {
            last_selected_workspace_id: None,
        }
    }
}

// ============================================================================
// Domain model conversions
// ============================================================================

/// Convert AppStateV1_0 DTO to domain model.
impl IntoDomain<AppState> for AppStateV1_0 {
    fn into_domain(self) -> AppState {
        AppState {
            last_selected_workspace_id: self.last_selected_workspace_id,
        }
    }
}

/// Convert domain model to AppStateV1_0 DTO for persistence.
impl version_migrate::FromDomain<AppState> for AppStateV1_0 {
    fn from_domain(state: AppState) -> Self {
        AppStateV1_0 {
            last_selected_workspace_id: state.last_selected_workspace_id,
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
/// - V1.0 → AppState: Converts DTO to domain model
///
/// # Example
///
/// ```ignore
/// let migrator = create_app_state_migrator();
/// let state: AppState = migrator.load_flat_from("app_state", toml_value)?;
/// ```
pub fn create_app_state_migrator() -> version_migrate::Migrator {
    let mut migrator = version_migrate::Migrator::builder().build();

    // Register migration path: V1.0 -> AppState
    let app_state_path = version_migrate::Migrator::define("app_state")
        .from::<AppStateV1_0>()
        .into_with_save::<AppState>();

    migrator
        .register(app_state_path)
        .expect("Failed to register app_state migration path");

    migrator
}
