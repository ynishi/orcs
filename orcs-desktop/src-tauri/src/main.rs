// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod slash_commands;

use llm_toolkit::agent::dialogue::ExecutionModel;
use orcs_application::SessionUseCase;
use orcs_core::persona::{Persona, get_default_presets};
use orcs_core::repository::{PersonaRepository, TaskRepository};
use orcs_core::session::{AppMode, ErrorSeverity, Session, SessionManager};
use orcs_core::slash_command::SlashCommandRepository;
use orcs_core::user::UserService;
use orcs_core::workspace::{UploadedFile, Workspace};
use orcs_execution::TaskExecutor;
use orcs_infrastructure::user_service::ConfigBasedUserService;
use orcs_infrastructure::workspace_manager::FileSystemWorkspaceManager;
use orcs_infrastructure::{
    AppStateService, AsyncDirPersonaRepository, AsyncDirSessionRepository,
    AsyncDirSlashCommandRepository, AsyncDirTaskRepository,
};
use orcs_interaction::{InteractionManager, InteractionResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::process::Command;
use tokio::sync::Mutex;

/// Application state shared across Tauri commands
struct AppState {
    session_usecase: Arc<SessionUseCase>,
    session_manager: Arc<SessionManager<InteractionManager>>,
    session_repository: Arc<AsyncDirSessionRepository>,
    app_mode: Mutex<AppMode>,
    persona_repository: Arc<dyn PersonaRepository>,
    persona_repository_concrete: Arc<AsyncDirPersonaRepository>,
    user_service: Arc<dyn UserService>,
    workspace_manager: Arc<FileSystemWorkspaceManager>,
    slash_command_repository: Arc<dyn SlashCommandRepository>,
    slash_command_repository_concrete: Arc<AsyncDirSlashCommandRepository>,
    app_state_service: Arc<AppStateService>,
    task_repository: Arc<dyn TaskRepository>,
    task_repository_concrete: Arc<AsyncDirTaskRepository>,
    task_executor: Arc<TaskExecutor>,
}

/// Serializable version of DialogueMessage for Tauri IPC
#[derive(Serialize, Clone)]
pub struct SerializableDialogueMessage {
    pub author: String,
    pub content: String,
}

/// Payload for persisting system messages emitted by frontend-only actions.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedSystemMessage {
    content: String,
    #[serde(default)]
    message_type: Option<String>,
    #[serde(default)]
    severity: Option<String>,
}

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

/// Serializable version of InteractionResult for Tauri IPC
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
enum SerializableInteractionResult {
    /// A new message to be displayed to the user
    NewMessage(String),
    /// The application mode has changed
    ModeChanged(AppMode),
    /// Tasks to be dispatched for execution
    TasksToDispatch { tasks: Vec<String> },
    /// New dialogue messages from multiple participants
    NewDialogueMessages(Vec<SerializableDialogueMessage>),
    /// No operation occurred
    NoOp,
}

impl From<InteractionResult> for SerializableInteractionResult {
    fn from(result: InteractionResult) -> Self {
        match result {
            InteractionResult::NewMessage(msg) => SerializableInteractionResult::NewMessage(msg),
            InteractionResult::ModeChanged(mode) => {
                SerializableInteractionResult::ModeChanged(mode)
            }
            InteractionResult::TasksToDispatch { tasks } => {
                SerializableInteractionResult::TasksToDispatch { tasks }
            }
            InteractionResult::NewDialogueMessages(messages) => {
                let serializable_messages = messages
                    .into_iter()
                    .map(|msg| SerializableDialogueMessage {
                        author: msg.author,
                        content: msg.content,
                    })
                    .collect();
                SerializableInteractionResult::NewDialogueMessages(serializable_messages)
            }
            InteractionResult::NoOp => SerializableInteractionResult::NoOp,
        }
    }
}

/// Creates a new session
#[tauri::command]
async fn create_session(state: State<'_, AppState>) -> Result<Session, String> {
    // Use SessionUseCase for proper workspace association
    let session = state
        .session_usecase
        .create_session()
        .await
        .map_err(|e| e.to_string())?;

    // Reset app mode
    *state.app_mode.lock().await = AppMode::Idle;

    Ok(session)
}

/// Creates a new config session with system prompt in admin workspace
#[tauri::command]
async fn create_config_session(
    workspace_root_path: String,
    system_prompt: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    // Use SessionUseCase for proper workspace and system prompt setup
    let session = state
        .session_usecase
        .create_config_session(workspace_root_path, system_prompt)
        .await
        .map_err(|e| e.to_string())?;

    // Reset app mode
    *state.app_mode.lock().await = AppMode::Idle;

    Ok(session)
}

/// Lists all saved sessions with enriched participants
#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let sessions = state
        .session_manager
        .list_sessions()
        .await
        .map_err(|e| e.to_string())?;

    // Enrich each session's participants field
    let enriched_sessions = sessions
        .into_iter()
        .map(|session| state.session_usecase.enrich_session_participants(session))
        .collect();

    Ok(enriched_sessions)
}

/// Lists all saved tasks
#[tauri::command]
async fn list_tasks(state: State<'_, AppState>) -> Result<Vec<orcs_core::task::Task>, String> {
    state
        .task_repository
        .list_all()
        .await
        .map_err(|e| e.to_string())
}

/// Switches to a different session
#[tauri::command]
async fn switch_session(session_id: String, state: State<'_, AppState>) -> Result<Session, String> {
    // Use SessionUseCase for proper workspace context restoration
    let session = state
        .session_usecase
        .switch_session(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    // Update app mode
    *state.app_mode.lock().await = session.app_mode.clone();

    Ok(session)
}

/// Deletes a session
#[tauri::command]
async fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .session_manager
        .delete_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Renames a session
#[tauri::command]
async fn rename_session(
    session_id: String,
    new_title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_manager
        .rename_session(&session_id, new_title)
        .await
        .map_err(|e| e.to_string())
}

/// Saves the current session
#[tauri::command]
async fn save_current_session(state: State<'_, AppState>) -> Result<(), String> {
    let app_mode = state.app_mode.lock().await.clone();
    state
        .session_manager
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())
}

/// Appends system messages to the active session and persists immediately.
#[tauri::command]
async fn append_system_messages(
    messages: Vec<PersistedSystemMessage>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or_else(|| "No active session".to_string())?;

    for message in messages {
        let PersistedSystemMessage {
            content,
            message_type,
            severity,
        } = message;

        let severity_enum =
            severity
                .as_ref()
                .map(|s| s.to_lowercase())
                .and_then(|level| match level.as_str() {
                    "error" => Some(ErrorSeverity::Critical),
                    "warning" => Some(ErrorSeverity::Warning),
                    "info" => Some(ErrorSeverity::Info),
                    _ => None,
                });

        manager
            .add_system_conversation_message(content, message_type, severity_enum)
            .await;
    }

    let app_mode = state.app_mode.lock().await.clone();
    state
        .session_manager
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())
}

/// Gets the currently active session
#[tauri::command]
async fn get_active_session(state: State<'_, AppState>) -> Result<Option<Session>, String> {
    if let Some(manager) = state.session_manager.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = manager.to_session(app_mode, None).await;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

/// Gets all personas from the single source of truth
#[tauri::command]
async fn get_personas(state: State<'_, AppState>) -> Result<Vec<Persona>, String> {
    state.persona_repository.get_all()
}

/// Saves persona configurations
#[tauri::command]
async fn save_persona_configs(
    configs: Vec<Persona>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save the configurations
    state.persona_repository.save_all(&configs)?;

    // Invalidate the dialogue so it will be recreated with new persona settings
    if let Some(manager) = state.session_manager.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(())
}

/// Gets all available persona backend options
#[tauri::command]
async fn get_persona_backend_options() -> Result<Vec<(String, String)>, String> {
    use orcs_core::persona::PersonaBackend;
    Ok(PersonaBackend::all_variants())
}

/// Gets the user's nickname from the config
#[tauri::command]
async fn get_user_nickname(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.user_service.get_user_name())
}

/// Executes a message content as a task using TaskExecutor
#[tauri::command]
async fn execute_message_as_task(
    message_content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Get the active session to retrieve session_id
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let app_mode = state.app_mode.lock().await.clone();
    let session = manager.to_session(app_mode, None).await;
    let session_id = session.id;

    state
        .task_executor
        .execute_from_message(session_id, message_content)
        .await
        .map_err(|e| e.to_string())
}

/// Adds a participant to the active session
#[tauri::command]
async fn add_participant(persona_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .add_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    // Auto-save after adding participant
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Removes a participant from the active session
#[tauri::command]
async fn remove_participant(persona_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .remove_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    // Auto-save after removing participant
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the list of active participants in the current session
#[tauri::command]
async fn get_active_participants(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.get_active_participants().await
}

/// Sets the execution strategy for the active session
#[tauri::command]
async fn set_execution_strategy(
    strategy: ExecutionModel,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.set_execution_strategy(strategy).await;

    // Auto-save after changing strategy
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current execution strategy for the active session
#[tauri::command]
async fn get_execution_strategy(state: State<'_, AppState>) -> Result<ExecutionModel, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    Ok(manager.get_execution_strategy().await)
}

/// Sets the conversation mode for the active session
#[tauri::command]
async fn set_conversation_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    use orcs_core::session::ConversationMode;

    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    // Parse mode string to ConversationMode enum
    let conversation_mode = match mode.as_str() {
        "normal" => ConversationMode::Normal,
        "concise" => ConversationMode::Concise,
        "brief" => ConversationMode::Brief,
        "discussion" => ConversationMode::Discussion,
        _ => return Err(format!("Unknown conversation mode: {}", mode)),
    };

    manager.set_conversation_mode(conversation_mode).await;

    // Auto-save after changing mode
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current conversation mode for the active session
#[tauri::command]
async fn get_conversation_mode(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let mode = manager.get_conversation_mode().await;
    let mode_str = match mode {
        orcs_core::session::ConversationMode::Normal => "normal",
        orcs_core::session::ConversationMode::Concise => "concise",
        orcs_core::session::ConversationMode::Brief => "brief",
        orcs_core::session::ConversationMode::Discussion => "discussion",
    };

    Ok(mode_str.to_string())
}

/// Sets the talk style for the active session
#[tauri::command]
async fn set_talk_style(style: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    use llm_toolkit::agent::dialogue::TalkStyle;

    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    // Parse style string to TalkStyle enum
    let talk_style = if let Some(s) = style {
        match s.as_str() {
            "brainstorm" => Some(TalkStyle::Brainstorm),
            "casual" => Some(TalkStyle::Casual),
            "decision_making" => Some(TalkStyle::DecisionMaking),
            "debate" => Some(TalkStyle::Debate),
            "problem_solving" => Some(TalkStyle::ProblemSolving),
            "review" => Some(TalkStyle::Review),
            "planning" => Some(TalkStyle::Planning),
            _ => return Err(format!("Unknown talk style: {}", s)),
        }
    } else {
        None
    };

    manager.set_talk_style(talk_style).await;

    // Auto-save after changing style
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current talk style for the active session
#[tauri::command]
async fn get_talk_style(state: State<'_, AppState>) -> Result<Option<String>, String> {
    use llm_toolkit::agent::dialogue::TalkStyle;

    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let style = manager.get_talk_style().await;
    let style_str = style.map(|s| {
        match s {
            TalkStyle::Brainstorm => "brainstorm",
            TalkStyle::Casual => "casual",
            TalkStyle::DecisionMaking => "decision_making",
            TalkStyle::Debate => "debate",
            TalkStyle::ProblemSolving => "problem_solving",
            TalkStyle::Review => "review",
            TalkStyle::Planning => "planning",
        }
        .to_string()
    });

    Ok(style_str)
}

/// Gets the path to the configuration file, creating it if it doesn't exist
#[tauri::command]
async fn get_config_path(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual config file path from the repository
    let config_file = state.persona_repository_concrete.config_file_path();

    // Convert PathBuf to String
    let path_str = config_file
        .to_str()
        .ok_or("Config path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the sessions directory path
#[tauri::command]
async fn get_sessions_directory(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual sessions directory path from the repository
    let sessions_dir = state.session_repository.sessions_dir();

    // Convert Path to String
    let path_str = sessions_dir
        .to_str()
        .ok_or("Sessions directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the workspaces directory path
#[tauri::command]
async fn get_workspaces_directory(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual workspaces root directory path from the manager
    let workspaces_dir = state.workspace_manager.workspaces_root_dir();

    // Convert Path to String
    let path_str = workspaces_dir
        .to_str()
        .ok_or("Workspaces directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the personas directory path
#[tauri::command]
async fn get_personas_directory(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual personas directory path from the repository
    let personas_dir = state.persona_repository_concrete.personas_dir();

    // Convert Path to String
    let path_str = personas_dir
        .to_str()
        .ok_or("Personas directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the slash commands directory path
#[tauri::command]
async fn get_slash_commands_directory(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual slash commands directory path from the repository
    let slash_commands_dir = state.slash_command_repository_concrete.slash_commands_dir();

    // Convert Path to String
    let path_str = slash_commands_dir
        .to_str()
        .ok_or("Slash commands directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the tasks directory path
#[tauri::command]
async fn get_tasks_directory(state: State<'_, AppState>) -> Result<String, String> {
    // Get the actual tasks directory path from the repository
    let tasks_dir = state.task_repository_concrete.tasks_dir();

    // Convert Path to String
    let path_str = tasks_dir
        .to_str()
        .ok_or("Tasks directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the root directory where the application is running from
#[tauri::command]
async fn get_root_directory() -> Result<String, String> {
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let path_str = current_dir
        .to_str()
        .ok_or("Root directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the logs directory path
#[tauri::command]
async fn get_logs_directory() -> Result<String, String> {
    use orcs_infrastructure::paths::OrcsPaths;

    let logs_dir = OrcsPaths::logs_dir().map_err(|e| e.to_string())?;

    let path_str = logs_dir
        .to_str()
        .ok_or("Logs directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Gets the secret file path (~/.config/orcs/secret.json)
/// Creates the file with a template if it doesn't exist
#[tauri::command]
async fn get_secret_path() -> Result<String, String> {
    use orcs_infrastructure::paths::OrcsPaths;

    // Ensure the secret file exists (creates template if missing)
    let secret_file = OrcsPaths::ensure_secret_file().map_err(|e| e.to_string())?;

    let path_str = secret_file
        .to_str()
        .ok_or("Secret file path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Helper function to execute shell commands
async fn execute_shell_command(command: &str, working_dir: Option<&str>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let shell = "cmd";
    #[cfg(target_os = "windows")]
    let shell_arg = "/C";

    #[cfg(not(target_os = "windows"))]
    let shell = "sh";
    #[cfg(not(target_os = "windows"))]
    let shell_arg = "-c";

    let mut cmd = Command::new(shell);
    cmd.arg(shell_arg).arg(command);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if stderr.is_empty() {
            Ok(stdout.to_string())
        } else {
            Ok(format!("{}\n\nStderr:\n{}", stdout, stderr))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "Command failed with exit code {:?}:\n{}",
            output.status.code(),
            stderr
        ))
    }
}

/// Handles user input
#[tauri::command]
async fn handle_input(
    input: String,
    file_paths: Option<Vec<String>>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    // Get the current mode
    let current_mode = state.app_mode.lock().await.clone();

    // Check if input is a slash command
    let processed_input = if input.trim().starts_with('/') {
        // Extract command name (everything after / until first space or end)
        let trimmed = input.trim();
        let cmd_end = trimmed.find(' ').unwrap_or(trimmed.len());
        let cmd_name = &trimmed[1..cmd_end]; // Skip the '/'
        let args = if cmd_end < trimmed.len() {
            trimmed[cmd_end..].trim()
        } else {
            ""
        };

        eprintln!(
            "[SLASH_COMMAND] Detected command: '{}', args: '{}'",
            cmd_name, args
        );

        // Debug: List all available commands
        if let Ok(all_commands) = state.slash_command_repository.list_commands().await {
            eprintln!(
                "[SLASH_COMMAND] Available commands: {:?}",
                all_commands.iter().map(|c| &c.name).collect::<Vec<_>>()
            );
        }

        // Try to get the command from repository
        eprintln!(
            "[SLASH_COMMAND] Getting command '{}' from repository...",
            cmd_name
        );
        match state.slash_command_repository.get_command(cmd_name).await {
            Ok(Some(cmd)) => {
                use orcs_core::slash_command::CommandType;

                match cmd.command_type {
                    CommandType::Prompt => {
                        // Replace {args} placeholder if it exists
                        let expanded = if cmd.content.contains("{args}") {
                            cmd.content.replace("{args}", args)
                        } else {
                            // If no placeholder, append args if provided
                            if !args.is_empty() {
                                format!("{}\n\n{}", cmd.content, args)
                            } else {
                                cmd.content.clone()
                            }
                        };
                        expanded
                    }
                    CommandType::Shell => {
                        // Execute shell command and return output
                        let cmd_to_run = if cmd.content.contains("{args}") {
                            cmd.content.replace("{args}", args)
                        } else {
                            cmd.content.clone()
                        };

                        // TODO: Get workspace path for {workspace_path} replacement
                        let working_dir = cmd.working_dir.as_deref();

                        match execute_shell_command(&cmd_to_run, working_dir).await {
                            Ok(output) => {
                                format!("Command output:\n```\n{}\n```", output)
                            }
                            Err(e) => {
                                format!("Error executing command: {}", e)
                            }
                        }
                    }
                }
            }
            Ok(None) => {
                // Command not found, return error message
                format!(
                    "Unknown command: /{}\n\nAvailable commands can be viewed in Settings.",
                    cmd_name
                )
            }
            Err(e) => {
                format!("Error loading command: {}", e)
            }
        }
    } else {
        input.clone()
    };

    // Handle the input with streaming support
    let app_clone = app.clone();
    let result = manager
        .handle_input_with_streaming(&current_mode, &processed_input, file_paths, move |turn| {
            use tauri::Emitter;

            // Log each dialogue turn as it becomes available with timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            let preview: String = turn.content.chars().take(50).collect();
            eprintln!(
                "[TAURI] [{}.{:03}] Streaming turn: {} - {}...",
                now.as_secs(),
                now.subsec_millis(),
                turn.author,
                preview
            );

            // Emit event to frontend for real-time streaming
            if let Err(e) = app_clone.emit("dialogue-turn", turn) {
                eprintln!("[TAURI] Failed to emit dialogue-turn event: {}", e);
            }
        })
        .await;

    // Update the mode if it changed
    if let InteractionResult::ModeChanged(ref new_mode) = result {
        *state.app_mode.lock().await = new_mode.clone();
    }

    // Auto-save after each interaction
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(result.into())
}

/// Gets the current workspace based on the active session
#[tauri::command]
async fn get_current_workspace(state: State<'_, AppState>) -> Result<Workspace, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    println!("[Backend] get_current_workspace called");

    // Priority 1: Use last selected workspace from AppStateService (user's explicit selection)
    if let Some(workspace_id) = state.app_state_service.get_last_selected_workspace().await {
        println!(
            "[Backend] AppStateService last selected workspace: {}",
            workspace_id
        );
        let workspace = state
            .workspace_manager
            .get_workspace(&workspace_id)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(ws) = workspace {
            println!("[Backend] Found workspace: {} ({})", ws.name, ws.id);
            return Ok(ws);
        } else {
            println!(
                "[Backend] AppStateService workspace not found: {}",
                workspace_id
            );
        }
    }

    // Priority 2: Fallback to session's workspace_id
    println!("[Backend] No AppStateService workspace, checking session");
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let app_mode = state.app_mode.lock().await.clone();
    let session = manager.to_session(app_mode, None).await;

    println!("[Backend] Session workspace_id: {:?}", session.workspace_id);

    if let Some(workspace_id) = &session.workspace_id {
        println!("[Backend] Looking up workspace: {}", workspace_id);
        let workspace = state
            .workspace_manager
            .get_workspace(workspace_id)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(ws) = workspace {
            println!("[Backend] Found workspace: {} ({})", ws.name, ws.id);
            return Ok(ws);
        } else {
            println!("[Backend] Session workspace not found: {}", workspace_id);
        }
    }

    // No workspace found
    Err("No workspace selected or associated with session".to_string())
}

/// Creates a new workspace for the given directory path
#[tauri::command]
async fn create_workspace(
    root_path: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    use orcs_core::workspace::manager::WorkspaceManager;
    use std::path::PathBuf;

    let path = PathBuf::from(root_path);
    state
        .workspace_manager
        .get_or_create_workspace(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Lists all registered workspaces
#[tauri::command]
async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    state
        .workspace_manager
        .list_all_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Switches to a different workspace for the active session
#[tauri::command]
async fn switch_workspace(
    _session_id: String, // Kept for API compatibility but unused
    workspace_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Emitter;

    println!(
        "[Backend] switch_workspace called: workspace_id={}",
        workspace_id
    );

    // Use SessionUseCase for coordinated workspace switching
    // (SessionUseCase will handle AppStateService update)
    state
        .session_usecase
        .switch_workspace(&workspace_id)
        .await
        .map_err(|e| {
            println!("[Backend] Failed to switch workspace: {}", e);
            e.to_string()
        })?;

    println!(
        "[Backend] Successfully switched to workspace {}",
        workspace_id
    );

    // Notify frontend of the workspace switch
    app.emit("workspace-switched", &workspace_id)
        .map_err(|e| e.to_string())?;

    println!("[Backend] workspace-switched event emitted");

    Ok(())
}

/// Toggles the favorite status of a workspace
#[tauri::command]
async fn toggle_favorite_workspace(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    state
        .workspace_manager
        .toggle_favorite(&workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a workspace
#[tauri::command]
async fn delete_workspace(workspace_id: String, state: State<'_, AppState>) -> Result<(), String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    println!(
        "[Backend] delete_workspace called: workspace_id={}",
        workspace_id
    );

    state
        .workspace_manager
        .delete_workspace(&workspace_id)
        .await
        .map_err(|e| {
            println!("[Backend] Failed to delete workspace: {}", e);
            e.to_string()
        })?;

    println!("[Backend] Successfully deleted workspace {}", workspace_id);
    Ok(())
}

/// Lists all files in a workspace
#[tauri::command]
async fn list_workspace_files(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<UploadedFile>, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    // Get the workspace
    let workspace = state
        .workspace_manager
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return uploaded files or empty vector if workspace not found
    Ok(workspace
        .map(|w| w.resources.uploaded_files)
        .unwrap_or_default())
}

/// Uploads a file to a workspace
#[tauri::command]
async fn upload_file_to_workspace(
    workspace_id: String,
    local_file_path: String,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    // Convert the local file path to a Path reference
    let file_path = Path::new(&local_file_path);

    // Add the file to the workspace
    state
        .workspace_manager
        .add_file_to_workspace(&workspace_id, file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Uploads a file to a workspace from binary data
#[tauri::command]
async fn upload_file_from_bytes(
    workspace_id: String,
    filename: String,
    file_data: Vec<u8>,
    session_id: Option<String>,
    message_timestamp: Option<String>,
    author: Option<String>,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<UploadedFile, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    // Directly add the file from bytes - no temporary file needed
    let result = state
        .workspace_manager
        .add_file_from_bytes(
            &workspace_id,
            &filename,
            &file_data,
            session_id,
            message_timestamp,
            author,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Emit workspace-files-changed event
    app.emit("workspace-files-changed", &workspace_id)
        .map_err(|e| e.to_string())?;

    tracing::info!(
        "upload_file_from_bytes: Emitted workspace-files-changed event for workspace: {}",
        workspace_id
    );

    Ok(result)
}

/// Deletes a file from a workspace
#[tauri::command]
async fn delete_file_from_workspace(
    workspace_id: String,
    file_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    state
        .workspace_manager
        .delete_file_from_workspace(&workspace_id, &file_id)
        .await
        .map_err(|e| e.to_string())
}

/// Renames a file in a workspace
#[tauri::command]
async fn rename_file_in_workspace(
    workspace_id: String,
    file_id: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    state
        .workspace_manager
        .rename_file_in_workspace(&workspace_id, &file_id, &new_name)
        .await
        .map_err(|e| e.to_string())
}

/// Reads a file from a workspace and returns its content as bytes
#[tauri::command]
async fn read_workspace_file(file_path: String) -> Result<Vec<u8>, String> {
    use tokio::fs;

    fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Saves code snippet to a file
#[tauri::command]
async fn save_code_snippet(
    file_path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tokio::fs;
    use std::path::Path;

    let path = Path::new(&file_path);

    // Validate that the path is absolute
    if !path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Write the file
    fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    // Log the save operation as a system message
    if let Some(manager) = state.session_manager.active_session().await {
        manager
            .add_system_conversation_message(
                format!("Saved file: {}", file_path),
                Some("file_save".to_string()),
                None,
            )
            .await;

        // Auto-save session after file save
        let app_mode = state.app_mode.lock().await.clone();
        let _ = state.session_manager.save_active_session(app_mode).await;
    }

    Ok(())
}

/// Opens a terminal in the specified directory
#[tauri::command]
async fn open_terminal(directory: String) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Terminal", &directory])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", "cd", "/D", &directory])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try common terminal emulators in order of preference
        let terminals = [
            ("x-terminal-emulator", vec!["--working-directory", &directory]),
            ("gnome-terminal", vec!["--working-directory", &directory]),
            ("xterm", vec!["-e", &format!("cd '{}' && bash", directory)]),
        ];

        let mut success = false;
        for (terminal, args) in &terminals {
            if Command::new(terminal).args(args.iter()).spawn().is_ok() {
                success = true;
                break;
            }
        }

        if !success {
            return Err("No terminal emulator found".to_string());
        }
    }

    Ok(())
}

/// Gets Git repository information for the current workspace
#[tauri::command]
async fn get_git_info(state: State<'_, AppState>) -> Result<GitInfo, String> {
    use orcs_core::workspace::manager::WorkspaceManager;
    use std::process::Command;

    // Get the current workspace
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

    // Use workspace root_path, or fall back to current directory
    let working_dir = workspace
        .as_ref()
        .map(|ws| ws.root_path.as_path())
        .unwrap_or_else(|| std::path::Path::new("."));

    // Check if we're in a git repository
    let is_repo = Command::new("git")
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

    // Get current branch
    let branch = Command::new("git")
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

    // Get repository name from remote origin URL
    let repo_name = Command::new("git")
        .current_dir(working_dir)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().and_then(|url| {
                    // Extract repo name from URL
                    // e.g., "git@github.com:user/repo.git" -> "repo"
                    // e.g., "https://github.com/user/repo.git" -> "repo"
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
            // Fallback: use the workspace name or root directory name
            workspace.as_ref().map(|ws| ws.name.clone()).or_else(|| {
                Command::new("git")
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

fn main() {
    // Initialize logging to file
    use orcs_infrastructure::paths::OrcsPaths;

    let log_dir = OrcsPaths::logs_dir().expect("Failed to get logs directory");

    std::fs::create_dir_all(&log_dir).expect("Failed to create logs directory");

    // Custom log file naming: orcs-desktop-YYYY-MM-DD.log
    use chrono::Local;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_file_name = format!("orcs-desktop-{}.log", today);
    let log_file_path = log_dir.join(&log_file_name);

    // Use non-rotating file appender (we handle rotation manually with date in filename)
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("Failed to open log file");

    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_max_level(tracing::Level::TRACE)
        .init();

    tracing::info!("ORCS Desktop starting...");

    tauri::async_runtime::block_on(async {
        use orcs_infrastructure::paths::OrcsPaths;

        // Composition Root: Create the concrete repository instances
        let persona_repository_concrete = Arc::new(
            AsyncDirPersonaRepository::default_location()
                .await
                .expect("Failed to initialize persona repository"),
        );
        let persona_repository: Arc<dyn PersonaRepository> = persona_repository_concrete.clone();
        let user_service: Arc<dyn UserService> = Arc::new(ConfigBasedUserService::new());

        // Initialize FileSystemWorkspaceManager with unified path
        let workspace_root =
            OrcsPaths::workspaces_dir().expect("Failed to get workspaces directory");
        let workspace_manager = Arc::new(
            FileSystemWorkspaceManager::new(workspace_root)
                .await
                .expect("Failed to initialize workspace manager"),
        );

        // Initialize AsyncDirSlashCommandRepository
        let slash_command_repository_concrete = Arc::new(
            AsyncDirSlashCommandRepository::new()
                .await
                .expect("Failed to initialize slash command repository"),
        );
        let slash_command_repository: Arc<dyn SlashCommandRepository> =
            slash_command_repository_concrete.clone();

        // Seed the personas directory with default personas if it's empty on first run.
        if let Ok(personas) = persona_repository.get_all() {
            if personas.is_empty() {
                let default_presets = get_default_presets();
                if let Err(e) = persona_repository.save_all(&default_presets) {
                    // This is a critical failure on startup, so we panic.
                    panic!("Failed to seed default personas: {}", e);
                }
            }
        }

        // TODO: Ensure user profile is initialized with default if it doesn't exist
        // This will be implemented via UserProfileRepository
        // if let Err(e) = user_profile_repository.ensure_initialized() {
        //     eprintln!("Warning: Failed to initialize user profile: {}", e);
        // }

        // Create AsyncDirSessionRepository at default location
        let session_repository = Arc::new(
            AsyncDirSessionRepository::default_location(persona_repository.clone())
                .await
                .expect("Failed to create session repository"),
        );

        // Initialize SessionManager with the repository
        let session_manager: Arc<SessionManager<InteractionManager>> =
            Arc::new(SessionManager::new(session_repository.clone()));

        // Initialize AppStateService
        let app_state_service = Arc::new(
            AppStateService::new()
                .await
                .expect("Failed to initialize AppStateService"),
        );

        // Create SessionUseCase for coordinated session-workspace management
        let session_usecase = Arc::new(SessionUseCase::new(
            session_manager.clone(),
            workspace_manager.clone(),
            app_state_service.clone(),
            persona_repository.clone(),
            user_service.clone(),
        ));

        // Create Task Repository
        let task_repository_concrete = Arc::new(
            AsyncDirTaskRepository::default_location()
                .await
                .expect("Failed to initialize Task Repository"),
        );
        let task_repository = task_repository_concrete.clone() as Arc<dyn TaskRepository>;

        // Create TaskExecutor with task repository
        let task_executor = Arc::new(
            TaskExecutor::new().with_task_repository(task_repository.clone())
        );

        // Try to restore last session using SessionUseCase
        let restored = session_usecase.restore_last_session().await.ok().flatten();

        if restored.is_none() {
            // Try to restore workspace from last selected workspace ID
            let workspace_selected = if let Some(last_workspace_id) =
                app_state_service.get_last_selected_workspace().await
            {
                tracing::info!(
                    "[Startup] Found last selected workspace: {}",
                    last_workspace_id
                );

                // Try to switch to that workspace
                match session_usecase.switch_workspace(&last_workspace_id).await {
                    Ok(_) => {
                        tracing::info!(
                            "[Startup] Successfully restored workspace: {}",
                            last_workspace_id
                        );
                        true
                    }
                    Err(e) => {
                        tracing::warn!(
                            "[Startup] Failed to restore last workspace {}: {}",
                            last_workspace_id,
                            e
                        );
                        false
                    }
                }
            } else {
                false
            };

            if !workspace_selected {
                // No last workspace or failed to restore - try to find any workspace
                use orcs_core::workspace::manager::WorkspaceManager;
                match workspace_manager.list_all_workspaces().await {
                    Ok(workspaces) if !workspaces.is_empty() => {
                        // Use most recently accessed workspace
                        let most_recent = &workspaces[0];
                        tracing::info!(
                            "[Startup] Using most recent workspace: {} ({})",
                            most_recent.name,
                            most_recent.id
                        );

                        if let Err(e) = session_usecase.switch_workspace(&most_recent.id).await {
                            tracing::warn!(
                                "[Startup] Failed to switch to most recent workspace: {}",
                                e
                            );
                            // Fall back to creating a new session without workspace validation
                            tracing::info!("[Startup] Creating new session without workspace");
                        }
                    }
                    Ok(_) => {
                        // No workspaces exist - just start with empty session
                        tracing::info!(
                            "[Startup] No workspaces found, starting with empty session"
                        );
                    }
                    Err(e) => {
                        tracing::warn!("[Startup] Failed to list workspaces: {}", e);
                    }
                }
            }
        }

        let app_mode = Mutex::new(AppMode::Idle);

        // Clone session_manager for setup closure
        let session_manager_for_setup = session_manager.clone();

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init())
            .manage(AppState {
                session_usecase,
                session_manager,
                session_repository: session_repository.clone(),
                app_mode,
                persona_repository,
                persona_repository_concrete: persona_repository_concrete.clone(),
                user_service,
                workspace_manager: workspace_manager.clone(),
                slash_command_repository,
                slash_command_repository_concrete: slash_command_repository_concrete.clone(),
                app_state_service: app_state_service.clone(),
                task_repository: task_repository.clone(),
                task_repository_concrete: task_repository_concrete.clone(),
                task_executor,
            })
            .invoke_handler(tauri::generate_handler![
                create_session,
                create_config_session,
                list_sessions,
                list_tasks,
                switch_session,
                delete_session,
                rename_session,
                save_current_session,
                append_system_messages,
                get_active_session,
                get_personas,
                save_persona_configs,
                get_persona_backend_options,
                get_user_nickname,
                execute_message_as_task,
                add_participant,
                remove_participant,
                get_active_participants,
                set_execution_strategy,
                get_execution_strategy,
                set_conversation_mode,
                get_conversation_mode,
                set_talk_style,
                get_talk_style,
                get_config_path,
                get_sessions_directory,
                get_workspaces_directory,
                get_personas_directory,
                get_slash_commands_directory,
                get_tasks_directory,
                get_root_directory,
                get_logs_directory,
                get_secret_path,
                get_git_info,
                get_current_workspace,
                create_workspace,
                list_workspaces,
                switch_workspace,
                toggle_favorite_workspace,
                delete_workspace,
                list_workspace_files,
                upload_file_to_workspace,
                upload_file_from_bytes,
                delete_file_from_workspace,
                rename_file_in_workspace,
                read_workspace_file,
                save_code_snippet,
                open_terminal,
                handle_input,
                slash_commands::list_slash_commands,
                slash_commands::get_slash_command,
                slash_commands::save_slash_command,
                slash_commands::remove_slash_command,
                slash_commands::expand_command_template,
                slash_commands::execute_shell_command,
            ])
            .setup(move |app| {
                // Emit workspace-switched event after startup to trigger frontend session load
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Wait a bit to ensure frontend is ready
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    // Get current workspace from session manager
                    if let Some(session_mgr) = session_manager_for_setup.active_session().await {
                        let app_mode_locked = AppMode::Idle;
                        let session = session_mgr.to_session(app_mode_locked, None).await;
                        if let Some(workspace_id) = session.workspace_id {
                            tracing::info!(
                                "[Startup] Emitting workspace-switched event for: {}",
                                workspace_id
                            );
                            let _ = handle.emit("workspace-switched", &workspace_id);
                        }
                    }
                });
                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
