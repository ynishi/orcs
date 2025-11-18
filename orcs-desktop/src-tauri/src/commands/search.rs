//! Tauri commands for unified search functionality.

use llm_toolkit::agent::{Agent, Payload};
use orcs_core::agent::{WebSearchAgent, WebSearchReference};
use orcs_core::search::{
    SearchFilters, SearchResult, SearchResultItem, SearchScope, SearchService,
};
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
    if request.scope == SearchScope::Global {
        return execute_global_search(&request.query, state).await;
    }

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

async fn execute_global_search(
    query: &str,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let secrets = state
        .secret_service
        .load_secrets()
        .await
        .map_err(|e| format!("Failed to load secrets: {e}"))?;

    let gemini_config = secrets
        .gemini
        .ok_or_else(|| "Gemini configuration missing in secret.json".to_string())?;

    if gemini_config.api_key.trim().is_empty() {
        return Err("Gemini API key is not configured".to_string());
    }

    let agent = WebSearchAgent::new(gemini_config.api_key);
    let payload: Payload = query.to_string().into();
    let response = agent
        .execute(payload)
        .await
        .map_err(|e| format!("WebSearch failed: {e}"))?;

    let items: Vec<SearchResultItem> = response
        .references
        .iter()
        .map(|reference| SearchResultItem {
            path: reference.title.clone(),
            line_number: None,
            content: format_reference(reference),
            context_before: None,
            context_after: None,
        })
        .collect();

    let mut result = SearchResult::new(query.to_string(), SearchScope::Global, items);
    if !response.answer.trim().is_empty() {
        result.summary = Some(response.answer);
    }

    Ok(result)
}

fn format_reference(reference: &WebSearchReference) -> String {
    let mut lines = Vec::new();
    if let Some(snippet) = &reference.snippet {
        if !snippet.trim().is_empty() {
            lines.push(snippet.trim().to_string());
        }
    }
    lines.push(reference.url.clone());
    if let Some(source) = &reference.source {
        if !source.trim().is_empty() {
            lines.push(format!("Source: {}", source));
        }
    }
    lines.join("\n")
}
