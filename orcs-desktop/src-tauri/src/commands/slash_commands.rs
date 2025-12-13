use std::process::Command;

use orcs_core::agent::build_enhanced_path;
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::slash_command::{CreateSlashCommandRequest, SlashCommand};
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
        let (workspace, sandbox_state) = if let Some(session_mgr) = state.session_usecase.active_session().await {
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
