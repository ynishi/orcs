use orcs_core::persona::{Persona, PersonaBackend};
use tauri::State;

use crate::app::AppState;

/// Creates an adhoc expert persona and adds it to the active session
#[tauri::command]
pub async fn create_adhoc_persona(
    expertise: String,
    state: State<'_, AppState>,
) -> Result<Persona, String> {
    let persona = state
        .adhoc_persona_service
        .generate_expert(expertise)
        .await
        .map_err(|e| e.to_string())?;

    let manager = state
        .session_manager
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .add_participant(&persona.id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_manager.save_active_session(app_mode).await;

    Ok(persona)
}

/// Saves an adhoc persona to permanent user persona storage
#[tauri::command]
pub async fn save_adhoc_persona(
    persona_id: String,
    state: State<'_, AppState>,
) -> Result<Persona, String> {
    let persona = state
        .adhoc_persona_service
        .promote_to_user(&persona_id)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(manager) = state.session_manager.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(persona)
}

/// Gets all personas from the single source of truth
#[tauri::command]
pub async fn get_personas(state: State<'_, AppState>) -> Result<Vec<Persona>, String> {
    state.persona_repository.get_all().await
        .map_err(|e| e.to_string())
}

/// Saves persona configurations
#[tauri::command]
pub async fn save_persona_configs(
    configs: Vec<Persona>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.persona_repository.save_all(&configs).await
        .map_err(|e| e.to_string())?;

    if let Some(manager) = state.session_manager.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(())
}

/// Gets all available persona backend options
#[tauri::command]
pub async fn get_persona_backend_options() -> Result<Vec<(String, String)>, String> {
    Ok(PersonaBackend::all_variants())
}

