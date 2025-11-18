//! Tauri commands for unified search functionality.

use orcs_core::search::{SearchFilters, SearchResult, SearchScope, SearchService};
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::workspace::manager::WorkspaceStorageService;
use orcs_infrastructure::search::RipgrepSearchService;
use serde::Deserialize;
use tauri::State;

use crate::app::AppState;

/// Request for executing a search command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    /// The search query string
    pub query: String,

    /// The scope in which to search (workspace, local, global)
    #[serde(default)]
    pub scope: SearchScope,

    /// Optional filters to refine search results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
}

/// Executes a unified search command.
#[tauri::command]
pub async fn execute_search(
    request: SearchRequest,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    tracing::info!("execute_search: Query: {}", request.query);
    tracing::info!("execute_search: Scope: {:?}", request.scope);

    // Get workspace path from active session
    let workspace_path = if request.scope != SearchScope::Global {
        let session_mgr = state
            .session_usecase
            .active_session()
            .await
            .ok_or("No active session")?;

        let app_mode = state.app_mode.lock().await.clone();
        let session = session_mgr
            .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
            .await;

        if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
            let workspace_id = &session.workspace_id;
            let workspace = state
                .workspace_storage_service
                .get_workspace(workspace_id)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Workspace not found: {}", workspace_id))?;

            Some(workspace.root_path)
        } else {
            None
        }
    } else {
        None
    };

    // Execute search using RipgrepSearchService
    let search_service = RipgrepSearchService::new();
    let result = search_service
        .search(
            &request.query,
            request.scope,
            workspace_path,
            request.filters,
        )
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("execute_search: Found {} matches", result.total_matches);

    Ok(result)
}
