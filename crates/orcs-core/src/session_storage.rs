use anyhow::{Context, Result};
use orcs_types::SessionData;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages session persistence to the filesystem.
///
/// `SessionStorage` handles reading and writing session data to JSON files
/// in a structured directory layout. It supports CRUD operations for sessions
/// and tracking the currently active session.
pub struct SessionStorage {
    base_dir: PathBuf,
}

impl SessionStorage {
    /// Creates a new `SessionStorage` instance with the specified base directory.
    ///
    /// The directory structure will be created if it doesn't exist:
    /// ```text
    /// base_dir/
    /// ├── sessions/
    /// │   ├── session-id-1.json
    /// │   └── session-id-2.json
    /// └── active_session.txt
    /// ```
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory for storing session data
    ///
    /// # Errors
    ///
    /// Returns an error if the directory structure cannot be created.
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();

        // Create directory structure
        let sessions_dir = base_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)
            .context("Failed to create sessions directory")?;

        Ok(Self { base_dir })
    }

    /// Creates a `SessionStorage` instance at the default location (~/.orcs).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined or if
    /// the directory structure cannot be created.
    pub fn default_location() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .context("Failed to get home directory")?;
        let base_dir = home_dir.join(".orcs");
        Self::new(base_dir)
    }

    /// Saves a session to disk.
    ///
    /// The session is serialized to JSON and written to a file named
    /// `<session_id>.json` in the sessions directory.
    ///
    /// # Arguments
    ///
    /// * `session` - The session data to save
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails or if the file cannot be written.
    pub fn save_session(&self, session: &SessionData) -> Result<()> {
        let file_path = self.session_file_path(&session.id);
        let json = serde_json::to_string_pretty(session)
            .context("Failed to serialize session data")?;

        fs::write(&file_path, json)
            .context(format!("Failed to write session file: {:?}", file_path))?;

        Ok(())
    }

    /// Loads a session from disk by its ID.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to load
    ///
    /// # Returns
    ///
    /// The loaded session data.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist, cannot be read, or
    /// contains invalid JSON.
    pub fn load_session(&self, session_id: &str) -> Result<SessionData> {
        let file_path = self.session_file_path(session_id);
        let json = fs::read_to_string(&file_path)
            .context(format!("Failed to read session file: {:?}", file_path))?;

        let session: SessionData = serde_json::from_str(&json)
            .context("Failed to deserialize session data")?;

        Ok(session)
    }

    /// Lists all sessions stored on disk.
    ///
    /// Sessions are returned sorted by update time (most recent first).
    ///
    /// # Returns
    ///
    /// A vector of all stored session data.
    ///
    /// # Errors
    ///
    /// Returns an error if the sessions directory cannot be read.
    pub fn list_sessions(&self) -> Result<Vec<SessionData>> {
        let sessions_dir = self.base_dir.join("sessions");
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&sessions_dir)
            .context("Failed to read sessions directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session) = self.load_session_from_path(&path) {
                    sessions.push(session);
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// Deletes a session from disk.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let file_path = self.session_file_path(session_id);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .context(format!("Failed to delete session file: {:?}", file_path))?;
        }

        Ok(())
    }

    /// Saves the ID of the currently active session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the active session
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save_active_session_id(&self, session_id: &str) -> Result<()> {
        let active_file = self.base_dir.join("active_session.txt");
        fs::write(&active_file, session_id)
            .context("Failed to write active session ID")?;
        Ok(())
    }

    /// Loads the ID of the currently active session.
    ///
    /// # Returns
    ///
    /// `Some(session_id)` if an active session is set, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read.
    pub fn load_active_session_id(&self) -> Result<Option<String>> {
        let active_file = self.base_dir.join("active_session.txt");

        if !active_file.exists() {
            return Ok(None);
        }

        let session_id = fs::read_to_string(&active_file)
            .context("Failed to read active session ID")?;

        Ok(Some(session_id.trim().to_string()))
    }

    /// Returns the file path for a given session ID.
    fn session_file_path(&self, session_id: &str) -> PathBuf {
        self.base_dir
            .join("sessions")
            .join(format!("{}.json", session_id))
    }

    /// Loads a session from a specific file path.
    fn load_session_from_path(&self, path: &Path) -> Result<SessionData> {
        let json = fs::read_to_string(path)
            .context(format!("Failed to read session file: {:?}", path))?;

        let session: SessionData = serde_json::from_str(&json)
            .context("Failed to deserialize session data")?;

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orcs_types::{AppMode, MessageRole, ConversationMessage};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_session(id: &str) -> SessionData {
        let mut persona_histories = HashMap::new();
        persona_histories.insert(
            "mai".to_string(),
            vec![
                ConversationMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                    timestamp: "2024-01-01T00:00:00Z".to_string(),
                },
                ConversationMessage {
                    role: MessageRole::Assistant,
                    content: "Hi there!".to_string(),
                    timestamp: "2024-01-01T00:00:01Z".to_string(),
                },
            ],
        );

        SessionData {
            id: id.to_string(),
            name: format!("Test Session {}", id),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            current_persona_id: "mai".to_string(),
            persona_histories,
            app_mode: AppMode::Idle,
        }
    }

    #[test]
    fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();

        let session = create_test_session("test-session-1");

        // Save
        storage.save_session(&session).unwrap();

        // Load
        let loaded = storage.load_session("test-session-1").unwrap();

        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.name, session.name);
        assert_eq!(loaded.current_persona_id, session.current_persona_id);
    }

    #[test]
    fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();

        // Save multiple sessions
        storage.save_session(&create_test_session("session-1")).unwrap();
        storage.save_session(&create_test_session("session-2")).unwrap();
        storage.save_session(&create_test_session("session-3")).unwrap();

        // List
        let sessions = storage.list_sessions().unwrap();

        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_delete_session() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();

        let session = create_test_session("session-to-delete");
        storage.save_session(&session).unwrap();

        // Before delete
        assert!(storage.load_session("session-to-delete").is_ok());

        // Delete
        storage.delete_session("session-to-delete").unwrap();

        // After delete
        assert!(storage.load_session("session-to-delete").is_err());
    }

    #[test]
    fn test_active_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SessionStorage::new(temp_dir.path()).unwrap();

        // Initial state
        assert_eq!(storage.load_active_session_id().unwrap(), None);

        // Save
        storage.save_active_session_id("active-session").unwrap();

        // Load
        assert_eq!(
            storage.load_active_session_id().unwrap(),
            Some("active-session".to_string())
        );
    }
}
