//! Tauri commands for unified search functionality.
//!
//! Search options:
//! - Default: current workspace sessions + workspace files
//! - `-p`: + project files (workspace.root_path)
//! - `-a`: all workspaces sessions + files
//! - `-f` (or `-ap`): all + project files
//! - `-m`: search Kaiba memory (RAG semantic search)

use orcs_core::memory::MemorySyncService;
use orcs_core::repository::SessionRepository;
use orcs_core::search::{SearchFilters, SearchOptions, SearchResult, SearchResultItem, SearchService};
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::workspace::manager::WorkspaceStorageService;
use orcs_infrastructure::paths::{OrcsPaths, ServiceType};
use orcs_infrastructure::search::RipgrepSearchService;
use orcs_interaction::KaibaMemorySyncService;
use serde::Deserialize;
use std::path::PathBuf;
use tauri::State;

use crate::app::AppState;

/// Request for executing a search command.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    /// The search query string
    pub query: String,

    /// Search options
    #[serde(default)]
    pub options: SearchOptions,

    /// Optional filters to refine search results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
}

/// Executes a unified search command.
///
/// Builds search paths based on options:
/// - Default: sessions (current workspace) + workspace_dir
/// - `-p`: + project root_path
/// - `-a`: all sessions + all workspace_dirs
/// - `-f`: all + project root_path
/// - `-m`: search Kaiba memory (RAG semantic search)
#[tauri::command]
pub async fn execute_search(
    request: SearchRequest,
    state: State<'_, AppState>,
) -> Result<SearchResult, String> {
    tracing::info!(
        "execute_search: Query: {}, Options: {:?}",
        request.query,
        request.options
    );

    // If memory search is requested, use Kaiba
    if request.options.search_memory {
        return execute_memory_search(&request, &state).await;
    }

    // Build search paths based on options
    let search_paths = build_search_paths(&request.options, &state).await?;

    tracing::info!("execute_search: Search paths: {:?}", search_paths);

    if search_paths.is_empty() {
        return Ok(SearchResult::empty(request.query, request.options));
    }

    // Execute search using RipgrepSearchService
    let search_service = RipgrepSearchService::new();
    let result = search_service
        .search(
            &request.query,
            request.options,
            search_paths,
            request.filters,
        )
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("execute_search: Found {} matches", result.total_matches);

    Ok(result)
}

/// Executes a memory search using Kaiba RAG.
async fn execute_memory_search(
    request: &SearchRequest,
    state: &State<'_, AppState>,
) -> Result<SearchResult, String> {
    // Get current workspace to find kaiba_rei_id
    let workspace = get_current_workspace(state).await?;

    let Some(workspace) = workspace else {
        return Err("No workspace selected. Memory search requires an active workspace.".to_string());
    };

    let Some(rei_id) = workspace.kaiba_rei_id else {
        return Err("No Kaiba Rei configured for this workspace. Memory sync has not been performed yet.".to_string());
    };

    // Create KaibaMemorySyncService
    let sync_service = KaibaMemorySyncService::try_from_env()
        .await
        .map_err(|e| format!("Failed to initialize Kaiba: {}", e))?;

    // Determine result limit
    let limit = request
        .filters
        .as_ref()
        .and_then(|f| f.max_results)
        .unwrap_or(20);

    tracing::info!(
        "execute_memory_search: Searching Kaiba rei={} query='{}' limit={}",
        rei_id,
        request.query,
        limit
    );

    // Execute semantic search
    let memories = sync_service
        .search_memories(&rei_id, &request.query, limit)
        .await
        .map_err(|e| format!("Kaiba search failed: {}", e))?;

    // Convert to SearchResultItems
    let items: Vec<SearchResultItem> = memories
        .into_iter()
        .map(|m| SearchResultItem {
            path: format!("[memory:{}]", m.id),
            line_number: None,
            content: m.content,
            context_before: None,
            context_after: None,
        })
        .collect();

    let total_matches = items.len();

    tracing::info!(
        "execute_memory_search: Found {} memories",
        total_matches
    );

    Ok(SearchResult {
        query: request.query.clone(),
        options: request.options.clone(),
        items,
        summary: Some(format!(
            "Found {} relevant memories from Kaiba RAG",
            total_matches
        )),
        total_matches,
    })
}

/// Builds search paths based on search options.
async fn build_search_paths(
    options: &SearchOptions,
    state: &State<'_, AppState>,
) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();

    // Get sessions directory
    let sessions_dir = OrcsPaths::new(None)
        .get_path(ServiceType::Session)
        .map_err(|e| e.to_string())?
        .into_path_buf();

    // Get current workspace info
    let current_workspace = get_current_workspace(state).await?;

    if options.all_workspaces {
        // -a or -f: Search all sessions
        if sessions_dir.exists() {
            paths.push(sessions_dir);
        }

        // Search all workspace storage directories
        let workspace_storage_dir = OrcsPaths::new(None)
            .get_path(ServiceType::WorkspaceStorage)
            .map_err(|e| e.to_string())?
            .into_path_buf();

        if workspace_storage_dir.exists() {
            paths.push(workspace_storage_dir);
        }
    } else {
        // Default or -p: Search current workspace only
        if let Some(ref ws) = current_workspace {
            // Search sessions for current workspace (filtered by workspace_id)
            if sessions_dir.exists() {
                let workspace_session_paths =
                    get_workspace_session_paths(&ws.id, &sessions_dir, state).await?;
                paths.extend(workspace_session_paths);
            }

            // Search current workspace storage directory
            if ws.workspace_dir.exists() {
                paths.push(ws.workspace_dir.clone());
            }
        }
    }

    // -p or -f: Include project files
    if options.include_project {
        if let Some(ref ws) = current_workspace {
            if ws.root_path.exists() {
                paths.push(ws.root_path.clone());
            }
        }
    }

    Ok(paths)
}

/// Gets the current workspace from the active session.
async fn get_current_workspace(
    state: &State<'_, AppState>,
) -> Result<Option<orcs_core::workspace::Workspace>, String> {
    let session_mgr = match state.session_usecase.active_session().await {
        Some(mgr) => mgr,
        None => return Ok(None),
    };

    let app_mode = state.app_mode.lock().await.clone();
    let session = session_mgr
        .to_session(app_mode, PLACEHOLDER_WORKSPACE_ID.to_string())
        .await;

    if session.workspace_id == PLACEHOLDER_WORKSPACE_ID {
        return Ok(None);
    }

    let workspace = state
        .workspace_storage_service
        .get_workspace(&session.workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(workspace)
}

/// Gets session file paths for a specific workspace.
async fn get_workspace_session_paths(
    workspace_id: &str,
    sessions_dir: &PathBuf,
    state: &State<'_, AppState>,
) -> Result<Vec<PathBuf>, String> {
    // Get all sessions from repository
    let all_sessions = state
        .session_repository
        .list_all()
        .await
        .map_err(|e| e.to_string())?;

    // Filter sessions by workspace_id and build file paths
    // sessions_dir is data_dir/sessions, so session files are at sessions_dir/{session_id}.toml
    let session_paths: Vec<PathBuf> = all_sessions
        .iter()
        .filter(|session| session.workspace_id == workspace_id)
        .map(|session| sessions_dir.join(format!("{}.toml", session.id)))
        .filter(|path| path.exists())
        .collect();

    tracing::info!(
        "get_workspace_session_paths: Found {} sessions for workspace {}",
        session_paths.len(),
        workspace_id
    );

    Ok(session_paths)
}
