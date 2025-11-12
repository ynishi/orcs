use std::time::SystemTime;

use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use orcs_core::session::{
    AppMode, AutoChatConfig, ConversationMode, ErrorSeverity, ModeratorAction,
    PLACEHOLDER_WORKSPACE_ID, Session, SessionEvent,
};
use orcs_core::workspace::manager::WorkspaceManager;
use orcs_interaction::InteractionResult;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::process::Command;

use crate::app::AppState;

/// Serializable version of DialogueMessage for Tauri IPC
#[derive(Serialize, Clone)]
pub struct SerializableDialogueMessage {
    pub author: String,
    pub content: String,
}

/// Payload for persisting system messages emitted by frontend-only actions.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedSystemMessage {
    pub content: String,
    #[serde(default)]
    pub message_type: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
}

/// Serializable version of InteractionResult for Tauri IPC
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
pub enum SerializableInteractionResult {
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
///
/// # Arguments
///
/// * `workspace_id` - The workspace ID to associate with the new session (required)
#[tauri::command]
pub async fn create_session(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session = state
        .session_usecase
        .create_session(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    *state.app_mode.lock().await = AppMode::Idle;

    Ok(session)
}

/// Creates a new config session with system prompt in admin workspace
#[tauri::command]
pub async fn create_config_session(
    workspace_root_path: String,
    system_prompt: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session = state
        .session_usecase
        .create_config_session(workspace_root_path, system_prompt)
        .await
        .map_err(|e| e.to_string())?;

    *state.app_mode.lock().await = AppMode::Idle;

    Ok(session)
}

/// Lists all saved sessions with enriched participants
#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let sessions = state
        .session_manager
        .list_sessions()
        .await
        .map_err(|e| e.to_string())?;

    let mut enriched_sessions = Vec::new();
    for session in sessions {
        let enriched = state
            .session_usecase
            .enrich_session_participants(session)
            .await;
        enriched_sessions.push(enriched);
    }

    Ok(enriched_sessions)
}

/// Switches to a different session
#[tauri::command]
pub async fn switch_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session = state
        .session_usecase
        .switch_session(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    *state.app_mode.lock().await = session.app_mode.clone();

    Ok(session)
}

/// Deletes a session
#[tauri::command]
pub async fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .session_manager
        .delete_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Renames a session
#[tauri::command]
pub async fn rename_session(
    session_id: String,
    new_title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_metadata_service
        .rename(&session_id, new_title)
        .await
        .map_err(|e| e.to_string())
}

/// Toggles the favorite status of a session
#[tauri::command]
pub async fn toggle_session_favorite(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_metadata_service
        .toggle_favorite(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Toggles the archive status of a session
#[tauri::command]
pub async fn toggle_session_archive(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_metadata_service
        .toggle_archive(&session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Updates the manual sort order of a session
#[tauri::command]
pub async fn update_session_sort_order(
    session_id: String,
    sort_order: Option<i32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .session_metadata_service
        .update_sort_order(&session_id, sort_order)
        .await
        .map_err(|e| e.to_string())
}

/// Saves the current session
#[tauri::command]
pub async fn save_current_session(state: State<'_, AppState>) -> Result<(), String> {
    let app_mode = state.app_mode.lock().await.clone();
    state
        .session_manager
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())
}

/// Appends system messages to the active session and persists immediately.
#[tauri::command]
pub async fn append_system_messages(
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

/// Publishes a structured session event (user/system/moderator).
#[tauri::command]
pub async fn publish_session_event(
    event: SessionEvent,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    match event {
        SessionEvent::UserInput {
            content,
            attachments,
        } => {
            let paths = if attachments.is_empty() {
                None
            } else {
                Some(attachments)
            };
            handle_input(content, paths, app, state).await
        }
        SessionEvent::SystemEvent {
            content,
            message_type,
            severity,
        } => {
            // Delegate to SessionUseCase (business logic layer)
            state
                .session_usecase
                .add_system_message(content, message_type, severity)
                .await
                .map_err(|e| e.to_string())?;

            // Save the session (Tauri layer responsibility for now)
            let app_mode = state.app_mode.lock().await.clone();
            state
                .session_manager
                .save_active_session(app_mode)
                .await
                .map_err(|e| e.to_string())?;

            Ok(InteractionResult::NoOp.into())
        }
        SessionEvent::ModeratorAction { action } => {
            handle_moderator_action(action, state).await?;
            Ok(InteractionResult::NoOp.into())
        }
    }
}

async fn handle_moderator_action(
    action: ModeratorAction,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or_else(|| "No active session".to_string())?;

    match action {
        ModeratorAction::SetConversationMode { mode } => {
            manager.set_conversation_mode(mode).await;
        }
        ModeratorAction::AppendSystemMessage {
            content,
            message_type,
            severity,
        } => {
            manager
                .add_system_conversation_message(content, message_type, severity)
                .await;
        }
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
pub async fn get_active_session(state: State<'_, AppState>) -> Result<Option<Session>, String> {
    if let Some(manager) = state.session_manager.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = manager
            .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
            .await;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

/// Executes a message content as a task using TaskExecutor
#[tauri::command]
pub async fn execute_message_as_task(
    message_content: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let app_mode = state.app_mode.lock().await.clone();
    let session = manager
        .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;
    let session_id = session.id.clone();
    let workspace_id = &session.workspace_id;

    // Get workspace root path from workspace_id
    let workspace_root = if workspace_id != PLACEHOLDER_WORKSPACE_ID {
        match state.workspace_manager.get_workspace(workspace_id).await {
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

    state
        .task_executor
        .execute_from_message(session_id, message_content, workspace_root)
        .await
        .map_err(|e| e.to_string())
}

/// Adds a participant to the active session
#[tauri::command]
pub async fn add_participant(persona_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .add_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Removes a participant from the active session
#[tauri::command]
pub async fn remove_participant(
    persona_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .remove_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the list of active participants in the current session
#[tauri::command]
pub async fn get_active_participants(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.get_active_participants().await
}

/// Sets the execution strategy for the active session
#[tauri::command]
pub async fn set_execution_strategy(
    strategy: ExecutionModel,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.set_execution_strategy(strategy).await;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current execution strategy for the active session
#[tauri::command]
pub async fn get_execution_strategy(state: State<'_, AppState>) -> Result<ExecutionModel, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    Ok(manager.get_execution_strategy().await)
}

/// Sets the conversation mode for the active session
#[tauri::command]
pub async fn set_conversation_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let conversation_mode = match mode.as_str() {
        "normal" => ConversationMode::Normal,
        "concise" => ConversationMode::Concise,
        "brief" => ConversationMode::Brief,
        "discussion" => ConversationMode::Discussion,
        _ => return Err(format!("Unknown conversation mode: {}", mode)),
    };

    manager.set_conversation_mode(conversation_mode).await;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current conversation mode for the active session
#[tauri::command]
pub async fn get_conversation_mode(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let mode = manager.get_conversation_mode().await;
    let mode_str = match mode {
        ConversationMode::Normal => "normal",
        ConversationMode::Concise => "concise",
        ConversationMode::Brief => "brief",
        ConversationMode::Discussion => "discussion",
    };

    Ok(mode_str.to_string())
}

/// Sets the talk style for the active session
#[tauri::command]
pub async fn set_talk_style(
    style: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

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

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current talk style for the active session
#[tauri::command]
pub async fn get_talk_style(state: State<'_, AppState>) -> Result<Option<String>, String> {
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

/// Handles user input
#[tauri::command]
pub async fn handle_input(
    input: String,
    file_paths: Option<Vec<String>>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    let current_mode = state.app_mode.lock().await.clone();

    let processed_input = if input.trim().starts_with('/') {
        let trimmed = input.trim();
        let cmd_end = trimmed.find(' ').unwrap_or(trimmed.len());
        let cmd_name = &trimmed[1..cmd_end];
        let args = if cmd_end < trimmed.len() {
            trimmed[cmd_end..].trim()
        } else {
            ""
        };

        eprintln!(
            "[SLASH_COMMAND] Detected command: '{}', args: '{}'",
            cmd_name, args
        );

        if let Ok(all_commands) = state.slash_command_repository.list_commands().await {
            eprintln!(
                "[SLASH_COMMAND] Available commands: {:?}",
                all_commands.iter().map(|c| &c.name).collect::<Vec<_>>()
            );
        }

        eprintln!(
            "[SLASH_COMMAND] Getting command '{}' from repository...",
            cmd_name
        );
        match state.slash_command_repository.get_command(cmd_name).await {
            Ok(Some(cmd)) => {
                use orcs_core::slash_command::CommandType;

                match cmd.command_type {
                    CommandType::Prompt => {
                        if cmd.content.contains("{args}") {
                            cmd.content.replace("{args}", args)
                        } else if !args.is_empty() {
                            format!("{}\n\n{}", cmd.content, args)
                        } else {
                            cmd.content.clone()
                        }
                    }
                    CommandType::Shell => {
                        let cmd_to_run = if cmd.content.contains("{args}") {
                            cmd.content.replace("{args}", args)
                        } else {
                            cmd.content.clone()
                        };

                        let working_dir = cmd.working_dir.as_deref();

                        match execute_shell_command(&cmd_to_run, working_dir).await {
                            Ok(output) => format!("Command output:\n```\n{}\n```", output),
                            Err(e) => format!("Error executing command: {}", e),
                        }
                    }
                }
            }
            Ok(None) => format!(
                "Unknown command: /{}\n\nAvailable commands can be viewed in Settings.",
                cmd_name
            ),
            Err(e) => format!("Error loading command: {}", e),
        }
    } else {
        input.clone()
    };

    let app_clone = app.clone();
    let result = manager
        .handle_input_with_streaming(&current_mode, &processed_input, file_paths, move |turn| {
            use orcs_interaction::{StreamingDialogueTurn, StreamingDialogueTurnKind};

            let now = SystemTime::now()
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

            // Convert DialogueMessage to StreamingDialogueTurn for frontend
            let streaming_turn = StreamingDialogueTurn {
                session_id: turn.session_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                kind: StreamingDialogueTurnKind::Chunk {
                    author: turn.author.clone(),
                    content: turn.content.clone(),
                },
            };

            if let Err(e) = app_clone.emit("dialogue-turn", streaming_turn) {
                eprintln!("[TAURI] Failed to emit dialogue-turn event: {}", e);
            }
        })
        .await;

    if let InteractionResult::ModeChanged(ref new_mode) = result {
        *state.app_mode.lock().await = new_mode.clone();
    }

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(result.into())
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

// ============================================================================
// AutoChat Commands
// ============================================================================

/// Gets the AutoChat configuration for a session.
#[tauri::command]
pub async fn get_auto_chat_config(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<AutoChatConfig>, String> {
    // Get the current active session's manager
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or_else(|| "No active session".to_string())?;

    // Verify session ID matches
    if manager.session_id() != session_id {
        return Err(format!(
            "Session ID mismatch: requested {}, active is {}",
            session_id,
            manager.session_id()
        ));
    }

    Ok(manager.get_auto_chat_config().await)
}

/// Updates the AutoChat configuration for a session.
#[tauri::command]
pub async fn update_auto_chat_config(
    session_id: String,
    config: Option<AutoChatConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Get the current active session's manager
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or_else(|| "No active session".to_string())?;

    // Verify session ID matches
    if manager.session_id() != session_id {
        return Err(format!(
            "Session ID mismatch: requested {}, active is {}",
            session_id,
            manager.session_id()
        ));
    }

    manager.set_auto_chat_config(config).await;

    // Persist the updated session
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current AutoChat iteration status.
#[tauri::command]
pub async fn get_auto_chat_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<u32>, String> {
    // Get the current active session's manager
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or_else(|| "No active session".to_string())?;

    // Verify session ID matches
    if manager.session_id() != session_id {
        return Err(format!(
            "Session ID mismatch: requested {}, active is {}",
            session_id,
            manager.session_id()
        ));
    }

    Ok(manager.get_auto_chat_iteration().await)
}

/// Starts AutoChat mode with the given initial input.
///
/// This will execute multiple dialogue iterations automatically based on the
/// session's AutoChat configuration.
#[tauri::command]
pub async fn start_auto_chat(
    input: String,
    file_paths: Option<Vec<String>>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    tracing::info!(
        "[AutoChat] Starting with input: {}",
        input.chars().take(50).collect::<String>()
    );

    // Get config for progress tracking
    let config = manager.get_auto_chat_config().await;
    let max_iterations = config.as_ref().map(|c| c.max_iterations).unwrap_or(5);
    let session_id = manager.session_id().to_string();

    let app_clone = app.clone();
    let app_clone2 = app.clone();
    let session_id_clone = session_id.clone();

    let result = manager
        .execute_auto_chat(&input, file_paths, move |turn| {
            use orcs_interaction::{StreamingDialogueTurn, StreamingDialogueTurnKind};

            // Convert DialogueMessage to StreamingDialogueTurn for frontend
            let streaming_turn = StreamingDialogueTurn {
                session_id: turn.session_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                kind: StreamingDialogueTurnKind::Chunk {
                    author: turn.author.clone(),
                    content: turn.content.clone(),
                },
            };

            if let Err(e) = app_clone.emit("dialogue-turn", streaming_turn) {
                eprintln!("[TAURI] Failed to emit dialogue-turn event: {}", e);
            }
        })
        .await;

    // Emit AutoChat completion event
    use orcs_interaction::{StreamingDialogueTurn, StreamingDialogueTurnKind};
    let completion_event = StreamingDialogueTurn {
        session_id: session_id_clone.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        kind: StreamingDialogueTurnKind::AutoChatComplete {
            total_iterations: max_iterations,
        },
    };

    if let Err(e) = app_clone2.emit("dialogue-turn", completion_event) {
        eprintln!("[TAURI] Failed to emit AutoChat completion event: {}", e);
    }

    // Save the session after AutoChat completes
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(result.into())
}
