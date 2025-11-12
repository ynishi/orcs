use std::path::{Path, PathBuf};

use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::state::repository::StateRepository;
use orcs_core::workspace::{UploadedFile, Workspace, manager::WorkspaceManager};
use tauri::{AppHandle, Emitter, State};

use crate::app::AppState;

/// Gets the current workspace based on the active session
#[tauri::command]
pub async fn get_current_workspace(state: State<'_, AppState>) -> Result<Workspace, String> {
    println!("[Backend] get_current_workspace called");

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

    println!("[Backend] No AppStateService workspace, checking session");
    let manager = state
        .session_usecase
        .active_session()
        .await
        .ok_or("No active session")?;

    let app_mode = state.app_mode.lock().await.clone();
    let session = manager
        .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;

    println!("[Backend] Session workspace_id: {:?}", session.workspace_id);

    if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
        let workspace_id = &session.workspace_id;
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

    Err("No workspace selected or associated with session".to_string())
}

/// Creates a new workspace for the given directory path
#[tauri::command]
pub async fn create_workspace(
    root_path: String,
    state: State<'_, AppState>,
) -> Result<Workspace, String> {
    let path = PathBuf::from(root_path);
    state
        .workspace_manager
        .get_or_create_workspace(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Creates a new workspace and immediately creates a session associated with it.
///
/// This is the recommended way to create workspaces, as a workspace without
/// a session doesn't make sense. This ensures both workspace and session are
/// created atomically, and the workspace is set as the currently selected workspace.
///
/// Returns: { workspace: Workspace, session: Session }
#[tauri::command]
pub async fn create_workspace_with_session(
    root_path: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!(
        "[Backend] create_workspace_with_session called: path={}",
        root_path
    );

    let path = PathBuf::from(root_path);
    let (workspace, session) = state
        .session_usecase
        .create_workspace_with_session(&path)
        .await
        .map_err(|e| {
            println!("[Backend] Failed to create workspace with session: {}", e);
            e.to_string()
        })?;

    println!(
        "[Backend] Successfully created workspace {} and session {}",
        workspace.id, session.id
    );

    // Return both workspace and session as JSON
    Ok(serde_json::json!({
        "workspace": workspace,
        "session": session,
    }))
}

/// Lists all registered workspaces
#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    state
        .workspace_manager
        .list_all_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Switches to a different workspace for the active session
#[tauri::command]
pub async fn switch_workspace(
    _session_id: String,
    workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "[Backend] switch_workspace called: workspace_id={}",
        workspace_id
    );

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

    app.emit("workspace-switched", &workspace_id)
        .map_err(|e| e.to_string())?;

    println!("[Backend] workspace-switched event emitted");

    Ok(())
}

/// Toggles the favorite status of a workspace
#[tauri::command]
pub async fn toggle_favorite_workspace(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_manager
        .toggle_favorite(&workspace_id)
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a workspace
#[tauri::command]
pub async fn delete_workspace(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
pub async fn list_workspace_files(
    workspace_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<UploadedFile>, String> {
    let workspace = state
        .workspace_manager
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(workspace
        .map(|w| w.resources.uploaded_files)
        .unwrap_or_default())
}

/// Uploads a file to a workspace
#[tauri::command]
pub async fn upload_file_to_workspace(
    workspace_id: String,
    local_file_path: String,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    let file_path = Path::new(&local_file_path);

    state
        .workspace_manager
        .add_file_to_workspace(&workspace_id, file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Uploads a file to a workspace from binary data
#[tauri::command]
pub async fn upload_file_from_bytes(
    workspace_id: String,
    filename: String,
    file_data: Vec<u8>,
    session_id: Option<String>,
    message_timestamp: Option<String>,
    author: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<UploadedFile, String> {
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
pub async fn delete_file_from_workspace(
    workspace_id: String,
    file_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_manager
        .delete_file_from_workspace(&workspace_id, &file_id)
        .await
        .map_err(|e| e.to_string())
}

/// Renames a file in a workspace
#[tauri::command]
pub async fn rename_file_in_workspace(
    workspace_id: String,
    file_id: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    state
        .workspace_manager
        .rename_file_in_workspace(&workspace_id, &file_id, &new_name)
        .await
        .map_err(|e| e.to_string())
}
