use std::env;

use orcs_infrastructure::paths::{OrcsPaths, ServiceType};
use orcs_infrastructure::storage_repository::StorageRepository;
use tauri::State;

use crate::app::AppState;

/// Gets the path to the configuration file
#[tauri::command]
pub async fn get_config_path(_state: State<'_, AppState>) -> Result<String, String> {
    let path_type = OrcsPaths::new(None)
        .get_path(ServiceType::Config)
        .map_err(|e| e.to_string())?;
    let config_file = path_type.into_path_buf();

    let path_str = config_file
        .to_str()
        .ok_or("Config path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the sessions directory path
#[tauri::command]
pub async fn get_sessions_directory(state: State<'_, AppState>) -> Result<String, String> {
    let sessions_dir = state.session_repository.base_dir();

    let path_str = sessions_dir
        .to_str()
        .ok_or("Sessions directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the workspaces directory path
#[tauri::command]
pub async fn get_workspaces_directory(state: State<'_, AppState>) -> Result<String, String> {
    let workspaces_dir = state.workspace_manager.workspaces_root_path();

    let path_str = workspaces_dir
        .to_str()
        .ok_or("Workspaces directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the workspaces directory path
#[tauri::command]
pub async fn get_workspaces_repository_directory(state: State<'_, AppState>) -> Result<String, String> {
    let workspaces_dir = state.workspace_manager.workspace_data_path();

    let path_str = workspaces_dir
        .to_str()
        .ok_or("Workspaces directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the personas directory path
#[tauri::command]
pub async fn get_personas_directory(state: State<'_, AppState>) -> Result<String, String> {
    let personas_dir = state.persona_repository_concrete.base_dir();

    let path_str = personas_dir
        .to_str()
        .ok_or("Personas directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the slash commands directory path
#[tauri::command]
pub async fn get_slash_commands_directory(state: State<'_, AppState>) -> Result<String, String> {
    let slash_commands_dir = state.slash_command_repository_concrete.base_dir();

    let path_str = slash_commands_dir
        .to_str()
        .ok_or("Slash commands directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the root directory where the application is running from
#[tauri::command]
pub async fn get_root_pathectory() -> Result<String, String> {
    let current_dir =
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let path_str = current_dir
        .to_str()
        .ok_or("Root directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the logs directory path
#[tauri::command]
pub async fn get_logs_directory() -> Result<String, String> {
    let path_type = OrcsPaths::new(None)
        .get_path(ServiceType::Logs)
        .map_err(|e| e.to_string())?;
    let logs_dir = path_type.into_path_buf();

    let path_str = logs_dir
        .to_str()
        .ok_or("Logs directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the secret file path (~/.config/orcs/secret.json)
/// Creates the file with a template if it doesn't exist
#[tauri::command]
pub async fn get_secret_path() -> Result<String, String> {
    let path_type = OrcsPaths::new(None)
        .get_path(ServiceType::Secret)
        .map_err(|e| e.to_string())?;
    let secret_file = path_type.into_path_buf();

    let path_str = secret_file
        .to_str()
        .ok_or("Secret file path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the default workspace path (~/orcs)
#[tauri::command]
pub async fn get_default_workspace_path() -> Result<String, String> {
    let orcs_paths = OrcsPaths::new(None);
    let default_path = orcs_paths
        .default_user_workspace_path()
        .map_err(|e| e.to_string())?;

    let path_str = default_path
        .to_str()
        .ok_or("Default workspace path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

