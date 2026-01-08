use std::path::{Path, PathBuf};

use llm_toolkit::agent::Agent;
use llm_toolkit::agent::impls::claude_code::ClaudeCodeAgent;
use orcs_core::agent::build_enhanced_path;
use orcs_core::session::PLACEHOLDER_WORKSPACE_ID;
use orcs_core::state::repository::StateRepository;
use orcs_core::workspace::{UploadedFile, Workspace, manager::WorkspaceStorageService};
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
            .workspace_storage_service
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
            .workspace_storage_service
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
        .workspace_storage_service
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
    app: AppHandle,
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

    // Emit workspace:update event (Phase 4)
    if let Err(e) = app.emit("workspace:update", &workspace) {
        println!("[Backend] Failed to emit workspace:update: {}", e);
    }

    // Emit workspace-switched event to trigger frontend session switching
    if let Err(e) = app.emit("workspace-switched", &workspace.id) {
        println!("[Backend] Failed to emit workspace-switched: {}", e);
    } else {
        println!(
            "[Backend] workspace-switched event emitted for {}",
            workspace.id
        );
    }

    // Emit app-state:update event for SSOT synchronization
    use orcs_core::state::repository::StateRepository;
    if let Ok(app_state) = state.app_state_service.get_state().await {
        let _ = app.emit("app-state:update", &app_state);
    }

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
        .workspace_storage_service
        .list_all_workspaces()
        .await
        .map_err(|e| e.to_string())
}

/// Gets all workspaces snapshot for initial load (Phase 4)
#[tauri::command]
pub async fn get_workspaces_snapshot(state: State<'_, AppState>) -> Result<Vec<Workspace>, String> {
    state
        .workspace_storage_service
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

    // Save last selected workspace for app restart restoration (Phase 3)
    use orcs_core::state::repository::StateRepository;
    if let Err(e) = state
        .app_state_service
        .set_last_selected_workspace(workspace_id.clone())
        .await
    {
        println!("[Backend] Failed to save last_selected_workspace: {}", e);
    }

    app.emit("workspace-switched", &workspace_id)
        .map_err(|e| e.to_string())?;

    println!("[Backend] workspace-switched event emitted");

    // Emit app-state:update event for SSOT synchronization
    match state.app_state_service.get_state().await {
        Ok(app_state) => {
            if let Err(e) = app.emit("app-state:update", &app_state) {
                println!("[Backend] Failed to emit app-state:update: {}", e);
            } else {
                println!("[Backend] app-state:update event emitted");
            }
        }
        Err(e) => {
            println!("[Backend] Failed to get app state: {}", e);
        }
    }

    Ok(())
}

/// Toggles the favorite status of a workspace
#[tauri::command]
pub async fn toggle_favorite_workspace(
    workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_storage_service
        .toggle_favorite(&workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        println!("[Backend] Failed to emit workspace:update: {}", e);
    }

    Ok(())
}

/// Deletes a workspace
#[tauri::command]
pub async fn delete_workspace(
    workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    println!(
        "[Backend] delete_workspace called: workspace_id={}",
        workspace_id
    );

    state
        .workspace_storage_service
        .delete_workspace(&workspace_id)
        .await
        .map_err(|e| {
            println!("[Backend] Failed to delete workspace: {}", e);
            e.to_string()
        })?;

    // Emit workspace:delete event (Phase 4)
    if let Err(e) = app.emit(
        "workspace:delete",
        serde_json::json!({ "id": workspace_id }),
    ) {
        println!("[Backend] Failed to emit workspace:delete: {}", e);
    }

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
        .workspace_storage_service
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
        .workspace_storage_service
        .add_file_to_workspace(&workspace_id, file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Uploads a file to a workspace from binary data
#[tauri::command]
#[allow(clippy::too_many_arguments)]
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
        .workspace_storage_service
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

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
    {
        if let Err(e) = app.emit("workspace:update", &workspace) {
            tracing::error!("Failed to emit workspace:update: {}", e);
        } else {
            tracing::info!(
                "upload_file_from_bytes: Emitted workspace:update event for workspace: {}",
                workspace_id
            );
        }
    }

    Ok(result)
}

/// Deletes a file from a workspace
#[tauri::command]
pub async fn delete_file_from_workspace(
    workspace_id: String,
    file_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_storage_service
        .delete_file_from_workspace(&workspace_id, &file_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(())
}

/// Renames a file in a workspace
#[tauri::command]
pub async fn rename_file_in_workspace(
    workspace_id: String,
    file_id: String,
    new_name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    let result = state
        .workspace_storage_service
        .rename_file_in_workspace(&workspace_id, &file_id, &new_name)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(result)
}

/// Toggles the archive status of a file in a workspace
#[tauri::command]
pub async fn toggle_workspace_file_archive(
    workspace_id: String,
    file_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_storage_service
        .toggle_file_archive(&workspace_id, &file_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(())
}

/// Toggles the favorite status of a file in a workspace
#[tauri::command]
pub async fn toggle_workspace_file_favorite(
    workspace_id: String,
    file_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_storage_service
        .toggle_file_favorite(&workspace_id, &file_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(())
}

/// Moves a file's sort order within favorited files
#[tauri::command]
pub async fn move_workspace_file_sort_order(
    workspace_id: String,
    file_id: String,
    direction: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .workspace_storage_service
        .move_file_sort_order(&workspace_id, &file_id, &direction)
        .await
        .map_err(|e| e.to_string())?;

    // Get updated workspace and emit event (Phase 4)
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(())
}

/// Investigates the current workspace using Claude Code Agent
///
/// This command launches a Claude Code Agent to perform comprehensive
/// investigation of the workspace, including project structure, recent
/// development activity, and technical analysis.
#[tauri::command]
pub async fn investigate_workspace(
    workspace_id: String,
    investigation_focus: Option<String>,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    println!(
        "[Backend] investigate_workspace called: workspace_id={}",
        workspace_id
    );

    // Get workspace information
    let workspace = state
        .workspace_storage_service
        .get_workspace(&workspace_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Workspace not found")?;

    let workspace_path = workspace.root_path.clone();
    let workspace_name = workspace.name.clone();

    // Build enhanced PATH for the agent
    let enhanced_path = build_enhanced_path(&workspace_path, None);

    // Create Claude Code Agent with workspace context
    // Allow tools needed for investigation: Read, Bash, Glob, Grep, Edit, Write
    let agent = ClaudeCodeAgent::new()
        .with_args(vec![
            "--allowed-tools".to_string(),
            "Read,Bash,Glob,Grep,Edit,Write".to_string(),
        ])
        .with_cwd(workspace_path.clone())
        .with_env("PATH", enhanced_path);

    // Build investigation prompt
    let focus_instruction = investigation_focus
        .map(|f| format!("\n\n## 特に注目してほしい点\n{}", f))
        .unwrap_or_default();

    let prompt = format!(
        r#"# Workspace Investigation Report Request

あなたはプロジェクト分析のエキスパートです。以下のワークスペースを調査し、**実用的なレポート**を作成してください。

## 対象ワークスペース
- **名前**: {workspace_name}
- **パス**: {workspace_path}

## 調査項目と出力形式

### 1. プロジェクト概要 (Project Overview)
- プロジェクトの目的・ゴール
- 主要な技術スタック（言語、フレームワーク、ライブラリ）
- アーキテクチャの特徴（モノレポ、マイクロサービス、モノリス等）

### 2. ディレクトリ構造 (Directory Structure)
- 主要なディレクトリとその役割
- 重要な設定ファイル（Cargo.toml, package.json, tsconfig.json等）の概要

### 3. 特徴的な機能・モジュール (Key Features)
- このプロジェクト固有の特徴的な実装
- 主要なモジュール/クレート/パッケージとその責務
- 外部APIやサービスとの連携

### 4. 開発状況 (Development Status)
- 現在のGitブランチと最近のコミット（直近5-10件）
- 最近活発に開発されている機能・ファイル
- 未解決のTODO/FIXME（主要なもの）

### 5. コードベースの健全性 (Code Health)
- テストの有無と概要
- CI/CD設定の有無
- ドキュメントの充実度

### 6. 推奨アクション (Recommendations)
- 改善が必要な箇所
- 技術的負債の兆候
- 次に取り組むべき優先事項
{focus_instruction}

## 出力形式
- Markdown形式で構造化
- 各セクションは具体的な情報を含める
- コードパスは相対パスで記載
- 不明な点は「調査が必要」と明記

## 注意事項
- ファイルを読んで実際の内容を確認すること
- 推測ではなく、実際のコードに基づいた分析を行うこと
- 長すぎるファイルは冒頭部分のみで判断してよい
"#,
        workspace_name = workspace_name,
        workspace_path = workspace_path.display(),
        focus_instruction = focus_instruction
    );

    // Execute the agent
    let report = agent
        .execute(prompt.as_str().into())
        .await
        .map_err(|e| format!("Agent execution failed: {}", e))?;

    // Build response
    let result = serde_json::json!({
        "workspace": {
            "id": workspace.id,
            "name": workspace_name,
            "path": workspace_path,
            "last_accessed": workspace.last_accessed,
            "is_favorite": workspace.is_favorite
        },
        "report": report,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "agent": "ClaudeCodeAgent"
    });

    println!("[Backend] Workspace investigation completed via Claude Code Agent");
    Ok(result)
}

/// Copies a file from one workspace to another
#[tauri::command]
pub async fn copy_file_to_workspace(
    source_workspace_id: String,
    file_id: String,
    target_workspace_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<UploadedFile, String> {
    let result = state
        .workspace_storage_service
        .copy_file_to_workspace(&source_workspace_id, &file_id, &target_workspace_id)
        .await
        .map_err(|e| e.to_string())?;

    // Emit workspace:update event for the target workspace
    if let Some(workspace) = state
        .workspace_storage_service
        .get_workspace(&target_workspace_id)
        .await
        .map_err(|e| e.to_string())?
        && let Err(e) = app.emit("workspace:update", &workspace)
    {
        tracing::error!("Failed to emit workspace:update: {}", e);
    }

    Ok(result)
}
