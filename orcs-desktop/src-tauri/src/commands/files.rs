use std::path::Path;
use std::process::Command;

use tauri::State;
use tokio::fs;

use crate::app::AppState;

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

    if let Some(manager) = state.session_manager.active_session().await {
        manager
            .add_system_conversation_message(
                format!("Saved file: {}", file_path),
                Some("file_save".to_string()),
                None,
            )
            .await;

        let app_mode = state.app_mode.lock().await.clone();
        let _ = state.session_manager.save_active_session(app_mode).await;
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
        let terminals = [
            ("x-terminal-emulator", vec!["--working-directory", &directory]),
            ("gnome-terminal", vec!["--working-directory", &directory]),
            ("xterm", vec!["-e", &format!("cd '{}' && bash", directory)]),
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

