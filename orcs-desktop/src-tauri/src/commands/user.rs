use tauri::State;

use crate::app::AppState;

/// Gets the user's nickname from the config
#[tauri::command]
pub async fn get_user_nickname(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.user_service.get_user_name())
}

/// Gets the complete user profile from the config
#[tauri::command]
pub async fn get_user_profile(
    state: State<'_, AppState>,
) -> Result<orcs_core::user::UserProfile, String> {
    Ok(state.user_service.get_user_profile())
}

/// Gets the debug settings from the config
#[tauri::command]
pub async fn get_debug_settings(
    state: State<'_, AppState>,
) -> Result<orcs_core::config::DebugSettings, String> {
    Ok(state.user_service.get_debug_settings())
}

/// Updates the debug settings in the config
#[tauri::command]
pub async fn update_debug_settings(
    state: State<'_, AppState>,
    enable_llm_debug: bool,
    log_level: String,
) -> Result<(), String> {
    state
        .user_service
        .update_debug_settings(enable_llm_debug, log_level)
        .await
        .map_err(|e| e.to_string())
}
