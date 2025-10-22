// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use orcs_interaction::{InteractionManager, InteractionResult};
use orcs_types::AppMode;
use serde::Serialize;
use tauri::State;

/// Application state shared across Tauri commands
struct AppState {
    interaction_manager: Arc<InteractionManager>,
    app_mode: Mutex<AppMode>,
}

/// Serializable version of InteractionResult for Tauri IPC
///
/// This enum mirrors the variants from `orcs_interaction::InteractionResult`
/// and implements `Serialize` for JSON serialization in Tauri's IPC layer.
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
enum SerializableInteractionResult {
    /// A new message to be displayed to the user
    NewMessage(String),
    /// The application mode has changed
    ModeChanged(AppMode),
    /// Tasks to be dispatched for execution
    TasksToDispatch { tasks: Vec<String> },
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
            InteractionResult::NoOp => SerializableInteractionResult::NoOp,
        }
    }
}

/// Tauri command to handle user input
///
/// This async command processes user input based on the current application mode
/// and returns a serializable result for the frontend.
///
/// # Arguments
///
/// * `input` - The user's input string
/// * `state` - The shared application state
///
/// # Returns
///
/// A `Result` containing either a `SerializableInteractionResult` or an error message
#[tauri::command]
async fn handle_input(
    input: String,
    state: State<'_, AppState>,
) -> Result<SerializableInteractionResult, String> {
    // Get the current mode
    let current_mode = {
        let mode_guard = state
            .app_mode
            .lock()
            .map_err(|e| format!("Failed to lock app_mode: {}", e))?;
        mode_guard.clone()
    };

    // Handle the input based on the current mode
    let result = state
        .interaction_manager
        .handle_input(&current_mode, &input)
        .await;

    // Update the mode if it changed
    if let InteractionResult::ModeChanged(ref new_mode) = result {
        let mut mode_guard = state
            .app_mode
            .lock()
            .map_err(|e| format!("Failed to lock app_mode for update: {}", e))?;
        *mode_guard = new_mode.clone();
    }

    Ok(result.into())
}

fn main() {
    // Initialize the InteractionManager
    let interaction_manager = Arc::new(InteractionManager::new());
    let app_mode = Mutex::new(AppMode::Idle);

    tauri::Builder::default()
        .manage(AppState {
            interaction_manager,
            app_mode,
        })
        .invoke_handler(tauri::generate_handler![handle_input])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
