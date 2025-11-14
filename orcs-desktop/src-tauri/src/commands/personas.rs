use orcs_core::persona::{Persona, PersonaBackend};
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::workspace::manager::WorkspaceStorageService;
use tauri::State;

use crate::app::AppState;

/// Creates an adhoc expert persona and adds it to the active session
#[tauri::command]
pub async fn create_adhoc_persona(
    expertise: String,
    state: State<'_, AppState>,
) -> Result<Persona, String> {
    // Get workspace root path from active session
    let workspace_root = if let Some(session_mgr) = state.session_usecase.active_session().await {
        let app_mode = state.app_mode.lock().await.clone();
        let session = session_mgr
            .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
            .await;
        let workspace_id = &session.workspace_id;

        if workspace_id != PLACEHOLDER_WORKSPACE_ID {
            match state
                .workspace_storage_service
                .get_workspace(workspace_id)
                .await
            {
                Ok(Some(workspace)) => {
                    tracing::info!(
                        "[create_adhoc_persona] Using workspace root: {}",
                        workspace.root_path.display()
                    );
                    Some(workspace.root_path)
                }
                Ok(None) => {
                    tracing::warn!(
                        "[create_adhoc_persona] Workspace not found for id: {}, using None",
                        workspace_id
                    );
                    None
                }
                Err(e) => {
                    tracing::warn!(
                        "[create_adhoc_persona] Failed to get workspace: {}, using None",
                        e
                    );
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let persona = state
        .adhoc_persona_service
        .generate_expert(expertise, workspace_root)
        .await
        .map_err(|e| e.to_string())?;

    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    manager
        .add_participant(&persona.id)
        .await
        .map_err(|e| e.to_string())?;

    let app_mode = state.app_mode.lock().await.clone();
    let _ = state.session_usecase.save_active_session(app_mode).await;

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

    if let Some(manager) = state.session_usecase.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(persona)
}

/// Gets all personas from the single source of truth
#[tauri::command]
pub async fn get_personas(state: State<'_, AppState>) -> Result<Vec<Persona>, String> {
    state
        .persona_repository
        .get_all()
        .await
        .map_err(|e| e.to_string())
}

/// Saves persona configurations
#[tauri::command]
pub async fn save_persona_configs(
    configs: Vec<Persona>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .persona_repository
        .save_all(&configs)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(manager) = state.session_usecase.active_session().await {
        manager.invalidate_dialogue().await;
    }

    Ok(())
}

/// Gets all available persona backend options
#[tauri::command]
pub async fn get_persona_backend_options() -> Result<Vec<(String, String)>, String> {
    Ok(PersonaBackend::all_variants())
}

/// Creates a new persona from a CreatePersonaRequest (unified creation logic)
#[tauri::command]
pub async fn create_persona(
    request: orcs_core::persona::CreatePersonaRequest,
    state: State<'_, AppState>,
) -> Result<Persona, String> {
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
