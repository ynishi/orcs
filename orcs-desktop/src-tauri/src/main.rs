// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use orcs_core::session::{AppMode, Session, SessionManager};
use orcs_core::persona::{Persona, get_default_presets};
use orcs_core::repository::PersonaRepository;
use orcs_core::user::UserService;
use orcs_core::workspace::{Workspace, UploadedFile};
use orcs_infrastructure::repository::{TomlPersonaRepository, TomlSessionRepository};
use orcs_infrastructure::user_service::ConfigBasedUserService;
use orcs_infrastructure::toml_storage;
use orcs_infrastructure::workspace_manager::FileSystemWorkspaceManager;
use orcs_interaction::{InteractionManager, InteractionResult};
use serde::Serialize;
use tauri::State;

/// Application state shared across Tauri commands
struct AppState {
    session_manager: Arc<SessionManager<InteractionManager>>,
    app_mode: Mutex<AppMode>,
    persona_repository: Arc<dyn PersonaRepository>,
    user_service: Arc<dyn UserService>,
    workspace_manager: Arc<FileSystemWorkspaceManager>,
}

/// Serializable version of DialogueMessage for Tauri IPC
#[derive(Serialize, Clone)]
pub struct SerializableDialogueMessage {
    pub author: String,
    pub content: String,
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
            InteractionResult::NewMessage(msg) => {
                SerializableInteractionResult::NewMessage(msg)
            }
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
async fn create_session(
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let manager = state.session_manager
        .create_session(session_id.clone(), |sid| InteractionManager::new_session(sid, state.persona_repository.clone(), state.user_service.clone()))
        .await
        .map_err(|e| e.to_string())?;

    // Reset app mode
    *state.app_mode.lock().await = AppMode::Idle;

    // Get the full session data to return
    let session = manager.to_session(AppMode::Idle).await;

    Ok(session)
}

/// Lists all saved sessions
#[tauri::command]
async fn list_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<Session>, String> {
    state.session_manager
        .list_sessions()
        .await
        .map_err(|e| e.to_string())
}

/// Switches to a different session
#[tauri::command]
async fn switch_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let manager = state.session_manager
        .switch_session(&session_id, |data| InteractionManager::from_session(data, state.persona_repository.clone(), state.user_service.clone()))
        .await
        .map_err(|e| e.to_string())?;

    let session = manager.to_session(AppMode::Idle).await;

    // Update app mode
    *state.app_mode.lock().await = session.app_mode.clone();

    Ok(session)
}

/// Deletes a session
#[tauri::command]
async fn delete_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.session_manager
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
    state.session_manager
        .rename_session(&session_id, new_title)
        .await
        .map_err(|e| e.to_string())
}

/// Saves the current session
#[tauri::command]
async fn save_current_session(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_mode = state.app_mode.lock().await.clone();
    state.session_manager
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())
}

/// Gets the currently active session
#[tauri::command]
async fn get_active_session(
    state: State<'_, AppState>,
) -> Result<Option<Session>, String> {
    if let Some(manager) = state.session_manager.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = manager.to_session(app_mode).await;
        Ok(Some(session))
    } else {
        Ok(None)
    }
}

/// Gets all personas from the single source of truth
#[tauri::command]
async fn get_personas(
    state: State<'_, AppState>,
) -> Result<Vec<Persona>, String> {
    state.persona_repository.get_all()
}

/// Saves persona configurations
#[tauri::command]
async fn save_persona_configs(
    configs: Vec<Persona>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.persona_repository.save_all(&configs)
}

/// Gets the user's nickname from the config
#[tauri::command]
async fn get_user_nickname(
    state: State<'_, AppState>,
) -> Result<String, String> {
    Ok(state.user_service.get_user_name())
}

/// Adds a participant to the active session
#[tauri::command]
async fn add_participant(
    persona_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.add_participant(&persona_id).await
        .map_err(|e| e.to_string())?;

    // Auto-save after adding participant
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Removes a participant from the active session
#[tauri::command]
async fn remove_participant(
    persona_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.remove_participant(&persona_id).await
        .map_err(|e| e.to_string())?;

    // Auto-save after removing participant
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(())
}

/// Gets the list of active participants in the current session
#[tauri::command]
async fn get_active_participants(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager.get_active_participants().await
}

/// Sets the execution strategy for the active session
#[tauri::command]
async fn set_execution_strategy(
    strategy: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.session_manager
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
async fn get_execution_strategy(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    Ok(manager.get_execution_strategy().await)
}

/// Gets the path to the configuration file, creating it if it doesn't exist
#[tauri::command]
async fn get_config_path() -> Result<String, String> {
    // Get the base config directory
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?;

    // Construct the path to the orcs subdirectory
    let orcs_config_dir = config_dir.join("orcs");

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&orcs_config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Construct the final path to config.toml
    let config_file = orcs_config_dir.join("config.toml");

    // Create the file if it doesn't exist
    if !config_file.exists() {
        std::fs::File::create(&config_file)
            .map_err(|e| format!("Failed to create config file: {}", e))?;
    }

    // Convert PathBuf to String
    let path_str = config_file.to_str()
        .ok_or("Config path is not valid UTF-8")?;

    Ok(path_str.to_string())
}

/// Handles user input
#[tauri::command]
async fn handle_input(
    input: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    // Get the current mode
    let current_mode = state.app_mode.lock().await.clone();

    // Handle the input with streaming support
    let app_clone = app.clone();
    let result = manager.handle_input_with_streaming(&current_mode, &input, move |turn| {
        use tauri::Emitter;

        // Log each dialogue turn as it becomes available with timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let preview: String = turn.content.chars().take(50).collect();
        eprintln!("[TAURI] [{}.{:03}] Streaming turn: {} - {}...",
            now.as_secs(),
            now.subsec_millis(),
            turn.author,
            preview);

        // Emit event to frontend for real-time streaming
        if let Err(e) = app_clone.emit("dialogue-turn", turn) {
            eprintln!("[TAURI] Failed to emit dialogue-turn event: {}", e);
        }
    }).await;

    // Update the mode if it changed
    if let InteractionResult::ModeChanged(ref new_mode) = result {
        *state.app_mode.lock().await = new_mode.clone();
    }

    // Auto-save after each interaction
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(result.into())
}

/// Gets the current workspace for the application's working directory
#[tauri::command]
async fn get_current_workspace(
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    // Get the current working directory
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Get or create workspace for the current directory
    state.workspace_manager
        .get_or_create_workspace(&current_dir)
        .await
        .map_err(|e| e.to_string())
}

/// Lists all files in a workspace
#[tauri::command]
async fn list_workspace_files(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<UploadedFile>, String> {
    use orcs_core::workspace::manager::WorkspaceManager;

    // Get the workspace
    let workspace = state.workspace_manager
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    // Return uploaded files or empty vector if workspace not found
    Ok(workspace
        .map(|w| w.resources.uploaded_files)
        .unwrap_or_default())
}

/// Gets Git repository information for the current directory
#[tauri::command]
fn get_git_info() -> GitInfo {
    use std::process::Command;

    // Check if we're in a git repository
    let is_repo = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !is_repo {
        return GitInfo {
            is_repo: false,
            branch: None,
            repo_name: None,
        };
    }

    // Get current branch
    let branch = Command::new("git")
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
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .and_then(|url| {
                        // Extract repo name from URL
                        // e.g., "git@github.com:user/repo.git" -> "repo"
                        // e.g., "https://github.com/user/repo.git" -> "repo"
                        url.trim()
                            .split('/')
                            .last()
                            .map(|name| {
                                name.trim_end_matches(".git").to_string()
                            })
                    })
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback: use the root directory name if no remote origin
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .ok()
                            .and_then(|path| {
                                std::path::Path::new(path.trim())
                                    .file_name()
                                    .and_then(|name| name.to_str())
                                    .map(|s| s.to_string())
                            })
                    } else {
                        None
                    }
                })
        });

    GitInfo {
        is_repo: true,
        branch,
        repo_name,
    }
}

fn main() {
    tauri::async_runtime::block_on(async {
        // Composition Root: Create the concrete repository instances
        let persona_repository = Arc::new(TomlPersonaRepository);
        let user_service: Arc<dyn UserService> = Arc::new(ConfigBasedUserService::new());

        // Initialize FileSystemWorkspaceManager
        let workspace_root = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".orcs")
            .join("workspaces");
        let workspace_manager = Arc::new(
            FileSystemWorkspaceManager::new(workspace_root)
                .await
                .expect("Failed to initialize workspace manager")
        );

        // Seed the config file with default personas if it's empty on first run.
        if let Ok(configs) = persona_repository.get_all() {
            if configs.is_empty() {
                let default_presets = get_default_presets();
                if let Err(e) = persona_repository.save_all(&default_presets) {
                    // This is a critical failure on startup, so we panic.
                    panic!("Failed to seed default personas into config file: {}", e);
                }
            } else {
                // Auto-migrate: re-save to ensure V2 format
                // This converts any V1 configs to V2 on startup
                if let Err(e) = persona_repository.save_all(&configs) {
                    eprintln!("Warning: Failed to auto-migrate persona config to V2: {}", e);
                }
            }
        }

        // Ensure user profile is initialized with default if it doesn't exist
        if let Err(e) = toml_storage::ensure_user_profile_initialized() {
            eprintln!("Warning: Failed to initialize user profile: {}", e);
        }

        // Create TomlSessionRepository at default location
        let session_repository = Arc::new(
            TomlSessionRepository::default_location(persona_repository.clone())
                .expect("Failed to create session repository")
        );

        // Initialize SessionManager with the repository
        let session_manager: Arc<SessionManager<InteractionManager>> = Arc::new(
            SessionManager::new(session_repository)
        );

        // Try to restore last session, otherwise create a new one
        let restored = session_manager
            .restore_last_session(|data| InteractionManager::from_session(data, persona_repository.clone(), user_service.clone()))
            .await
            .ok()
            .flatten();

        if restored.is_none() {
            // Create initial session
            let initial_session_id = uuid::Uuid::new_v4().to_string();
            session_manager
                .create_session(initial_session_id, |sid| InteractionManager::new_session(sid, persona_repository.clone(), user_service.clone()))
                .await
                .expect("Failed to create initial session");
        }

        let app_mode = Mutex::new(AppMode::Idle);

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .manage(AppState {
                session_manager,
                app_mode,
                persona_repository,
                user_service,
                workspace_manager,
            })
            .invoke_handler(tauri::generate_handler![
                create_session,
                list_sessions,
                switch_session,
                delete_session,
                rename_session,
                save_current_session,
                get_active_session,
                get_personas,
                save_persona_configs,
                get_user_nickname,
                add_participant,
                remove_participant,
                get_active_participants,
                set_execution_strategy,
                get_execution_strategy,
                get_config_path,
                get_git_info,
                get_current_workspace,
                list_workspace_files,
                handle_input,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
