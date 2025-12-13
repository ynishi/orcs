use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use serde::{Deserialize, Serialize};
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
    use orcs_core::workspace::manager::WorkspaceStorageService;

    let workspace = match state.session_usecase.active_session().await {
        Some(manager) => {
            let app_mode = state.app_mode.lock().await.clone();
            let session = manager
                .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
                .await;

            if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                state
                    .workspace_storage_service
                    .get_workspace(&session.workspace_id)
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
                        .next_back()
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

/// Result of creating a sandbox worktree
#[derive(Serialize)]
pub struct CreateSandboxResult {
    pub worktree_path: String,
    pub original_branch: String,
    pub sandbox_branch: String,
}

/// Creates a git worktree for sandbox mode
#[tauri::command]
pub async fn create_sandbox_worktree(
    session_id: String,
    sandbox_root: Option<String>,
    state: State<'_, AppState>,
) -> Result<CreateSandboxResult, String> {
    use orcs_core::workspace::manager::WorkspaceStorageService;

    // Get current workspace
    let workspace = match state.session_usecase.active_session().await {
        Some(manager) => {
            let app_mode = state.app_mode.lock().await.clone();
            let session = manager
                .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
                .await;

            if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                state
                    .workspace_storage_service
                    .get_workspace(&session.workspace_id)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                return Err("No workspace found for current session".to_string());
            }
        }
        None => return Err("No active session".to_string()),
    };

    let workspace = workspace.ok_or("Workspace not found")?;
    let working_dir = workspace.root_path.as_path();

    // Check if this is a git repository
    let is_repo = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_repo {
        return Err("Not a git repository".to_string());
    }

    // Get current branch
    let original_branch = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get current branch: {}", e))
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map(|s| s.trim().to_string())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))
            } else {
                Err("Failed to get current branch".to_string())
            }
        })?;

    // Generate sandbox branch name
    let sandbox_branch = format!("sandbox-{}", &session_id[..8]);

    // Get git root
    let git_root = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("Failed to get git root: {}", e))
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map(|s| s.trim().to_string())
                    .map_err(|e| format!("Invalid UTF-8: {}", e))
            } else {
                Err("Failed to get git root".to_string())
            }
        })?;

    // Determine sandbox root directory (default: "../")
    let sandbox_root_dir = sandbox_root.as_deref().unwrap_or("../");

    // Create worktree path based on sandbox_root
    let git_root_path = PathBuf::from(&git_root);
    let worktree_base = if sandbox_root_dir.starts_with("./") || sandbox_root_dir.starts_with(".\\")
    {
        // Relative to git root (e.g., "./.orcs-sandboxes")
        git_root_path.join(sandbox_root_dir)
    } else if sandbox_root_dir == "../" || sandbox_root_dir == ".." {
        // Parent directory
        git_root_path
            .parent()
            .ok_or_else(|| "Cannot use ../ - git root has no parent directory".to_string())?
            .join(".orcs-sandboxes")
    } else {
        // Absolute path or other relative path
        PathBuf::from(sandbox_root_dir)
    };

    let worktree_path = worktree_base.join(&sandbox_branch);

    // Create sandbox directory if it doesn't exist
    std::fs::create_dir_all(worktree_path.parent().unwrap())
        .map_err(|e| format!("Failed to create sandbox directory: {}", e))?;

    // Check if branch already exists and delete it
    let branch_check = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["rev-parse", "--verify", &sandbox_branch])
        .output()
        .map_err(|e| format!("Failed to check branch: {}", e))?;

    if branch_check.status.success() {
        // Branch exists, delete it
        ProcessCommand::new("git")
            .current_dir(working_dir)
            .args(["branch", "-D", &sandbox_branch])
            .output()
            .map_err(|e| format!("Failed to delete existing branch: {}", e))?;
    }

    // Create worktree with new branch
    let output = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args([
            "worktree",
            "add",
            "-b",
            &sandbox_branch,
            worktree_path.to_str().unwrap(),
            "HEAD",
        ])
        .output()
        .map_err(|e| format!("Failed to create worktree: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git worktree add failed: {}", stderr));
    }

    Ok(CreateSandboxResult {
        worktree_path: worktree_path.to_string_lossy().to_string(),
        original_branch,
        sandbox_branch,
    })
}

/// Options for exiting sandbox mode
#[derive(Deserialize)]
pub struct ExitSandboxOptions {
    pub worktree_path: String,
    pub original_branch: String,
    pub sandbox_branch: String,
    pub merge: bool, // true = merge changes, false = discard
}

/// Exits sandbox mode by removing worktree
#[tauri::command]
pub async fn exit_sandbox_worktree(
    options: ExitSandboxOptions,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use orcs_core::workspace::manager::WorkspaceStorageService;

    // Get current workspace
    let workspace = match state.session_usecase.active_session().await {
        Some(manager) => {
            let app_mode = state.app_mode.lock().await.clone();
            let session = manager
                .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
                .await;

            if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                state
                    .workspace_storage_service
                    .get_workspace(&session.workspace_id)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                return Err("No workspace found for current session".to_string());
            }
        }
        None => return Err("No active session".to_string()),
    };

    let workspace = workspace.ok_or("Workspace not found")?;
    let working_dir = workspace.root_path.as_path();

    if options.merge {
        // Switch to original branch first
        let checkout_output = ProcessCommand::new("git")
            .current_dir(working_dir)
            .args(["checkout", &options.original_branch])
            .output()
            .map_err(|e| format!("Failed to checkout original branch: {}", e))?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            return Err(format!("Failed to checkout original branch: {}", stderr));
        }

        // Merge sandbox branch
        let merge_output = ProcessCommand::new("git")
            .current_dir(working_dir)
            .args(["merge", &options.sandbox_branch, "--no-ff"])
            .output()
            .map_err(|e| format!("Failed to merge: {}", e))?;

        if !merge_output.status.success() {
            let stderr = String::from_utf8_lossy(&merge_output.stderr);
            return Err(format!(
                "Merge failed. Please resolve conflicts manually:\n{}",
                stderr
            ));
        }
    }

    // Remove worktree
    let remove_output = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["worktree", "remove", &options.worktree_path, "--force"])
        .output()
        .map_err(|e| format!("Failed to remove worktree: {}", e))?;

    if !remove_output.status.success() {
        let stderr = String::from_utf8_lossy(&remove_output.stderr);
        return Err(format!("Failed to remove worktree: {}", stderr));
    }

    // Delete sandbox branch
    let branch_output = ProcessCommand::new("git")
        .current_dir(working_dir)
        .args(["branch", "-D", &options.sandbox_branch])
        .output()
        .map_err(|e| format!("Failed to delete branch: {}", e))?;

    if !branch_output.status.success() {
        let stderr = String::from_utf8_lossy(&branch_output.stderr);
        // Non-fatal: log warning but don't fail
        eprintln!("Warning: Failed to delete sandbox branch: {}", stderr);
    }

    Ok(())
}
