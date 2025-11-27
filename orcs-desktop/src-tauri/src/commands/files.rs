use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::fs;

use crate::app::AppState;

/// ファイルプレビューデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreviewData {
    pub name: String,
    pub path: String,
    pub mime_type: String,
    pub size: u64,
    pub data: String, // Base64 encoded
}

/// ファイル拡張子からMIMEタイプを推測
fn guess_mime_type(file_path: &str) -> String {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        // 画像
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",

        // ドキュメント
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",

        // コード
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "ts" => "application/typescript",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        "java" => "text/x-java",

        // アーカイブ
        "zip" => "application/zip",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",

        // 動画・音声
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",

        _ => "application/octet-stream",
    }
    .to_string()
}

/// ファイルプレビューデータを取得（Base64エンコード付き）
#[tauri::command]
pub async fn get_file_preview_data(file_path: String) -> Result<FilePreviewData, String> {
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};

    // ファイルを読み込み
    let bytes = fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // メタデータを取得
    let metadata = fs::metadata(&file_path)
        .await
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let path_obj = Path::new(&file_path);
    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // MIMEタイプを推測
    let mime_type = guess_mime_type(&file_path);

    // Base64エンコード
    let data = BASE64_STANDARD.encode(&bytes);

    Ok(FilePreviewData {
        name,
        path: file_path,
        mime_type,
        size: metadata.len(),
        data,
    })
}

/// Reads a file from a workspace and returns its content as bytes
#[tauri::command]
pub async fn read_workspace_file(file_path: String) -> Result<Vec<u8>, String> {
    fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Saves code snippet to a file
#[tauri::command]
pub async fn save_code_snippet(
    file_path: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = Path::new(&file_path);

    if !path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    if let Some(manager) = state.session_usecase.active_session().await {
        manager
            .add_system_conversation_message(
                format!("Saved file: {}", file_path),
                Some("file_save".to_string()),
                None,
            )
            .await;

        let app_mode = state.app_mode.lock().await.clone();
        let _ = state.session_usecase.save_active_session(app_mode).await;
    }

    Ok(())
}

/// Opens a terminal in the specified directory
#[tauri::command]
pub async fn open_terminal(directory: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Terminal", &directory])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", "cd", "/D", &directory])
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let xterm_cmd = format!("cd '{}' && bash", directory);
        let terminals = [
            (
                "x-terminal-emulator",
                vec!["--working-directory", &directory],
            ),
            ("gnome-terminal", vec!["--working-directory", &directory]),
            ("xterm", vec!["-e", &xterm_cmd]),
        ];

        let mut success = false;
        for (terminal, args) in &terminals {
            if Command::new(terminal).args(args.iter()).spawn().is_ok() {
                success = true;
                break;
            }
        }

        if !success {
            return Err("No terminal emulator found".to_string());
        }
    }

    Ok(())
}
