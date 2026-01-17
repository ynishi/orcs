use std::process::Command;

use orcs_application::SessionSupportAgentService;
use orcs_core::agent::build_enhanced_path;
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::slash_command::{CommandType, CreateSlashCommandRequest, SlashCommand};
use orcs_core::workspace::manager::WorkspaceStorageService;
use tauri::State;

use crate::app::AppState;
use crate::slash_commands::{ExpandedSlashCommand, expand_slash_command, get_git_branch, get_git_status};

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

/// Creates a new slash command from a CreateSlashCommandRequest (unified creation logic)
#[tauri::command]
pub async fn create_slash_command(
    request: CreateSlashCommandRequest,
    state: State<'_, AppState>,
) -> Result<SlashCommand, String> {
    // Validate request
    request.validate()?;

    // Convert to SlashCommand
    let command = request.into_slash_command();

    // Check for duplicate name
    if let Ok(Some(_)) = state
        .slash_command_repository
        .get_command(&command.name)
        .await
    {
        return Err(format!("Slash command '{}' already exists", command.name));
    }

    // Save to repository
    state
        .slash_command_repository
        .save_command(command.clone())
        .await
        .map_err(|e| format!("Failed to save slash command: {}", e))?;

    Ok(command)
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

/// Executes a task workflow command
#[tauri::command]
pub async fn execute_task_command(
    command_name: String,
    args: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use orcs_core::slash_command::CommandType;

    // Get the command
    let command = state
        .slash_command_repository
        .get_command(&command_name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Command not found: {}", command_name))?;

    // Verify it's a Task type
    if command.command_type != CommandType::Task {
        return Err(format!(
            "Command '{}' is not a task command (type: {:?})",
            command_name, command.command_type
        ));
    }

    // Note: task_blueprint is currently not used by TaskExecutor::execute_from_message
    // The executor uses BlueprintWorkflow::new(message_content) internally
    // Future enhancement: Pass blueprint to executor if provided

    // Get active session information for task execution
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let app_mode = state.app_mode.lock().await.clone();
    let session = manager
        .to_session(
            app_mode,
            orcs_core::session::PLACEHOLDER_WORKSPACE_ID.to_string(),
        )
        .await;
    let session_id = session.id.clone();
    let workspace_id = &session.workspace_id;

    // Get workspace root path from workspace_id
    let workspace_root = if workspace_id != orcs_core::session::PLACEHOLDER_WORKSPACE_ID {
        match state
            .workspace_storage_service
            .get_workspace(workspace_id)
            .await
        {
            Ok(Some(workspace)) => Some(workspace.root_path),
            Ok(None) => {
                tracing::warn!("Workspace not found for id: {}, using None", workspace_id);
                None
            }
            Err(e) => {
                tracing::warn!("Failed to get workspace: {}, using None", e);
                None
            }
        }
    } else {
        None
    };

    // Prepare message content for task execution
    let message_content = args.unwrap_or_else(|| command.content.clone());

    // Execute task using TaskExecutor (same as execute_message_as_task)
    state
        .task_executor
        .execute_from_message(session_id, message_content, workspace_root)
        .await
        .map_err(|e| e.to_string())
}

/// Executes an action command and returns the result.
///
/// Action commands use `content` as a prompt template with variables:
/// - Session: `{session_all}`, `{session_recent}`
/// - Workspace: `{workspace}`, `{workspace_path}`, `{files}`, `{git_branch}`, `{git_status}`
/// - Runtime: `{args}`
///
/// The expanded prompt is sent to AI and the result is returned.
#[tauri::command]
pub async fn execute_action_command(
    command_name: String,
    thread_content: String,
    args: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the command
    let command = state
        .slash_command_repository
        .get_command(&command_name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Command not found: {}", command_name))?;

    // Verify it's an Action type
    if command.command_type != CommandType::Action {
        return Err(format!(
            "Command '{}' is not an action command (type: {:?})",
            command_name, command.command_type
        ));
    }

    // Get action config (optional)
    let config = command.action_config.as_ref();

    tracing::info!(
        "[SlashCommand] Executing action command '{}' with config: {:?}",
        command_name,
        config
    );

    // Expand template variables in content
    let mut prompt = command.content.clone();

    // Session variables
    // {session_all} - Full conversation
    prompt = prompt.replace("{session_all}", &thread_content);

    // {session_recent} - Recent messages (last 10 messages based on delimiter)
    let session_recent = extract_recent_messages(&thread_content, 10);
    prompt = prompt.replace("{session_recent}", &session_recent);

    // {args} - User arguments
    let args_str = args.as_deref().unwrap_or("");
    prompt = prompt.replace("{args}", args_str);

    // Workspace variables (if available)
    if let Some(session_mgr) = state.session_usecase.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = session_mgr
            .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
            .await;

        if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
            if let Ok(Some(workspace)) = state
                .workspace_storage_service
                .get_workspace(&session.workspace_id)
                .await
            {
                // {workspace} - Workspace name
                prompt = prompt.replace("{workspace}", &workspace.name);

                // {workspace_path} - Workspace root path
                prompt = prompt.replace(
                    "{workspace_path}",
                    workspace.root_path.to_str().unwrap_or(""),
                );

                // {files} - Uploaded files list
                let file_list = workspace
                    .resources
                    .uploaded_files
                    .iter()
                    .map(|f| format!("- {} ({})", f.name, f.path.display()))
                    .collect::<Vec<_>>()
                    .join("\n");
                prompt = prompt.replace("{files}", &file_list);

                // {git_branch} - Current git branch
                let git_branch = get_git_branch(Some(&workspace.root_path))
                    .unwrap_or_else(|| "unknown".to_string());
                prompt = prompt.replace("{git_branch}", &git_branch);

                // {git_status} - Git status
                let git_status = get_git_status(Some(&workspace.root_path))
                    .unwrap_or_else(|| "unavailable".to_string());
                prompt = prompt.replace("{git_status}", &git_status);
            }
        }
    }

    tracing::debug!(
        "[SlashCommand] Expanded prompt length: {} chars",
        prompt.len()
    );

    // Execute with configured backend
    let backend = config
        .and_then(|c| c.backend.as_deref())
        .unwrap_or("gemini_api");
    let model_name = config.and_then(|c| c.model_name.as_deref());
    let thinking_level = config.and_then(|c| c.gemini_thinking_level.as_deref());
    let google_search = config.and_then(|c| c.gemini_google_search);

    let result = SessionSupportAgentService::execute_custom_prompt(
        &prompt,
        backend,
        model_name,
        thinking_level,
        google_search,
        Some(state.cancel_flag.clone()),
    )
    .await
    .map_err(|e| format!("Failed to execute action: {}", e))?;

    tracing::info!(
        "[SlashCommand] Action command '{}' completed successfully",
        command_name
    );

    Ok(result)
}

/// Extract recent messages from thread content
///
/// Messages are separated by "---\n" delimiter.
/// Returns the last N messages joined together.
fn extract_recent_messages(thread_content: &str, count: usize) -> String {
    let messages: Vec<&str> = thread_content.split("---\n").collect();
    let recent: Vec<&str> = messages.iter().rev().take(count).rev().copied().collect();
    recent.join("---\n")
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
    let (actual_dir, workspace_root) = if let Some(dir) = working_dir {
        tracing::info!("execute_shell_command: Using provided dir: {}", dir);
        let path = std::path::PathBuf::from(&dir);
        cmd.current_dir(&path);
        (dir, path)
    } else {
        // Default to workspace directory from active session
        let (workspace, sandbox_state) =
            if let Some(session_mgr) = state.session_usecase.active_session().await {
                let app_mode = state.app_mode.lock().await.clone();
                let session = session_mgr
                    .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
                    .await;

                let sandbox = session.sandbox_state.clone();

                if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                    let workspace_id = &session.workspace_id;
                    let ws = state
                        .workspace_storage_service
                        .get_workspace(workspace_id)
                        .await
                        .map_err(|e| format!("Failed to get workspace: {}", e))?
                        .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;
                    (ws, sandbox)
                } else {
                    return Err("No workspace associated with current session".to_string());
                }
            } else {
                return Err("No active session".to_string());
            };

        // Check if session is in sandbox mode
        if let Some(sandbox) = sandbox_state {
            let sandbox_path = std::path::PathBuf::from(&sandbox.worktree_path);
            let dir = sandbox_path.to_string_lossy().to_string();
            tracing::info!("execute_shell_command: Using sandbox worktree dir: {}", dir);
            cmd.current_dir(&sandbox_path);
            (dir, sandbox_path)
        } else {
            let dir = workspace.root_path.to_string_lossy().to_string();
            tracing::info!("execute_shell_command: Using workspace dir: {}", dir);
            cmd.current_dir(&workspace.root_path);
            (dir, workspace.root_path)
        }
    };

    // Build enhanced PATH from workspace root (without env_settings for now)
    // This includes workspace-specific tool dirs, system paths, and common binary locations
    let enhanced_path = build_enhanced_path(&workspace_root, None);

    // Set enhanced PATH as environment variable
    cmd.env("PATH", enhanced_path);

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
