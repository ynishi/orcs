//! Tauri commands for slash command management

use std::process::Command;

use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::slash_command::{CommandType, SlashCommand};
use orcs_core::workspace::manager::WorkspaceStorageService;
use serde::Serialize;
use tauri::State;

use crate::app::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpandedSlashCommand {
    pub content: String,
    pub working_dir: Option<String>,
}

pub async fn expand_slash_command(
    command: &SlashCommand,
    args: &str,
    state: &State<'_, AppState>,
) -> Result<ExpandedSlashCommand, String> {
    tracing::info!("expand_slash_command: Command name: {}", command.name);
    tracing::info!(
        "expand_slash_command: Command type: {:?}",
        command.command_type
    );
    tracing::info!(
        "expand_slash_command: Working dir (raw): {:?}",
        command.working_dir
    );

    let trimmed_args = args.trim();

    let mut content = match command.command_type {
        CommandType::Prompt => {
            if command.content.contains("{args}") {
                command.content.replace("{args}", trimmed_args)
            } else if !trimmed_args.is_empty() {
                format!("{}\n\n{}", command.content, trimmed_args)
            } else {
                command.content.clone()
            }
        }
        CommandType::Shell => {
            if command.content.contains("{args}") {
                command.content.replace("{args}", trimmed_args)
            } else {
                command.content.clone()
            }
        }
    };

    // Get workspace info from active session
    let workspace = if let Some(session_mgr) = state.session_usecase.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = session_mgr
            .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
            .await;
        if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
            let workspace_id = &session.workspace_id;
            tracing::info!(
                "expand_slash_command: Active session workspace_id: {}",
                workspace_id
            );
            let ws = state
                .workspace_storage_service
                .get_workspace(workspace_id)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;
            tracing::info!(
                "expand_slash_command: Workspace name: {}, root_path: {}",
                ws.name,
                ws.root_path.display()
            );
            ws
        } else {
            return Err("No workspace associated with current session".to_string());
        }
    } else {
        return Err("No active session".to_string());
    };

    let files = workspace.resources.uploaded_files.clone();

    let file_list = files
        .iter()
        .map(|f| format!("- {} ({})", f.name, f.path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    // Get git info from workspace directory
    let git_branch =
        get_git_branch(Some(&workspace.root_path)).unwrap_or_else(|| "unknown".to_string());
    let git_status =
        get_git_status(Some(&workspace.root_path)).unwrap_or_else(|| "unavailable".to_string());

    // Replace common placeholders in content
    content = content.replace("{workspace}", &workspace.name);
    content = content.replace(
        "{workspace_path}",
        workspace.root_path.to_str().unwrap_or(""),
    );
    content = content.replace("{files}", &file_list);
    content = content.replace("{git_branch}", &git_branch);
    content = content.replace("{git_status}", &git_status);
    if content.contains("{args}") {
        content = content.replace("{args}", trimmed_args);
    }

    // Expand working directory if provided
    let working_dir = command.working_dir.as_ref().map(|dir| {
        let mut expanded = dir.clone();
        tracing::info!("expand_slash_command: Expanding working_dir from: {}", dir);
        expanded = expanded.replace("{workspace}", &workspace.name);
        expanded = expanded.replace(
            "{workspace_path}",
            workspace.root_path.to_str().unwrap_or(""),
        );
        expanded = expanded.replace("{git_branch}", &git_branch);
        expanded = expanded.replace("{git_status}", &git_status);
        if expanded.contains("{args}") {
            expanded = expanded.replace("{args}", trimmed_args);
        }
        tracing::info!(
            "expand_slash_command: Expanded working_dir to: {}",
            expanded
        );
        expanded
    });

    tracing::info!("expand_slash_command: Final working_dir: {:?}", working_dir);

    Ok(ExpandedSlashCommand {
        content,
        working_dir,
    })
}

// Helper functions

fn get_git_branch(working_dir: Option<&std::path::Path>) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--abbrev-ref", "HEAD"]);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.output().ok().and_then(|output| {
        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    })
}

fn get_git_status(working_dir: Option<&std::path::Path>) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(["status", "--short"]);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.output().ok().and_then(|output| {
        if output.status.success() {
            String::from_utf8(output.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    })
}
