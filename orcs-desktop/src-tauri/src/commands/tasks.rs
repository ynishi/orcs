use orcs_infrastructure::storage_repository::StorageRepository;
use tauri::State;

use crate::app::AppState;

/// Gets all tasks snapshot (for store initialization)
#[tauri::command]
pub async fn get_tasks_snapshot(state: State<'_, AppState>) -> Result<Vec<orcs_core::task::Task>, String> {
    state
        .task_repository
        .list_all()
        .await
        .map_err(|e| e.to_string())
}

/// Lists all saved tasks
#[tauri::command]
pub async fn list_tasks(state: State<'_, AppState>) -> Result<Vec<orcs_core::task::Task>, String> {
    state
        .task_repository
        .list_all()
        .await
        .map_err(|e| e.to_string())
}

/// Deletes a task by ID
#[tauri::command]
pub async fn delete_task(task_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .task_repository
        .delete(&task_id)
        .await
        .map_err(|e| e.to_string())
}

/// Gets the tasks directory path
#[tauri::command]
pub async fn get_tasks_directory(state: State<'_, AppState>) -> Result<String, String> {
    let tasks_dir = state.task_repository_concrete.base_dir();

    let path_str = tasks_dir
        .to_str()
        .ok_or("Tasks directory path is not valid UTF-8")?;

    Ok(path_str.to_string())
}
