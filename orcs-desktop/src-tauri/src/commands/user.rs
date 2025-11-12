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
