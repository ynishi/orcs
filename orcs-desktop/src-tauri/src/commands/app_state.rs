use orcs_core::state::{model::AppState as CoreAppState, repository::StateRepository};
use tauri::{AppHandle, Emitter, State};

use crate::app::AppState;

/// Gets the current AppState snapshot (for initial load).
///
/// This command is called once when the frontend initializes to get
/// the current state. After that, the frontend listens to `app-state:update`
/// events for any changes.
#[tauri::command]
pub async fn get_app_state_snapshot(
    state: State<'_, AppState>,
) -> Result<CoreAppState, String> {
    state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())
}

/// Sets the last selected workspace ID and emits an update event.
#[tauri::command]
pub async fn set_last_selected_workspace(
    workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Update in repository
    state
        .app_state_service
        .set_last_selected_workspace(workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get the updated state
    let updated_state = state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())?;

    // Emit event to frontend
    app.emit("app-state:update", &updated_state)
        .map_err(|e| format!("Failed to emit app-state:update: {}", e))?;

    Ok(())
}

/// Clears the last selected workspace ID and emits an update event.
#[tauri::command]
pub async fn clear_last_selected_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .clear_last_selected_workspace()
        .await
        .map_err(|e| e.to_string())?;

    let updated_state = state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())?;

    app.emit("app-state:update", &updated_state)
        .map_err(|e| format!("Failed to emit app-state:update: {}", e))?;

    Ok(())
}

/// Sets the active session ID and emits an update event.
#[tauri::command]
pub async fn set_active_session_in_app_state(
    session_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .set_active_session(session_id)
        .await
        .map_err(|e| e.to_string())?;

    let updated_state = state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())?;

    app.emit("app-state:update", &updated_state)
        .map_err(|e| format!("Failed to emit app-state:update: {}", e))?;

    Ok(())
}

/// Clears the active session ID and emits an update event.
#[tauri::command]
pub async fn clear_active_session_in_app_state(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .clear_active_session()
        .await
        .map_err(|e| e.to_string())?;

    let updated_state = state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())?;

    app.emit("app-state:update", &updated_state)
        .map_err(|e| format!("Failed to emit app-state:update: {}", e))?;

    Ok(())
}
