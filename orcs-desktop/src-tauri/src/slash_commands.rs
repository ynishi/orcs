//! Tauri commands for slash command management

use orcs_core::slash_command::SlashCommand;
use orcs_core::workspace::manager::WorkspaceManager;
use std::process::Command;
use tauri::State;

use crate::AppState;

/// Lists all available slash commands
#[tauri::command]
pub async fn list_slash_commands(
    state: State<'_, AppState>,
) -> Result<Vec<SlashCommand>, String> {
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
pub async fn remove_slash_command(
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the command
    let command = state
        .slash_command_repository
        .get_command(&command_name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Command not found: {}", command_name))?;

    // Get workspace info
    let workspace = state
        .workspace_manager
        .get_current_workspace()
        .await
        .map_err(|e| e.to_string())?;

    // Get workspace files
    let files = workspace.resources.uploaded_files.clone();

    let file_list = files
        .iter()
        .map(|f| format!("- {} ({})", f.name, f.path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    // Get git info
    let git_branch = get_git_branch().unwrap_or_else(|| "unknown".to_string());
    let git_status = get_git_status().unwrap_or_else(|| "unavailable".to_string());

    // Expand variables
    let mut expanded = command.content.clone();
    expanded = expanded.replace("{workspace}", &workspace.name);
    expanded = expanded.replace(
        "{workspace_path}",
        workspace.root_path.to_str().unwrap_or(""),
    );
    expanded = expanded.replace("{files}", &file_list);
    expanded = expanded.replace("{git_branch}", &git_branch);
    expanded = expanded.replace("{git_status}", &git_status);

    Ok(expanded)
}

/// Executes a shell command and returns the output
#[tauri::command]
pub async fn execute_shell_command(
    command: String,
    working_dir: Option<String>,
) -> Result<String, String> {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", &command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", &command]);
        c
    };

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse output: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Command failed: {}", stderr))
    }
}

// Helper functions

fn get_git_branch() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn get_git_status() -> Option<String> {
    Command::new("git")
        .args(["status", "--short"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}
