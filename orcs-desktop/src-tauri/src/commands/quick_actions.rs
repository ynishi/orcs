//! Quick Action Dock Tauri commands.

use orcs_core::quick_action::QuickActionConfig;
use tauri::State;

use crate::app::AppState;

/// Gets the quick action configuration for a workspace.
#[tauri::command]
pub async fn get_quick_actions(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<QuickActionConfig, String> {
    state
        .quick_action_repository
        .load(&workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Saves the quick action configuration for a workspace.
#[tauri::command]
pub async fn save_quick_actions(
    workspace_id: String,
    config: QuickActionConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .quick_action_repository
        .save(&workspace_id, &config)
        .await
        .map_err(|e| e.to_string())
}

/// Updates a single slot in the quick action configuration.
#[tauri::command]
pub async fn update_quick_action_slot(
    workspace_id: String,
    slot_id: String,
    command_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<QuickActionConfig, String> {
    let mut config = state
        .quick_action_repository
        .load(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    config.set_slot_command(&slot_id, command_name);

    state
        .quick_action_repository
        .save(&workspace_id, &config)
        .await
        .map_err(|e| e.to_string())?;

    Ok(config)
}
