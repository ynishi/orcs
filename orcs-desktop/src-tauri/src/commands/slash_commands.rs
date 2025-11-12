use std::process::Command;

use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::slash_command::SlashCommand;
use orcs_core::workspace::manager::WorkspaceStorageService;
use tauri::State;

use crate::app::AppState;
use crate::slash_commands::{ExpandedSlashCommand, expand_slash_command};

/// Lists all available slash commands
#[tauri::command]
pub async fn list_slash_commands(state: State<'_, AppState>) -> Result<Vec<SlashCommand>, String> {
    state
        .slash_command_repository
        .list_commands()
        .await
        .map_err(|e| e.to_string())
}

/// Gets a specific slash command by name
#[tauri::command]
pub async fn get_slash_command(
    name: String,
    state: State<'_, AppState>,
) -> Result<Option<SlashCommand>, String> {
    state
        .slash_command_repository
        .get_command(&name)
        .await
        .map_err(|e| e.to_string())
}

/// Saves a slash command (add or update)
#[tauri::command]
pub async fn save_slash_command(
    command: SlashCommand,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .slash_command_repository
        .save_command(command)
        .await
        .map_err(|e| e.to_string())
}

/// Removes a slash command by name
#[tauri::command]
pub async fn remove_slash_command(name: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .slash_command_repository
        .remove_command(&name)
        .await
        .map_err(|e| e.to_string())
}

/// Expands template variables in a command's content
#[tauri::command]
pub async fn expand_command_template(
    command_name: String,
    args: Option<String>,
    state: State<'_, AppState>,
) -> Result<ExpandedSlashCommand, String> {
    // Get the command
    let command = state
        .slash_command_repository
        .get_command(&command_name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Command not found: {}", command_name))?;

    let expanded =
        expand_slash_command(&command, args.as_deref().unwrap_or_default(), &state).await?;

    Ok(expanded)
}

/// Executes a shell command and returns the output
#[tauri::command]
pub async fn execute_shell_command(
    command: String,
    working_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("execute_shell_command: Command: {}", command);
    tracing::info!(
        "execute_shell_command: Working dir provided: {:?}",
        working_dir
    );

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &command]);
        c
    };

    // If working_dir is provided, use it. Otherwise, use workspace directory from active session
    let actual_dir = if let Some(dir) = working_dir {
        tracing::info!("execute_shell_command: Using provided dir: {}", dir);
        cmd.current_dir(&dir);
        dir
    } else {
        // Default to workspace directory from active session
        let workspace = if let Some(session_mgr) = state.session_usecase.active_session().await {
            let app_mode = state.app_mode.lock().await.clone();
            let session = session_mgr
                .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
                .await;
            if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                let workspace_id = &session.workspace_id;
                state
                    .workspace_storage_service
                    .get_workspace(workspace_id)
                    .await
                    .map_err(|e| format!("Failed to get workspace: {}", e))?
                    .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?
            } else {
                return Err("No workspace associated with current session".to_string());
            }
        } else {
            return Err("No active session".to_string());
        };

        let dir = workspace.root_path.to_string_lossy().to_string();
        tracing::info!("execute_shell_command: Using workspace dir: {}", dir);
        cmd.current_dir(&workspace.root_path);
        dir
    };

    tracing::info!("execute_shell_command: Executing in: {}", actual_dir);

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|e| format!("Failed to parse output: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command failed: {}", stderr))
    }
}
