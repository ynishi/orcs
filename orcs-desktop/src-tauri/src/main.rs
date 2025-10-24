// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use orcs_core::session::Session;
use orcs_core::session_manager::SessionManager;
use orcs_core::session_storage::SessionStorage;
use orcs_core::config::PersonaConfig;
use orcs_core::repository::PersonaRepository;
use orcs_infrastructure::repository::{TomlPersonaRepository, TomlSessionRepository};
use orcs_interaction::{InteractionManager, InteractionResult};
use orcs_interaction::presets::get_default_presets;
use orcs_types::AppMode;
use serde::Serialize;
use tauri::State;

/// Application state shared across Tauri commands
struct AppState {
    session_manager: Arc<SessionManager<InteractionManager>>,
    app_mode: Mutex<AppMode>,
    persona_repository: Arc<dyn PersonaRepository>,
}

/// Serializable version of DialogueMessage for Tauri IPC
#[derive(Serialize, Clone)]
pub struct SerializableDialogueMessage {
    pub author: String,
    pub content: String,
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
        .create_session(session_id.clone(), |sid| InteractionManager::new_session(sid, state.persona_repository.clone()))
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
        .switch_session(&session_id, |data| InteractionManager::from_session(data, state.persona_repository.clone()))
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
) -> Result<Vec<PersonaConfig>, String> {
    state.persona_repository.get_all()
}

/// Saves persona configurations
#[tauri::command]
async fn save_persona_configs(
    configs: Vec<PersonaConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.persona_repository.save_all(&configs)
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
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    let manager = state.session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    // Get the current mode
    let current_mode = state.app_mode.lock().await.clone();

    // Handle the input
    let result = manager.handle_input(&current_mode, &input).await;

    // Update the mode if it changed
    if let InteractionResult::ModeChanged(ref new_mode) = result {
        *state.app_mode.lock().await = new_mode.clone();
    }

    // Auto-save after each interaction
    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(result.into())
}

fn main() {
    tauri::async_runtime::block_on(async {
        // Composition Root: Create the concrete repository instances
        let persona_repository = Arc::new(TomlPersonaRepository);

        // Seed the config file with default personas if it's empty on first run.
        if let Ok(configs) = persona_repository.get_all() {
            if configs.is_empty() {
                let default_presets = get_default_presets();
                if let Err(e) = persona_repository.save_all(&default_presets) {
                    // This is a critical failure on startup, so we panic.
                    panic!("Failed to seed default personas into config file: {}", e);
                }
            }
        }

        // Create SessionStorage and wrap it in TomlSessionRepository
        let storage = SessionStorage::default_location()
            .expect("Failed to create session storage");
        let session_repository = Arc::new(TomlSessionRepository::new(storage));

        // Initialize SessionManager with the repository
        let session_manager: Arc<SessionManager<InteractionManager>> = Arc::new(
            SessionManager::new(session_repository)
        );

        // Try to restore last session, otherwise create a new one
        let restored = session_manager
            .restore_last_session(|data| InteractionManager::from_session(data, persona_repository.clone()))
            .await
            .ok()
            .flatten();

        if restored.is_none() {
            // Create initial session
            let initial_session_id = uuid::Uuid::new_v4().to_string();
            session_manager
                .create_session(initial_session_id, |sid| InteractionManager::new_session(sid, persona_repository.clone()))
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
            })
            .invoke_handler(tauri::generate_handler![
                create_session,
                list_sessions,
                switch_session,
                delete_session,
                save_current_session,
                get_active_session,
                get_personas,
                save_persona_configs,
                add_participant,
                remove_participant,
                get_active_participants,
                get_config_path,
                handle_input,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
