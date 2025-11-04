//! File system-based workspace manager implementation.
//!
//! This module provides a file system-based implementation of the `WorkspaceManager` trait,
//! storing workspace metadata and files in the `~/.orcs/workspaces` directory.

use crate::async_dir_workspace_repository::AsyncDirWorkspaceRepository;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

use orcs_core::error::{OrcsError, Result};
use orcs_core::workspace::manager::WorkspaceManager;
use orcs_core::workspace::{
    ProjectContext, SessionWorkspace, TempFile, UploadedFile, Workspace, WorkspaceResources,
};

/// Infers the MIME type from a filename extension using the `mime_guess` library.
///
/// This provides comprehensive MIME type detection based on file extensions.
fn infer_mime_type(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

/// File system-based workspace manager.
///
/// Stores workspace data in the `~/.orcs/workspaces` directory. Each workspace
/// is stored in a subdirectory named by its workspace ID, containing:
/// - Workspace metadata stored via AsyncDirWorkspaceRepository
/// - `files/` - Uploaded and generated files
/// - `temp/` - Temporary session files
pub struct FileSystemWorkspaceManager {
    /// Root directory for all workspaces (typically `~/.orcs/workspaces`)
    root_dir: PathBuf,
    /// Repository for workspace data persistence
    workspace_repository: Arc<AsyncDirWorkspaceRepository>,
}

impl FileSystemWorkspaceManager {
    /// Creates a new `FileSystemWorkspaceManager` instance.
    ///
    /// Initializes the root directory and ensures it exists on the file system.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The root directory for workspaces (typically `~/.orcs/workspaces`)
    ///
    /// # Returns
    ///
    /// Returns a new `FileSystemWorkspaceManager` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or accessed.
    pub async fn new(root_dir: PathBuf) -> Result<Self> {
        // Ensure the root directory exists
        fs::create_dir_all(&root_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create workspace root directory '{}': {}",
                root_dir.display(),
                e
            ))
        })?;

        // Initialize AsyncDirWorkspaceRepository
        let workspace_repository = Arc::new(AsyncDirWorkspaceRepository::default_location().await?);

        Ok(Self {
            root_dir,
            workspace_repository,
        })
    }

    /// Generates a deterministic workspace ID from a repository path.
    ///
    /// Uses the canonicalized path to ensure consistency, creating a UUID v5
    /// based on the path string.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the repository
    ///
    /// # Returns
    ///
    /// Returns a workspace ID string.
    fn get_workspace_id(repo_path: &Path) -> Result<String> {
        let canonical_path = repo_path.canonicalize().map_err(|e| {
            OrcsError::Io(format!(
                "Failed to canonicalize repository path '{}': {}",
                repo_path.display(),
                e
            ))
        })?;

        let path_str = canonical_path.to_string_lossy();
        let workspace_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, path_str.as_bytes());

        Ok(workspace_id.to_string())
    }

    /// Returns the actual workspaces root directory path.
    ///
    /// This returns the real path where workspace directories are stored.
    pub fn workspaces_root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Gets the workspace directory path for a given workspace ID.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// Returns the path to the workspace directory.
    fn get_workspace_dir(&self, workspace_id: &str) -> PathBuf {
        self.root_dir.join(workspace_id)
    }

    /// Loads workspace metadata via AsyncDirWorkspaceRepository.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID
    ///
    /// # Returns
    ///
    /// Returns the workspace domain model if it exists and is valid.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be read or parsed.
    async fn load_workspace(&self, workspace_id: &str) -> Result<Workspace> {
        let mut workspace = self
            .workspace_repository
            .find_by_id(workspace_id)
            .await?
            .ok_or_else(|| OrcsError::Io(format!("Workspace '{}' not found", workspace_id)))?;

        // Set workspace_dir (calculated from workspace_id, not stored in repository)
        workspace.workspace_dir = self.get_workspace_dir(workspace_id);

        Ok(workspace)
    }

    /// Saves workspace metadata via AsyncDirWorkspaceRepository.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace to save
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be written.
    async fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        let workspace_dir = self.get_workspace_dir(&workspace.id);

        // Ensure workspace directory exists (for actual files)
        fs::create_dir_all(&workspace_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create workspace directory '{}': {}",
                workspace_dir.display(),
                e
            ))
        })?;

        // Save via repository
        self.workspace_repository.save(workspace).await?;

        Ok(())
    }

    /// Extracts the workspace name from a repository path.
    ///
    /// Uses the last component of the path as the workspace name.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The path to the repository
    ///
    /// # Returns
    ///
    /// Returns the workspace name.
    fn get_workspace_name(repo_path: &Path) -> String {
        repo_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unnamed-workspace")
            .to_string()
    }
}

#[async_trait]
impl WorkspaceManager for FileSystemWorkspaceManager {

    async fn get_or_create_workspace(&self, repo_path: &Path) -> Result<Workspace> {
        // Validate root_path: must not be root directory
        let canonical_path = repo_path.canonicalize().map_err(|e| {
            OrcsError::Io(format!("Failed to canonicalize path {:?}: {}", repo_path, e))
        })?;

        if canonical_path == Path::new("/") || canonical_path == Path::new("C:\\") {
            return Err(OrcsError::Io(
                "Cannot create workspace at root directory '/'".to_string(),
            ));
        }

        let workspace_id = Self::get_workspace_id(&canonical_path)?;

        // Check if workspace already exists
        if let Ok(Some(workspace)) = self.get_workspace(&workspace_id).await {
            return Ok(workspace);
        }

        // Create new workspace
        let workspace_dir = self.get_workspace_dir(&workspace_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OrcsError::Io(format!("Failed to get current timestamp: {}", e)))?
            .as_secs() as i64;

        let workspace = Workspace {
            id: workspace_id.clone(),
            name: Self::get_workspace_name(&canonical_path),
            root_path: canonical_path,
            workspace_dir,
            resources: WorkspaceResources::default(),
            project_context: ProjectContext::default(),
            last_accessed: now,
            is_favorite: false,
            last_active_session_id: None,
        };

        // Save via repository
        self.save_workspace(&workspace).await?;

        Ok(workspace)
    }

    async fn get_workspace(&self, workspace_id: &str) -> Result<Option<Workspace>> {
        match self.load_workspace(workspace_id).await {
            Ok(workspace) => Ok(Some(workspace)),
            Err(OrcsError::Io(msg)) if msg.contains("not found") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn add_file_to_workspace(
        &self,
        workspace_id: &str,
        source_path: &Path,
    ) -> Result<UploadedFile> {
        // Load existing workspace metadata
        let mut workspace = self.load_workspace(workspace_id).await?;

        // Get the workspace directory
        let workspace_dir = self.get_workspace_dir(workspace_id);

        // Construct the destination directory (resources/uploaded)
        let uploaded_dir = workspace_dir.join("resources").join("uploaded");

        // Ensure the destination directory exists
        fs::create_dir_all(&uploaded_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create uploaded directory '{}': {}",
                uploaded_dir.display(),
                e
            ))
        })?;

        // Generate a unique ID for the file
        let file_id = Uuid::new_v4().to_string();

        // Get the original filename
        let file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| OrcsError::Io("Invalid source file path".to_string()))?
            .to_string();

        // Construct the destination path
        let dest_path = uploaded_dir.join(&file_name);

        // Copy the file
        fs::copy(source_path, &dest_path).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to copy file from '{}' to '{}': {}",
                source_path.display(),
                dest_path.display(),
                e
            ))
        })?;

        // Get file metadata for size
        let metadata = fs::metadata(&dest_path).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to read file metadata for '{}': {}",
                dest_path.display(),
                e
            ))
        })?;
        let file_size = metadata.len();

        // Determine MIME type (simplified - you may want to use a proper library)
        let mime_type = infer_mime_type(&file_name);

        // Get current timestamp
        let uploaded_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OrcsError::Io(format!("Failed to get current timestamp: {}", e)))?
            .as_secs() as i64;

        // Create the UploadedFile domain model
        let uploaded_file = UploadedFile {
            id: file_id,
            name: file_name,
            path: dest_path,
            mime_type,
            size: file_size,
            uploaded_at,
            session_id: None,
            message_timestamp: None,
            author: None,
        };

        // Add to workspace's uploaded_files list
        workspace
            .resources
            .uploaded_files
            .push(uploaded_file.clone());

        // Save the updated workspace metadata
        self.save_workspace(&workspace).await?;

        Ok(uploaded_file)
    }

    async fn add_file_from_bytes(
        &self,
        workspace_id: &str,
        filename: &str,
        data: &[u8],
        session_id: Option<String>,
        message_timestamp: Option<String>,
        author: Option<String>,
    ) -> Result<UploadedFile> {
        // Load existing workspace metadata
        let mut workspace = self.load_workspace(workspace_id).await?;

        // Get the workspace directory
        let workspace_dir = self.get_workspace_dir(workspace_id);

        // Construct the destination directory (resources/uploaded)
        let uploaded_dir = workspace_dir.join("resources").join("uploaded");

        // Ensure the destination directory exists
        fs::create_dir_all(&uploaded_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create uploaded directory '{}': {}",
                uploaded_dir.display(),
                e
            ))
        })?;

        // Generate a unique ID for the file
        let file_id = Uuid::new_v4().to_string();

        // Construct the destination path
        let dest_path = uploaded_dir.join(filename);

        // Write the byte data to the file
        fs::write(&dest_path, data).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to write file to '{}': {}",
                dest_path.display(),
                e
            ))
        })?;

        // Get file size from data length
        let file_size = data.len() as u64;

        // Determine MIME type
        let mime_type = infer_mime_type(filename);

        // Get current timestamp
        let uploaded_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OrcsError::Io(format!("Failed to get current timestamp: {}", e)))?
            .as_secs() as i64;

        // Create the UploadedFile domain model
        let uploaded_file = UploadedFile {
            id: file_id,
            name: filename.to_string(),
            path: dest_path,
            mime_type,
            size: file_size,
            uploaded_at,
            session_id,
            message_timestamp,
            author,
        };

        // Add to workspace's uploaded_files list
        workspace
            .resources
            .uploaded_files
            .push(uploaded_file.clone());

        // Save the updated workspace metadata
        self.save_workspace(&workspace).await?;

        Ok(uploaded_file)
    }

    async fn delete_file_from_workspace(&self, workspace_id: &str, file_id: &str) -> Result<()> {
        // Load existing workspace metadata
        let mut workspace = self.load_workspace(workspace_id).await?;

        // Find the file in the uploaded_files list
        let file_index = workspace
            .resources
            .uploaded_files
            .iter()
            .position(|f| f.id == file_id)
            .ok_or_else(|| {
                OrcsError::Io(format!("File with ID '{}' not found in workspace", file_id))
            })?;

        // Get the file to delete
        let file = &workspace.resources.uploaded_files[file_index];

        // Delete the physical file
        if file.path.exists() {
            fs::remove_file(&file.path).await.map_err(|e| {
                OrcsError::Io(format!(
                    "Failed to delete file '{}': {}",
                    file.path.display(),
                    e
                ))
            })?;
        }

        // Remove from workspace's uploaded_files list
        workspace.resources.uploaded_files.remove(file_index);

        // Save the updated workspace metadata
        self.save_workspace(&workspace).await?;

        Ok(())
    }

    async fn rename_file_in_workspace(
        &self,
        workspace_id: &str,
        file_id: &str,
        new_name: &str,
    ) -> Result<UploadedFile> {
        // Load existing workspace metadata
        let mut workspace = self.load_workspace(workspace_id).await?;

        // Find the file in the uploaded_files list
        let file_index = workspace
            .resources
            .uploaded_files
            .iter()
            .position(|f| f.id == file_id)
            .ok_or_else(|| {
                OrcsError::Io(format!("File with ID '{}' not found in workspace", file_id))
            })?;

        // Get the current file
        let old_file = &workspace.resources.uploaded_files[file_index];
        let old_path = old_file.path.clone();

        // Construct the new path
        let new_path = old_path
            .parent()
            .ok_or_else(|| OrcsError::Io("Invalid file path".to_string()))?
            .join(new_name);

        // Check if a file with the new name already exists
        if new_path.exists() {
            return Err(OrcsError::Io(format!(
                "A file with name '{}' already exists",
                new_name
            )));
        }

        // Rename the physical file
        fs::rename(&old_path, &new_path).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to rename file from '{}' to '{}': {}",
                old_path.display(),
                new_path.display(),
                e
            ))
        })?;

        // Update MIME type based on new extension
        let mime_type = infer_mime_type(new_name);

        // Update the file metadata
        workspace.resources.uploaded_files[file_index].name = new_name.to_string();
        workspace.resources.uploaded_files[file_index].path = new_path;
        workspace.resources.uploaded_files[file_index].mime_type = mime_type;

        // Clone the updated file for return
        let updated_file = workspace.resources.uploaded_files[file_index].clone();

        // Save the updated workspace metadata
        self.save_workspace(&workspace).await?;

        Ok(updated_file)
    }

    async fn create_temp_file(
        &self,
        session_id: &str,
        workspace_id: &str,
        filename: &str,
        content: &[u8],
    ) -> Result<TempFile> {
        // Load existing workspace metadata
        let mut workspace = self.load_workspace(workspace_id).await?;

        // Get the workspace directory
        let workspace_dir = self.get_workspace_dir(workspace_id);

        // Construct the path to session's temp directory (sessions/{session_id}/temp)
        let session_temp_dir = workspace_dir.join("sessions").join(session_id).join("temp");

        // Ensure the temp directory exists
        fs::create_dir_all(&session_temp_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create session temp directory '{}': {}",
                session_temp_dir.display(),
                e
            ))
        })?;

        // Generate a unique ID for the temp file
        let file_id = Uuid::new_v4().to_string();

        // Construct the destination path
        let dest_path = session_temp_dir.join(filename);

        // Write the content to the file
        fs::write(&dest_path, content).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to write temp file to '{}': {}",
                dest_path.display(),
                e
            ))
        })?;

        // Get current timestamp
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OrcsError::Io(format!("Failed to get current timestamp: {}", e)))?
            .as_secs() as i64;

        // Create the TempFile domain model
        let temp_file = TempFile {
            id: file_id,
            path: dest_path,
            purpose: format!("Temporary file for session {}", session_id),
            created_at,
            auto_delete: true,
        };

        // Add to workspace's temp_files list
        workspace.resources.temp_files.push(temp_file.clone());

        // Save the updated workspace metadata
        self.save_workspace(&workspace).await?;

        Ok(temp_file)
    }

    async fn read_file_content(&self, workspace_id: &str, relative_path: &str) -> Result<String> {
        // Get the workspace directory
        let workspace_dir = self.get_workspace_dir(workspace_id);

        // Canonicalize the workspace directory to get the absolute path
        let canonical_workspace_dir = workspace_dir.canonicalize().map_err(|e| {
            OrcsError::Io(format!(
                "Failed to canonicalize workspace directory '{}': {}",
                workspace_dir.display(),
                e
            ))
        })?;

        // Construct the target file path by joining workspace dir with relative path
        let target_path = workspace_dir.join(relative_path);

        // Check if the file exists before attempting to canonicalize
        if !target_path.exists() {
            return Err(OrcsError::Io(format!(
                "File not found at '{}': No such file or directory",
                relative_path
            )));
        }

        // Canonicalize the target path to resolve any '..' or symlinks
        let canonical_target_path = target_path.canonicalize().map_err(|e| {
            OrcsError::Io(format!(
                "Failed to access file at '{}': {}",
                relative_path, e
            ))
        })?;

        // Security check: Ensure the canonical target path starts with the canonical workspace directory
        if !canonical_target_path.starts_with(&canonical_workspace_dir) {
            return Err(OrcsError::Security(format!(
                "Path traversal attempt detected: '{}' is outside workspace directory",
                relative_path
            )));
        }

        // Read the file content
        let content = fs::read_to_string(&canonical_target_path)
            .await
            .map_err(|e| {
                OrcsError::Io(format!("Failed to read file at '{}': {}", relative_path, e))
            })?;

        Ok(content)
    }

    async fn get_session_workspace(&self, _session_id: &str) -> Result<Option<SessionWorkspace>> {
        unimplemented!("get_session_workspace will be implemented in a subsequent step")
    }

    async fn list_all_workspaces(&self) -> Result<Vec<Workspace>> {
        // Load all workspaces from repository (already sorted by last_accessed)
        let mut workspaces = self.workspace_repository.list_all().await?;

        // Set workspace_dir for each workspace
        for ws in &mut workspaces {
            ws.workspace_dir = self.get_workspace_dir(&ws.id);
        }

        Ok(workspaces)
    }

    async fn toggle_favorite(&self, workspace_id: &str) -> Result<()> {
        let mut workspace = self.load_workspace(workspace_id).await?;
        workspace.is_favorite = !workspace.is_favorite;
        self.save_workspace(&workspace).await
    }

    async fn touch_workspace(&self, workspace_id: &str) -> Result<()> {
        let mut workspace = self.load_workspace(workspace_id).await?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| OrcsError::Io(format!("Failed to get current timestamp: {}", e)))?
            .as_secs() as i64;
        workspace.last_accessed = now;
        self.save_workspace(&workspace).await
    }

    async fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        let workspace_dir = self.get_workspace_dir(&workspace.id);

        // Ensure workspace directory exists (for actual files)
        fs::create_dir_all(&workspace_dir).await.map_err(|e| {
            OrcsError::Io(format!(
                "Failed to create workspace directory '{}': {}",
                workspace_dir.display(),
                e
            ))
        })?;

        // Save via repository
        self.workspace_repository.save(workspace).await?;

        Ok(())
    }

    async fn delete_workspace(&self, workspace_id: &str) -> Result<()> {
        // Delete metadata via repository
        self.workspace_repository.delete(workspace_id).await?;

        // Delete workspace files directory
        let workspace_dir = self.get_workspace_dir(workspace_id);
        if workspace_dir.exists() {
            fs::remove_dir_all(&workspace_dir).await.map_err(|e| {
                OrcsError::Io(format!(
                    "Failed to delete workspace directory '{}': {}",
                    workspace_dir.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_new_creates_root_directory() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");

        assert!(!root_path.exists());

        let _manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        assert!(root_path.exists());
    }

    #[tokio::test]
    async fn test_get_workspace_id_is_deterministic() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        let id1 = FileSystemWorkspaceManager::get_workspace_id(repo_path).unwrap();
        let id2 = FileSystemWorkspaceManager::get_workspace_id(repo_path).unwrap();

        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_get_or_create_workspace_creates_new() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        assert_eq!(workspace.name, "test-repo");
        assert_eq!(workspace.root_path, repo_path);
        assert!(workspace.resources.uploaded_files.is_empty());
        assert!(workspace.resources.temp_files.is_empty());
    }

    #[tokio::test]
    async fn test_get_or_create_workspace_loads_existing() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace first time
        let workspace1 = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Load workspace second time
        let workspace2 = manager.get_or_create_workspace(&repo_path).await.unwrap();

        assert_eq!(workspace1.id, workspace2.id);
        assert_eq!(workspace1.name, workspace2.name);
    }

    #[tokio::test]
    async fn test_get_workspace_returns_none_if_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        let result = manager.get_workspace("nonexistent-id").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_workspace_returns_existing() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        let retrieved = manager.get_workspace(&workspace.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, workspace.id);
    }

    #[tokio::test]
    async fn test_add_file_to_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        // Create a test file to upload
        let test_file_path = temp_dir.path().join("test.txt");
        let test_content = b"Hello, workspace!";
        fs::write(&test_file_path, test_content).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace
        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Add file to workspace
        let uploaded_file = manager
            .add_file_to_workspace(&workspace.id, &test_file_path)
            .await
            .unwrap();

        // Verify the uploaded file metadata
        assert_eq!(uploaded_file.name, "test.txt");
        assert_eq!(uploaded_file.mime_type, "text/plain");
        assert_eq!(uploaded_file.size, test_content.len() as u64);

        // Verify the file was copied to the correct location
        assert!(uploaded_file.path.exists());
        let copied_content = fs::read(&uploaded_file.path).await.unwrap();
        assert_eq!(copied_content, test_content);

        // Verify the workspace metadata was updated
        let updated_workspace = manager.get_workspace(&workspace.id).await.unwrap().unwrap();
        assert_eq!(updated_workspace.resources.uploaded_files.len(), 1);
        assert_eq!(
            updated_workspace.resources.uploaded_files[0].id,
            uploaded_file.id
        );
        assert_eq!(
            updated_workspace.resources.uploaded_files[0].name,
            uploaded_file.name
        );
    }

    #[tokio::test]
    async fn test_create_temp_file() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace
        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Create a temp file
        let session_id = "test-session-123";
        let filename = "temp_output.txt";
        let content = b"This is temporary content";

        let temp_file = manager
            .create_temp_file(session_id, &workspace.id, filename, content)
            .await
            .unwrap();

        // Verify the temp file metadata
        assert!(temp_file.id.len() > 0);
        assert_eq!(temp_file.auto_delete, true);
        assert!(temp_file.created_at > 0);

        // Verify the file was created at the correct location
        assert!(temp_file.path.exists());
        let expected_path = root_path
            .join(&workspace.id)
            .join("sessions")
            .join(session_id)
            .join("temp")
            .join(filename);
        assert_eq!(temp_file.path, expected_path);

        // Verify the file content
        let file_content = fs::read(&temp_file.path).await.unwrap();
        assert_eq!(file_content, content);

        // Verify the workspace metadata was updated
        let updated_workspace = manager.get_workspace(&workspace.id).await.unwrap().unwrap();
        assert_eq!(updated_workspace.resources.temp_files.len(), 1);
        assert_eq!(updated_workspace.resources.temp_files[0].id, temp_file.id);
        assert_eq!(
            updated_workspace.resources.temp_files[0].path,
            temp_file.path
        );
    }

    #[tokio::test]
    async fn test_add_file_from_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace
        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Test data
        let filename = "test.txt";
        let test_content = b"Hello, workspace from bytes!";

        // Add file from bytes
        let uploaded_file = manager
            .add_file_from_bytes(&workspace.id, filename, test_content, None, None, None)
            .await
            .unwrap();

        // Verify the uploaded file metadata
        assert_eq!(uploaded_file.name, filename);
        assert_eq!(uploaded_file.mime_type, "text/plain");
        assert_eq!(uploaded_file.size, test_content.len() as u64);

        // Verify the file was written to the correct location
        assert!(uploaded_file.path.exists());
        let file_content = fs::read(&uploaded_file.path).await.unwrap();
        assert_eq!(file_content, test_content);

        // Verify the workspace metadata was updated
        let updated_workspace = manager.get_workspace(&workspace.id).await.unwrap().unwrap();
        assert_eq!(updated_workspace.resources.uploaded_files.len(), 1);
        assert_eq!(
            updated_workspace.resources.uploaded_files[0].id,
            uploaded_file.id
        );
        assert_eq!(
            updated_workspace.resources.uploaded_files[0].name,
            uploaded_file.name
        );
    }

    #[tokio::test]
    async fn test_read_file_content() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace
        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Create a test file in the workspace directory
        let workspace_dir = manager.get_workspace_dir(&workspace.id);
        let test_file_path = workspace_dir.join("test_file.txt");
        let test_content = "Hello from test file!";
        fs::write(&test_file_path, test_content).await.unwrap();

        // Read the file content using the read_file_content method
        let content = manager
            .read_file_content(&workspace.id, "test_file.txt")
            .await
            .unwrap();

        // Verify the content matches
        assert_eq!(content, test_content);
    }

    #[tokio::test]
    async fn test_read_file_content_blocks_path_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().join("workspaces");
        let repo_path = temp_dir.path().join("test-repo");
        fs::create_dir_all(&repo_path).await.unwrap();

        let manager = FileSystemWorkspaceManager::new(root_path.clone())
            .await
            .unwrap();

        // Create workspace
        let workspace = manager.get_or_create_workspace(&repo_path).await.unwrap();

        // Create a file outside the workspace directory
        let outside_file = temp_dir.path().join("outside.txt");
        fs::write(&outside_file, "This should not be accessible")
            .await
            .unwrap();

        // Get the workspace directory and create a symlink to the outside file
        let workspace_dir = manager.get_workspace_dir(&workspace.id);
        let symlink_path = workspace_dir.join("symlink_to_outside.txt");

        // Create a symlink pointing to the outside file
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_file, &symlink_path).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside_file, &symlink_path).unwrap();

        // Attempt to read the file through the symlink
        let result = manager
            .read_file_content(&workspace.id, "symlink_to_outside.txt")
            .await;

        // Verify that the operation fails with a Security error
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            OrcsError::Security(msg) => {
                assert!(msg.contains("Path traversal attempt detected"));
            }
            _ => panic!("Expected Security error, got: {:?}", error),
        }
    }
}
