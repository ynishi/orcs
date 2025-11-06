//! AsyncDirStorage-based TaskRepository implementation

use crate::dto::create_task_migrator;
use anyhow::{Context, Result};
use async_trait::async_trait;
use orcs_core::repository::TaskRepository;
use orcs_core::task::Task;
use std::path::{Path, PathBuf};
use tokio::fs;
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, PathStrategy,
};

/// AsyncDirStorage-based task repository.
///
/// Directory structure:
/// ```text
/// base_dir/
/// └── tasks/
///     ├── task-uuid-1.toml
///     └── task-uuid-2.toml
/// ```
pub struct AsyncDirTaskRepository {
    storage: AsyncDirStorage,
    _base_dir: PathBuf,
}

impl AsyncDirTaskRepository {
    /// Creates an AsyncDirTaskRepository instance at the default location.
    ///
    /// Uses centralized path management via `ServiceType::Task`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined or if
    /// the directory structure cannot be created.
    pub async fn default_location() -> Result<Self> {
        use crate::paths::{OrcsPaths, ServiceType};
        let path_type = OrcsPaths::get_path(ServiceType::Task)
            .map_err(|e| anyhow::anyhow!("Failed to get task directory: {}", e))?;
        let base_dir = path_type.into_path_buf();
        Self::new(base_dir).await
    }

    /// Creates a new AsyncDirTaskRepository.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for tasks (e.g., ~/.config/orcs)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Directory creation fails
    /// - AsyncDirStorage initialization fails
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Ensure base directory exists
        fs::create_dir_all(&base_dir)
            .await
            .context("Failed to create base directory")?;

        // Setup AppPaths with CustomBase strategy to use our base_dir
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base_dir.clone()));

        // Setup migrator
        let migrator = create_task_migrator();

        // Setup storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        let storage = AsyncDirStorage::new(paths, "tasks", migrator, strategy)
            .await
            .context("Failed to create AsyncDirStorage")?;

        Ok(Self {
            storage,
            _base_dir: base_dir,
        })
    }

    /// Returns the actual tasks directory path.
    ///
    /// This returns the real path where task files are stored.
    pub fn tasks_dir(&self) -> &Path {
        self.storage.base_path()
    }
}

#[async_trait]
impl TaskRepository for AsyncDirTaskRepository {
    async fn find_by_id(&self, task_id: &str) -> Result<Option<Task>> {
        match self.storage.load::<Task>("task", task_id).await {
            Ok(task) => Ok(Some(task)),
            Err(e) => {
                // Check if it's a "not found" error
                let error_str = e.to_string();
                if error_str.contains("No such file or directory")
                    || error_str.contains("not found")
                    || error_str.contains("cannot find")
                {
                    return Ok(None);
                }
                Err(anyhow::anyhow!(e))
            }
        }
    }

    async fn save(&self, task: &Task) -> Result<()> {
        self.storage
            .save("task", &task.id, task)
            .await
            .context("Failed to save task")
    }

    async fn delete(&self, task_id: &str) -> Result<()> {
        self.storage
            .delete(task_id)
            .await
            .context("Failed to delete task")
    }

    async fn list_all(&self) -> Result<Vec<Task>> {
        let all_tasks = self
            .storage
            .load_all::<Task>("task")
            .await
            .context("Failed to load all tasks")?;

        // Extract tasks from (id, task) tuples
        let mut tasks: Vec<Task> = all_tasks.into_iter().map(|(_, task)| task).collect();

        // Sort by created_at descending (most recent first)
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(tasks)
    }

    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Task>> {
        let all_tasks = self.list_all().await?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| task.session_id == session_id)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_core::task::TaskStatus;
    use tempfile::TempDir;

    async fn create_test_repository() -> (AsyncDirTaskRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let repo = AsyncDirTaskRepository::new(temp_dir.path()).await.unwrap();
        (repo, temp_dir)
    }

    fn create_test_task(id: &str, session_id: &str, title: &str) -> Task {
        Task {
            id: id.to_string(),
            session_id: session_id.to_string(),
            title: title.to_string(),
            description: format!("Description for {}", title),
            status: TaskStatus::Completed,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:01:00Z".to_string(),
            completed_at: Some("2025-01-01T00:01:00Z".to_string()),
            steps_executed: 5,
            steps_skipped: 0,
            context_keys: 6,
            error: None,
            result: Some("Task completed successfully".to_string()),
            execution_details: None,
        }
    }

    #[tokio::test]
    async fn test_save_and_find_task() {
        let (repo, _temp_dir) = create_test_repository().await;

        let task = create_test_task(
            "550e8400-e29b-41d4-a716-446655440000",
            "session-1",
            "Test Task",
        );

        // Save task
        repo.save(&task).await.unwrap();

        // Find task
        let found = repo.find_by_id(&task.id).await.unwrap();
        assert!(found.is_some());
        let found_task = found.unwrap();
        assert_eq!(found_task.id, task.id);
        assert_eq!(found_task.title, task.title);
        assert_eq!(found_task.session_id, task.session_id);
    }

    #[tokio::test]
    async fn test_list_by_session() {
        let (repo, _temp_dir) = create_test_repository().await;

        // Create tasks for different sessions with valid UUIDs
        let task1_id = "550e8400-e29b-41d4-a716-446655440001";
        let task2_id = "550e8400-e29b-41d4-a716-446655440002";
        let task3_id = "550e8400-e29b-41d4-a716-446655440003";

        let task1 = create_test_task(task1_id, "session-1", "Task 1");
        let task2 = create_test_task(task2_id, "session-1", "Task 2");
        let task3 = create_test_task(task3_id, "session-2", "Task 3");

        repo.save(&task1).await.unwrap();
        repo.save(&task2).await.unwrap();
        repo.save(&task3).await.unwrap();

        // List tasks for session-1
        let session1_tasks = repo.list_by_session("session-1").await.unwrap();
        assert_eq!(session1_tasks.len(), 2);
        assert!(session1_tasks.iter().all(|t| t.session_id == "session-1"));

        // List tasks for session-2
        let session2_tasks = repo.list_by_session("session-2").await.unwrap();
        assert_eq!(session2_tasks.len(), 1);
        assert_eq!(session2_tasks[0].id, task3_id);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let (repo, _temp_dir) = create_test_repository().await;

        let task_id = "550e8400-e29b-41d4-a716-446655440004";
        let task = create_test_task(task_id, "session-1", "Delete Me");
        repo.save(&task).await.unwrap();

        // Verify task exists
        assert!(repo.find_by_id(&task.id).await.unwrap().is_some());

        // Delete task
        repo.delete(&task.id).await.unwrap();

        // Verify task is gone
        assert!(repo.find_by_id(&task.id).await.unwrap().is_none());
    }
}
