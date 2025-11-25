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
// ============================================================================
// Tab management commands
// ============================================================================

/// Opens a tab for the given session. If a tab for this session already exists,
/// returns the existing tab ID and sets it as active. Otherwise creates a new tab.
#[tauri::command]
pub async fn open_tab(
    session_id: String,
    workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let tab_id = state
        .app_state_service
        .open_tab(session_id, workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    let updated_state = state
        .app_state_service
        .get_state()
        .await
        .map_err(|e| e.to_string())?;

    app.emit("app-state:update", &updated_state)
        .map_err(|e| format!("Failed to emit app-state:update: {}", e))?;

    Ok(tab_id)
}

/// Closes a tab by its ID.
#[tauri::command]
pub async fn close_tab(
    tab_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .close_tab(tab_id)
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

/// Sets the active tab by its ID.
#[tauri::command]
pub async fn set_active_tab(
    tab_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .set_active_tab(tab_id)
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

/// Reorders tabs by moving a tab from one index to another.
#[tauri::command]
pub async fn reorder_tabs(
    from_index: usize,
    to_index: usize,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .reorder_tabs(from_index, to_index)
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

/// Updates tab UI state (input, attached files, AutoChat state, dirty flag).
/// This is a memory-only update for frequent changes (e.g., text input).
/// The state will be persisted to disk on app shutdown.
#[tauri::command]
pub async fn update_tab_ui_state(
    tab_id: String,
    input: Option<String>,
    attached_file_paths: Option<Vec<String>>,
    auto_mode: Option<bool>,
    auto_chat_iteration: Option<i32>,
    is_dirty: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .app_state_service
        .update_tab_ui_state(
            tab_id,
            input,
            attached_file_paths,
            auto_mode,
            auto_chat_iteration,
            is_dirty,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Memory-only update, no app-state:update event emission
    // (to avoid excessive event traffic for frequent updates like text input)

    Ok(())
}
