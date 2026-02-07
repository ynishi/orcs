use orcs_core::dialogue::DialoguePreset;
use tauri::State;

use crate::app::AppState;

/// Gets all dialogue presets (system + user)
#[tauri::command]
pub async fn get_dialogue_presets(
    state: State<'_, AppState>,
) -> Result<Vec<DialoguePreset>, String> {
    state
        .dialogue_preset_repository
        .get_all()
        .await
        .map_err(|e| e.to_string())
}

/// Saves a user dialogue preset
#[tauri::command]
pub async fn save_dialogue_preset(
    preset: DialoguePreset,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .dialogue_preset_repository
        .save(&preset)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a user dialogue preset by ID
#[tauri::command]
pub async fn delete_dialogue_preset(
    preset_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .dialogue_preset_repository
        .delete(&preset_id)
        .await
        .map_err(|e| e.to_string())
}

/// Applies a dialogue preset to the active session
#[tauri::command]
pub async fn apply_dialogue_preset(
    preset_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Load the preset
    let preset = state
        .dialogue_preset_repository
        .find_by_id(&preset_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Preset with ID '{}' not found", preset_id))?;

    // Get active session manager
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    // Apply preset settings
    manager
        .set_execution_strategy(preset.execution_strategy)
        .await;
    manager
        .set_conversation_mode(preset.conversation_mode)
        .await;

    if let Some(style) = preset.talk_style {
        manager.set_talk_style(Some(style)).await;
    }

    // Merge default personas (add if not already active, skip duplicates)
    if !preset.default_persona_ids.is_empty() {
        let active_ids = manager.get_active_participants().await.unwrap_or_default();
        for persona_id in &preset.default_persona_ids {
            if !active_ids.contains(persona_id) {
                if let Err(e) = manager.add_participant(persona_id).await {
                    eprintln!("Failed to add preset persona '{}': {}", persona_id, e);
                }
            }
        }
    }

    // Save session with app_mode
    let app_mode = state.app_mode.lock().await.clone();
    state
        .session_usecase
        .save_active_session(app_mode)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
