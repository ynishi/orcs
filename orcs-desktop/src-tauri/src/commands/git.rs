use std::process::Command as ProcessCommand;

use serde::Serialize;
use tauri::State;

use crate::app::AppState;

/// Git repository information
#[derive(Serialize, Clone)]
pub struct GitInfo {
    /// Whether the current directory is in a Git repository
    pub is_repo: bool,
    /// Current branch name (if in a repo)
    pub branch: Option<String>,
    /// Repository name (if in a repo)
    pub repo_name: Option<String>,
}

/// Gets Git repository information for the current workspace
#[tauri::command]
pub async fn get_git_info(state: State<'_, AppState>) -> Result<GitInfo, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    let workspace = match state.session_manager.active_session().await {
        Some(manager) => {
            let app_mode = state.app_mode.lock().await.clone();
            let session = manager.to_session(app_mode, None).await;

            if let Some(workspace_id) = &session.workspace_id {
                state
                    .workspace_manager
                    .get_workspace(workspace_id)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                None
            }
        }
        None => None,
    };

    let working_dir = workspace
        .as_ref()
        .map(|ws| ws.root_path.as_path())
        .unwrap_or_else(|| std::path::Path::new("."));

    let is_repo = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_repo {
        return Ok(GitInfo {
            is_repo: false,
            branch: None,
            repo_name: None,
        });
    }

    let branch = ProcessCommand::new("git")
        .current_dir(working_dir)
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
        });

    let repo_name = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().and_then(|url| {
                    url.trim()
                        .split('/')
                        .last()
                        .map(|name| name.trim_end_matches(".git").to_string())
                })
            } else {
                None
            }
        })
        .or_else(|| {
            workspace.as_ref().map(|ws| ws.name.clone()).or_else(|| {
                ProcessCommand::new("git")
                    .current_dir(working_dir)
                    .args(["rev-parse", "--show-toplevel"])
                    .output()
                    .ok()
                    .and_then(|output| {
                        if output.status.success() {
                            String::from_utf8(output.stdout).ok().and_then(|path| {
                                std::path::Path::new(path.trim())
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .map(|s| s.to_string())
                            })
                        } else {
                            None
                        }
                    })
            })
        });

    Ok(GitInfo {
        is_repo: true,
        branch,
        repo_name,
    })
}


