use std::time::SystemTime;

use llm_toolkit::ToPrompt;
use llm_toolkit::agent::dialogue::{ExecutionModel, TalkStyle};
use orcs_core::schema::{ExecutionModelType, TalkStyleType};
use orcs_core::session::{
    AppMode, AutoChatConfig, ConversationMode, ErrorSeverity, ModeratorAction,
    SessionRepository, PLACEHOLDER_WORKSPACE_ID, Session, SessionEvent,
};
use orcs_core::slash_command::{CommandType, SlashCommand};
use orcs_core::workspace::manager::WorkspaceStorageService;
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

/// Agent configuration for runtime backend/model selection
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Backend type (gemini_api, claude_api, open_ai_api, etc.)
    pub backend: String,
    /// Model name (optional, uses default if not specified)
    pub model_name: Option<String>,
    /// Gemini-specific options
    pub gemini_options: Option<GeminiOptions>,
}

/// Gemini-specific options
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiOptions {
    /// Thinking level: LOW, MEDIUM, HIGH
    pub thinking_level: Option<String>,
    /// Enable Google Search integration
    pub google_search: Option<bool>,
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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session = state
        .session_usecase
        .create_session(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    *state.app_mode.lock().await = AppMode::Idle;

    // Emit app-state:update event for SSOT synchronization
    use orcs_core::state::repository::StateRepository;
    if let Ok(app_state) = state.app_state_service.get_state().await {
        let _ = app.emit("app-state:update", &app_state);
    }

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
    use orcs_core::session::SessionRepository;
    let sessions = state
        .session_repository
        .list_all()
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
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session = state
        .session_usecase
        .switch_session(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    *state.app_mode.lock().await = session.app_mode.clone();

    // Emit app-state:update event for SSOT synchronization
    use orcs_core::state::repository::StateRepository;
    if let Ok(app_state) = state.app_state_service.get_state().await {
        let _ = app.emit("app-state:update", &app_state);
    }

    Ok(session)
}

/// Deletes a session
#[tauri::command]
pub async fn delete_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .session_usecase
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
        .session_usecase
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
        .session_usecase
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
        .session_usecase
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
                .session_usecase
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

/// Custom command information (Task/Prompt/Shell)
#[derive(Debug, Clone, Serialize, ToPrompt)]
#[prompt(template = r#"- `/{{ name }}`{% if args %} {{ args }}{% endif %}: {{ description }}"#)]
struct CustomCommandInfo {
    name: String,
    description: String,
    args: Option<String>,
}

impl From<&SlashCommand> for CustomCommandInfo {
    fn from(cmd: &SlashCommand) -> Self {
        Self {
            name: cmd.name.clone(),
            description: cmd.description.clone(),
            args: cmd.args_description.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToPrompt)]
#[prompt(template = r#"# Slash Commands - Execution Guide

You are authorized to execute slash commands to accomplish user tasks. When you identify that a slash command would help complete the task, **you MUST execute it immediately**.

## ORCS Workspace Architecture

**Important**: ORCS operates with a clear separation between project files and workspace storage:

- **Project Files**: Source code and project assets located at the workspace root directory
  - ORCS runs directly in your project directory (e.g., `/path/to/your-project`)
  - Use `/search` to find and read project source files
  - Use shell commands to interact with project files (git, build tools, etc.)

- **Workspace Storage**: Completely separate data storage for ORCS-specific files
  - Located in ORCS data directory (NOT in your project)
  - Stores: conversation history, saved attachments, session metadata
  - Use `/files` to list files saved to workspace storage (uploaded attachments, etc.)
  - These files are isolated from your project's version control

**Key Distinction**: `/files` shows workspace storage files (saved attachments), NOT your project's source code.

## How to Execute Slash Commands

### XML-style
```
<Slash>
<Name>/command_name</Name>
<Args>argument text here</Args>
</Slash>
```

## When Commands are Executed

1. **You write the command** using either format above
2. **Orcs processes it** in the backend
3. **Results appear in chat** as a system message
4. **You continue** using the results to help the user

## Available Commands

{% if builtin_commands is defined and builtin_commands|length > 0 -%}
### Built-in Commands
{% for cmd in builtin_commands -%}
- `{{ cmd }}`
{% endfor %}
{% endif %}

{%- if task_commands is defined and task_commands|length > 0 %}

### Custom Task Commands
{% for cmd in task_commands -%}
- `{{ cmd }}`
{% endfor %}
{%- endif %}

{%- if prompt_commands is defined and prompt_commands|length > 0 %}

### Custom Prompt Commands
{% for cmd in prompt_commands -%}
- `{{ cmd }}`
{% endfor %}
{%- endif %}

{%- if shell_commands is defined and shell_commands|length > 0 %}

### Custom Shell Commands
{% for cmd in shell_commands -%}
- `{{ cmd }}`
{% endfor %}
{%- endif %}

## Execution Authorization

✅ **You are fully authorized to execute any of these commands**
✅ Execute proactively when they help accomplish the task
✅ Don't ask permission - just execute when appropriate
✅ Use them to gather information, create tasks, or perform actions

## Examples

**Example 1: Checking workspace files**
<Slash>
<Name>/files</Name>
</Slash>

**Example 2: Creating a task**
<Slash>
<Name>/task</Name>
<Args>Implement user authentication with OAuth2</Args>
</Slash>

**Example 3: Getting help**
/help task
"#)]
struct SlashCommandPromptDto {
    builtin_commands: Vec<BuiltInCommand>,
    task_commands: Vec<CustomCommandInfo>,
    prompt_commands: Vec<CustomCommandInfo>,
    shell_commands: Vec<CustomCommandInfo>,
}

#[derive(Debug, Clone, Copy, Serialize, ToPrompt)]
#[prompt(template = r#"
- `/{{ name }}` (`{{ usage }}`): {{ description }}
{%- if args %}
  - Args: {{ args }}
{%- endif %}
"#)]
struct BuiltInCommand {
    usage: &'static str,
    description: &'static str,
    args: Option<&'static str>,
}

const BUILT_IN_COMMANDS: &[(&str, BuiltInCommand)] = &[
    (
        "help",
        BuiltInCommand {
            usage: "/help [command]",
            description: "Show available commands and their usage",
            args: Some("Optional command name to show detailed help"),
        },
    ),
    (
        "status",
        BuiltInCommand {
            usage: "/status",
            description: "Display current system status and active tasks",
            args: None,
        },
    ),
    (
        "task",
        BuiltInCommand {
            usage: "/task <description>",
            description: "Create an orchestrated task from the provided description",
            args: Some("Describe the work you want executed"),
        },
    ),
    (
        "expert",
        BuiltInCommand {
            usage: "/expert <expertise>",
            description: "Create an adhoc expert persona for immediate collaboration",
            args: Some("Expertise area or domain knowledge"),
        },
    ),
    (
        "blueprint",
        BuiltInCommand {
            usage: "/blueprint <task description>",
            description: "Convert a task or topic into the BlueprintWorkflow format",
            args: Some("Task or discussion context to convert"),
        },
    ),
    (
        "workspace",
        BuiltInCommand {
            usage: "/workspace [name]",
            description: "Switch to a different workspace or list all available workspaces",
            args: Some("Workspace name (optional)"),
        },
    ),
    (
        "files",
        BuiltInCommand {
            usage: "/files",
            description: "List files saved to workspace storage (not project source files)",
            args: None,
        },
    ),
    (
        "search",
        BuiltInCommand {
            usage: "/search <query> [scope:workspace|local|global]",
            description: "Search workspace or local files for the provided query",
            args: Some(
                "Provide a query to search. Optionally set scope:workspace|local|global to control coverage",
            ),
        },
    ),
    (
        "mode",
        BuiltInCommand {
            usage: "/mode [normal|concise|brief|discussion]",
            description: "Change conversation mode to control agent verbosity",
            args: Some("normal / concise / brief / discussion"),
        },
    ),
    (
        "talk",
        BuiltInCommand {
            usage: "/talk [brainstorm|casual|decision_making|debate|problem_solving|review|planning|none]",
            description: "Set dialogue style for multi-agent collaboration",
            args: Some(
                "brainstorm / casual / decision_making / debate / problem_solving / review / planning / none",
            ),
        },
    ),
    (
        "create-persona",
        BuiltInCommand {
            usage: "/create-persona <json>",
            description: "Create a new persona from JSON definition (UUID auto-generated)",
            args: Some(
                r#"JSON with required fields: name, role, background (min 10 chars), communication_style (min 10 chars), backend (claude_cli/claude_api/gemini_cli/gemini_api/open_ai_api/codex_cli). Optional: model_name, default_participant (bool), icon, base_color. NOTE: ID is always auto-generated as UUID (not accepted in request)"#,
            ),
        },
    ),
    (
        "create-slash-command",
        BuiltInCommand {
            usage: "/create-slash-command <json>",
            description: "Create a new slash command (not yet implemented)",
            args: Some("JSON slash command definition"),
        },
    ),
    (
        "create-workspace",
        BuiltInCommand {
            usage: "/create-workspace <json>",
            description: "Create a new workspace (not yet implemented)",
            args: Some("JSON workspace definition"),
        },
    ),
];

fn build_slash_command_prompt(commands: &[SlashCommand]) -> Option<String> {
    if commands.is_empty() && BUILT_IN_COMMANDS.is_empty() {
        return None;
    }

    // Convert built-in commands using From impl
    let builtin_commands: Vec<BuiltInCommand> = BUILT_IN_COMMANDS.iter().map(|cmd| cmd.1).collect();

    // Group custom commands by type using From impl
    let task_commands: Vec<CustomCommandInfo> = commands
        .iter()
        .filter(|c| c.command_type == CommandType::Task)
        .map(|cmd| cmd.into())
        .collect();

    let prompt_commands: Vec<CustomCommandInfo> = commands
        .iter()
        .filter(|c| c.command_type == CommandType::Prompt)
        .map(|cmd| cmd.into())
        .collect();

    let shell_commands: Vec<CustomCommandInfo> = commands
        .iter()
        .filter(|c| c.command_type == CommandType::Shell)
        .map(|cmd| cmd.into())
        .collect();

    // Build DTO and render template
    let dto = SlashCommandPromptDto {
        builtin_commands,
        task_commands,
        prompt_commands,
        shell_commands,
    };

    Some(dto.to_prompt())
}

async fn handle_moderator_action(
    action: ModeratorAction,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_usecase
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
        .session_usecase
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())
}

/// Gets the currently active session
#[tauri::command]
pub async fn get_active_session(state: State<'_, AppState>) -> Result<Option<Session>, String> {
    if let Some(manager) = state.session_usecase.active_session().await {
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
        .session_usecase
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

    // Build thread context from session's Summary and recent messages
    let thread_context = build_thread_context_for_task(&session);

    state
        .task_executor
        .execute_from_message_with_context(session_id, message_content, workspace_root, thread_context)
        .await
        .map_err(|e| e.to_string())
}

/// Builds thread context for task execution from session data.
///
/// Extracts Summary messages and recent conversation messages to provide
/// context for the task executor. This helps AI agents understand the
/// conversation context when executing tasks created via /task command.
fn build_thread_context_for_task(session: &Session) -> Option<String> {
    let mut context_parts: Vec<String> = Vec::new();

    // 1. Extract Summary from system_messages (most recent one)
    let summary = session
        .system_messages
        .iter()
        .filter(|msg| {
            msg.metadata
                .system_message_type
                .as_ref()
                .is_some_and(|t| t == "Summary")
        })
        .last()
        .map(|msg| msg.content.clone());

    if let Some(summary_content) = summary {
        context_parts.push(format!("### Thread Summary\n{}", summary_content));
    }

    // 2. Get recent conversation messages (last 10 user/assistant messages)
    // persona_histories is HashMap<persona_id, Vec<ConversationMessage>>
    let recent_messages: Vec<String> = session
        .persona_histories
        .iter()
        .flat_map(|(persona_id, history)| {
            history.iter().rev().take(5).map(move |msg| {
                let role_str = match msg.role {
                    orcs_core::session::MessageRole::User => "User",
                    orcs_core::session::MessageRole::Assistant => persona_id.as_str(),
                    orcs_core::session::MessageRole::System => "System",
                };
                format!("**{}**: {}", role_str, msg.content.chars().take(500).collect::<String>())
            })
        })
        .take(10)
        .collect();

    if !recent_messages.is_empty() {
        context_parts.push(format!(
            "### Recent Conversation\n{}",
            recent_messages.join("\n\n")
        ));
    }

    if context_parts.is_empty() {
        None
    } else {
        Some(context_parts.join("\n\n"))
    }
}

/// Adds a participant to the active session
#[tauri::command]
pub async fn add_participant(persona_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .add_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Removes a participant from the active session
#[tauri::command]
pub async fn remove_participant(
    persona_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .remove_participant(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the list of active participants in the current session
#[tauri::command]
pub async fn get_active_participants(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.get_active_participants().await
}

/// Toggles mute status for the active session and returns the new value
#[tauri::command]
pub async fn toggle_mute(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let is_muted = manager.toggle_mute().await;

    // Save session
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(is_muted)
}

/// Gets the mute status for the active session
#[tauri::command]
pub async fn get_mute_status(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    Ok(manager.is_muted().await)
}

/// Gets the context mode for the active session
#[tauri::command]
pub async fn get_context_mode(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let mode = manager.get_context_mode().await;
    Ok(match mode {
        orcs_core::session::ContextMode::Rich => "rich".to_string(),
        orcs_core::session::ContextMode::Clean => "clean".to_string(),
    })
}

/// Sets the context mode for the active session
#[tauri::command]
pub async fn set_context_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let context_mode = match mode.as_str() {
        "rich" => orcs_core::session::ContextMode::Rich,
        "clean" => orcs_core::session::ContextMode::Clean,
        _ => return Err(format!("Invalid context mode: {}", mode)),
    };

    manager.set_context_mode(context_mode).await;

    // Save session
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Sets the execution strategy for the active session
#[tauri::command]
pub async fn set_execution_strategy(
    strategy: ExecutionModelType,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Convert ExecutionModelType (Anti-Corruption Layer) to ExecutionModel (llm-toolkit)
    let execution_model: ExecutionModel = strategy.into();
    manager.set_execution_strategy(execution_model).await;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current execution strategy for the active session
#[tauri::command]
pub async fn get_execution_strategy(
    state: State<'_, AppState>,
) -> Result<ExecutionModelType, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Convert ExecutionModel (llm-toolkit) to ExecutionModelType (Anti-Corruption Layer)
    let execution_model = manager.get_execution_strategy().await;
    Ok(execution_model.into())
}

/// Sets the conversation mode for the active session
#[tauri::command]
pub async fn set_conversation_mode(mode: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state
        .session_usecase
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
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current conversation mode for the active session
#[tauri::command]
pub async fn get_conversation_mode(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let mode = manager.get_conversation_mode().await;
    let mode_str = match mode {
        ConversationMode::Detailed => "detailed",
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
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Parse TalkStyle using schema-bridge v0.2 string_conversion
    let talk_style = style
        .map(|s| {
            use std::str::FromStr;
            TalkStyleType::from_str(&s)
                .map(|style_type| TalkStyle::from(style_type))
                .map_err(|_e| format!("Unknown talk style: {}", s))
        })
        .transpose()?;

    manager.set_talk_style(talk_style).await;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current talk style for the active session
#[tauri::command]
pub async fn get_talk_style(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let style = manager.get_talk_style().await;
    // Serialize TalkStyle using schema-bridge v0.2 string_conversion
    let style_str = style.map(|s| TalkStyleType::from(s).to_string());

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
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Only inject SlashCommand prompt extension in Rich mode
    let context_mode = manager.get_context_mode().await;
    if matches!(context_mode, orcs_core::session::ContextMode::Rich) {
        let slash_commands = state
            .slash_command_repository
            .list_commands()
            .await
            .unwrap_or_else(|e| {
                eprintln!("[handle_input] Failed to list commands: {}", e);
                Vec::new()
            });
        let prompt_extension = build_slash_command_prompt(&slash_commands);
        manager.set_prompt_extension(prompt_extension).await;
    } else {
        // Clean mode: no prompt extension
        manager.set_prompt_extension(None).await;
    }

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

        // Check for built-in entity commands first (critical commands that should always work)
        match cmd_name {
            "create-persona" => match execute_create_persona(args, &state).await {
                Ok(persona) => format!(
                    "✅ Successfully created persona '{}'\n\nID: {}\nRole: {}\nBackend: {:?}\n\nThe persona is now available in the Personas panel.",
                    persona.name, persona.id, persona.role, persona.backend
                ),
                Err(e) => format!("❌ Failed to create persona: {}", e),
            },
            "create-slash-command" => {
                format!(
                    "❌ /create-slash-command is not yet implemented.\n\nPlease create slash commands manually in ~/.orcs/slash_commands/ for now."
                )
            }
            "create-workspace" => {
                format!(
                    "❌ /create-workspace is not yet implemented.\n\nPlease use the workspace management UI for now."
                )
            }
            // For all other commands, check the repository
            _ => {
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
                            CommandType::Task => {
                                // Task commands should be handled separately via execute_task_command
                                format!(
                                    "Task command '{}' requires async execution. Use the task execution UI or API instead.",
                                    cmd_name
                                )
                            }
                        }
                    }
                    Ok(None) => format!(
                        "Unknown command: /{}\n\nAvailable commands can be viewed in Settings.",
                        cmd_name
                    ),
                    Err(e) => format!("Error loading command: {}", e),
                }
            }
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
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(result.into())
}

/// Helper function to create a persona from JSON arguments
async fn execute_create_persona(
    args: &str,
    state: &State<'_, AppState>,
) -> Result<orcs_core::persona::Persona, String> {
    use orcs_core::persona::CreatePersonaRequest;

    // Parse JSON into CreatePersonaRequest
    let request: CreatePersonaRequest =
        serde_json::from_str(args).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Validate request
    request.validate()?;

    // Convert to Persona (UUID auto-generated if needed)
    let persona = request.into_persona();

    // Save to repository
    let mut all_personas = state
        .persona_repository
        .get_all()
        .await
        .map_err(|e| format!("Failed to load personas: {}", e))?;

    // Check for duplicate ID
    if all_personas.iter().any(|p| p.id == persona.id) {
        return Err(format!("Persona with ID '{}' already exists", persona.id));
    }

    all_personas.push(persona.clone());

    state
        .persona_repository
        .save_all(&all_personas)
        .await
        .map_err(|e| format!("Failed to save persona: {}", e))?;

    // Invalidate dialogue cache to reflect new persona
    if let Some(manager) = state.session_usecase.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(persona)
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
        .session_usecase
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
        .session_usecase
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
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the current AutoChat iteration status.
#[tauri::command]
pub async fn get_auto_chat_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<i32>, String> {
    // Get the current active session's manager
    let manager = state
        .session_usecase
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
        .session_usecase
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
        }, Some(state.cancel_flag.clone()))
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
    let _ = state.session_usecase.save_active_session(app_mode).await;

    Ok(result.into())
}

// ============================================================================
// Session Support Commands (Summary, ActionPlan)
// ============================================================================

/// Generates a summary from conversation thread content.
#[tauri::command]
pub async fn generate_summary(
    thread_content: String,
    session_id: String,
    agent_config: Option<AgentConfig>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use orcs_application::SessionSupportAgentService;

    tracing::info!(
        "[SessionSupport] Generating summary for session: {} with config: {:?}",
        session_id,
        agent_config
    );

    let summary = if let Some(config) = agent_config {
        // Use custom agent configuration
        SessionSupportAgentService::generate_summary_with_config(
            &thread_content,
            &config.backend,
            config.model_name.as_deref(),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.thinking_level.as_deref()),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.google_search),
            Some(state.cancel_flag.clone()),
        )
        .await
        .map_err(|e| format!("Failed to generate summary: {}", e))?
    } else {
        // Use default agent
        let service = SessionSupportAgentService::new();
        service
            .generate_summary(&thread_content)
            .await
            .map_err(|e| format!("Failed to generate summary: {}", e))?
    };

    tracing::info!("[SessionSupport] Summary generated successfully");

    Ok(summary)
}

/// Generates an action plan from conversation thread content.
#[tauri::command]
pub async fn generate_action_plan(
    thread_content: String,
    session_id: String,
    agent_config: Option<AgentConfig>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use orcs_application::SessionSupportAgentService;

    tracing::info!(
        "[SessionSupport] Generating action plan for session: {} with config: {:?}",
        session_id,
        agent_config
    );

    let action_plan = if let Some(config) = agent_config {
        // Use custom agent configuration
        SessionSupportAgentService::generate_action_plan_with_config(
            &thread_content,
            &config.backend,
            config.model_name.as_deref(),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.thinking_level.as_deref()),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.google_search),
            Some(state.cancel_flag.clone()),
        )
        .await
        .map_err(|e| format!("Failed to generate action plan: {}", e))?
    } else {
        // Use default agent
        let service = SessionSupportAgentService::new();
        service
            .generate_action_plan(&thread_content)
            .await
            .map_err(|e| format!("Failed to generate action plan: {}", e))?
    };

    tracing::info!("[SessionSupport] Action plan generated successfully");

    Ok(action_plan)
}

/// Generates expertise from conversation thread content.
#[tauri::command]
pub async fn generate_expertise(
    thread_content: String,
    session_id: String,
    agent_config: Option<AgentConfig>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use orcs_application::SessionSupportAgentService;

    tracing::info!(
        "[SessionSupport] Generating expertise for session: {} with config: {:?}",
        session_id,
        agent_config
    );

    let expertise = if let Some(config) = agent_config {
        // Use custom agent configuration
        SessionSupportAgentService::generate_expertise_with_config(
            &thread_content,
            &config.backend,
            config.model_name.as_deref(),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.thinking_level.as_deref()),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.google_search),
            Some(state.cancel_flag.clone()),
        )
        .await
        .map_err(|e| format!("Failed to generate expertise: {}", e))?
    } else {
        // Use default agent
        let service = SessionSupportAgentService::new();
        service
            .generate_expertise(&thread_content)
            .await
            .map_err(|e| format!("Failed to generate expertise: {}", e))?
    };

    tracing::info!("[SessionSupport] Expertise generated successfully");

    Ok(expertise)
}

/// Generates comprehensive Concept/Design Issue from conversation thread content.
#[tauri::command]
pub async fn generate_concept_issue(
    thread_content: String,
    session_id: String,
    agent_config: Option<AgentConfig>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    use orcs_application::SessionSupportAgentService;

    tracing::info!(
        "[SessionSupport] Generating concept/design issue for session: {} with config: {:?}",
        session_id,
        agent_config
    );

    let concept_issue = if let Some(config) = agent_config {
        // Use custom agent configuration
        SessionSupportAgentService::generate_concept_issue_with_config(
            &thread_content,
            &config.backend,
            config.model_name.as_deref(),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.thinking_level.as_deref()),
            config
                .gemini_options
                .as_ref()
                .and_then(|opts| opts.google_search),
            Some(state.cancel_flag.clone()),
        )
        .await
        .map_err(|e| format!("Failed to generate concept/design issue: {}", e))?
    } else {
        // Use default agent
        let service = SessionSupportAgentService::new();
        service
            .generate_concept_issue(&thread_content)
            .await
            .map_err(|e| format!("Failed to generate concept/design issue: {}", e))?
    };

    tracing::info!("[SessionSupport] Concept/Design Issue generated successfully");

    Ok(concept_issue)
}

// ============================================================================
// Operation Cancellation
// ============================================================================

/// Cancels the currently running operation.
#[tauri::command]
pub async fn cancel_current_operation(state: State<'_, AppState>) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    tracing::info!("[Cancel] Cancelling current operation");
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

/// Resets the cancellation flag.
#[tauri::command]
pub async fn reset_cancel_flag(state: State<'_, AppState>) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    tracing::debug!("[Cancel] Resetting cancel flag");
    state.cancel_flag.store(false, Ordering::SeqCst);
    Ok(())
}

// ============================================================================
// Sandbox Mode Commands
// ============================================================================

/// Enters sandbox mode by saving sandbox state to the current session
#[tauri::command]
pub async fn enter_sandbox_mode(
    worktree_path: String,
    original_branch: String,
    sandbox_branch: String,
    sandbox_root: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use orcs_core::session::SandboxState;
    use std::path::PathBuf;

    let session_manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Set sandbox state in InteractionManager
    let sandbox_state = SandboxState {
        worktree_path: worktree_path.clone(),
        original_branch,
        sandbox_branch,
        sandbox_root,
    };
    session_manager.set_sandbox_state(Some(sandbox_state)).await;

    // Change agent workspace root to worktree path (so agents run in sandbox)
    session_manager
        .set_agent_workspace_root(Some(PathBuf::from(&worktree_path)))
        .await;

    // Save session with updated sandbox state
    let app_mode = state.app_mode.lock().await.clone();
    let session = session_manager
        .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;

    state
        .session_repository
        .save(&session)
        .await
        .map_err(|e| format!("Failed to save session: {}", e))?;

    tracing::info!(
        "[Sandbox] Entered sandbox mode for session {}, agent CWD: {}",
        session.id,
        worktree_path
    );
    Ok(())
}

/// Exits sandbox mode by removing sandbox state from the current session
#[tauri::command]
pub async fn exit_sandbox_mode(state: State<'_, AppState>) -> Result<(), String> {
    use orcs_core::workspace::manager::WorkspaceStorageService;

    let session_manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Get current session to retrieve workspace_id
    let app_mode = state.app_mode.lock().await.clone();
    let session = session_manager
        .to_session(app_mode.clone(), PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;
    let workspace_id = &session.workspace_id;

    // Restore original workspace root from workspace_id
    if workspace_id != PLACEHOLDER_WORKSPACE_ID {
        match state.workspace_storage_service.get_workspace(workspace_id).await {
            Ok(Some(workspace)) => {
                session_manager
                    .set_agent_workspace_root(Some(workspace.root_path.clone()))
                    .await;
                tracing::info!(
                    "[Sandbox] Restored agent CWD to: {:?}",
                    workspace.root_path
                );
            }
            Ok(None) => {
                tracing::warn!("[Sandbox] Workspace not found for id: {}", workspace_id);
                // Set to None as fallback
                session_manager.set_agent_workspace_root(None).await;
            }
            Err(e) => {
                tracing::warn!("[Sandbox] Failed to get workspace: {}", e);
                // Set to None as fallback
                session_manager.set_agent_workspace_root(None).await;
            }
        }
    } else {
        // No workspace, set to None
        session_manager.set_agent_workspace_root(None).await;
    }

    // Clear sandbox state in InteractionManager
    session_manager.set_sandbox_state(None).await;

    // Save session with cleared sandbox state
    let session = session_manager
        .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;

    state
        .session_repository
        .save(&session)
        .await
        .map_err(|e| format!("Failed to save session: {}", e))?;

    tracing::info!("[Sandbox] Exited sandbox mode for session {}", session.id);
    Ok(())
}

/// Gets the current sandbox state for the active session
#[tauri::command]
pub async fn get_sandbox_state(
    state: State<'_, AppState>,
) -> Result<Option<orcs_core::session::SandboxState>, String> {
    let session_manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    Ok(session_manager.get_sandbox_state().await)
}
