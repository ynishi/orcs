//! Unified path management for orcs configuration files.
//!
//! All orcs configuration, secrets, and session data are managed via AppPaths
//! from the version-migrate crate for consistency across all storage.
//!
//! This ensures consistency across all platforms (Linux, macOS, Windows).
//!
//! # Architecture
//!
//! The path system is divided into two layers:
//!
//! ## Base Layer (Platform-dependent)
//! - `config_dir()`: Platform-specific config directory via AppPaths
//! - `data_dir()`: Platform-specific data directory via AppPaths
//! - `secret_dir()`: Currently same as config_dir, future Keychain migration
//!
//! ## Logical Layer (Application structure)
//! All application paths are built on top of the base layer:
//! - Config: `config_file()`
//! - Secret: `secret_file()`
//! - Data: `personas_dir()`, `content_dir()`, `workspaces_dir()`, etc.
//! - Logs: `logs_dir()`
//!
//! # Directory Structure
//!
//! ```text
//! config_dir() (via PrefPath)
//!   - macOS: ~/Library/Preferences/com.orcs-app/
//!   - Linux: ~/.config/com.orcs-app/
//!   - Windows: %APPDATA%\com.orcs-app\
//! ├── config.toml              # Application configuration
//! ├── secret.json              # API keys and secrets (future: OS Keychain)
//! └── logs/                    # Application logs
//!
//! data_dir() (via AppPaths)
//!   - macOS: ~/Library/Application Support/orcs/
//!   - Linux: ~/.local/share/orcs/
//!   - Windows: %LOCALAPPDATA%\orcs\
//! ├── personas/                # Persona definitions
//! └── content/                 # Application content
//!     ├── workspaces/          # Workspace data
//!     ├── sessions/            # Session data
//!     └── tasks/               # Task data
//! ```

use std::path::PathBuf;
use version_migrate::{AppPaths, PrefPath};

/// Errors that can occur during path resolution.
#[derive(Debug)]
pub enum PathError {
    /// Home directory could not be determined.
    HomeDirNotFound,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::HomeDirNotFound => write!(f, "Cannot find home directory"),
        }
    }
}

impl std::error::Error for PathError {}

/// Unified path management for orcs.
///
/// Provides a two-layer architecture:
/// - Base layer: Platform-dependent directories via AppPaths
/// - Logical layer: Application-specific paths built on base layer
pub struct OrcsPaths;

impl OrcsPaths {
    // ============================================
    // Base Layer (Platform-dependent via AppPaths)
    // ============================================

    /// Returns a configured AppPaths instance for orcs.
    ///
    /// This uses the default PathStrategy (XDG on Linux/macOS, appropriate on Windows).
    fn app_paths() -> AppPaths {
        AppPaths::new("orcs")
    }

    /// Returns a configured PrefPath instance for orcs.
    ///
    /// Uses OS-specific preference directories (e.g., ~/Library/Preferences on macOS).
    fn pref_path() -> PrefPath {
        PrefPath::new("com.orcs-app")
    }

    /// Returns the orcs configuration directory.
    ///
    /// Uses PrefPath to determine the OS-recommended preference directory.
    /// This is the base directory for all configuration-related files.
    ///
    /// # Platform Behavior
    ///
    /// - Linux: `~/.config/com.orcs-app/`
    /// - macOS: `~/Library/Preferences/com.orcs-app/`
    /// - Windows: `%APPDATA%\com.orcs-app\`
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to config directory
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine directory
    pub fn config_dir() -> Result<PathBuf, PathError> {
        Self::pref_path()
            .pref_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the orcs data directory.
    ///
    /// Uses AppPaths to determine the correct data directory for the platform.
    /// This is the base directory for all application data.
    ///
    /// # Platform Behavior
    ///
    /// - Linux: `~/.local/share/orcs/`
    /// - macOS: `~/Library/Application Support/orcs/`
    /// - Windows: `%LOCALAPPDATA%\orcs\`
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to data directory
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine directory
    pub fn data_dir() -> Result<PathBuf, PathError> {
        Self::app_paths()
            .data_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the orcs secret directory.
    ///
    /// Currently returns the same as config_dir().
    /// Future versions may migrate to platform-specific secure storage (e.g., macOS Keychain).
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to secret directory
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine directory
    pub fn secret_dir() -> Result<PathBuf, PathError> {
        // TODO: Migrate to platform-specific secure storage
        Self::config_dir()
    }

    // ============================================
    // Logical Layer (Application structure)
    // ============================================

    // (1) Config
    // ----------------------------------------

    /// Returns the path to the main configuration file.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to config.toml
    /// - `Err(PathError)`: Could not determine path
    pub fn config_file() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Returns the path to the application state file.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to state.toml
    /// - `Err(PathError)`: Could not determine path
    pub fn state_file() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("state.toml"))
    }


    // (2) Secret
    // ----------------------------------------

    /// Returns the path to the secrets file.
    ///
    /// # Security Note
    ///
    /// Ensure this file has appropriate permissions (e.g., 600) to prevent
    /// unauthorized access.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to secret.json
    /// - `Err(PathError)`: Could not determine path
    pub fn secret_file() -> Result<PathBuf, PathError> {
        Ok(Self::secret_dir()?.join("secret.json"))
    }

    /// Ensures the secret file exists, creating it with a template if it doesn't.
    ///
    /// Creates a secret.json file with a properly typed template structure if the file doesn't exist.
    /// The template includes placeholders for common API keys using the SecretConfig type.
    ///
    /// # Security Note
    ///
    /// This function sets file permissions to 600 (user read/write only) on Unix systems.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to the secret file (existing or newly created)
    /// - `Err(std::io::Error)`: If file creation or permission setting fails
    pub fn ensure_secret_file() -> Result<PathBuf, std::io::Error> {
        let secret_path = Self::secret_file()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;

        // If file already exists, return the path
        if secret_path.exists() {
            return Ok(secret_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = secret_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create typed template using SecretConfig
        use orcs_core::config::{ClaudeConfig, GeminiConfig, OpenAIConfig, SecretConfig};

        let template_config = SecretConfig {
            claude: Some(ClaudeConfig {
                api_key: String::new(),
                model_name: Some("claude-sonnet-4-20250514".to_string()),
            }),
            gemini: Some(GeminiConfig {
                api_key: String::new(),
                model_name: Some("gemini-2.5-flash".to_string()),
            }),
            openai: Some(OpenAIConfig {
                api_key: String::new(),
                model_name: Some("gpt-4o".to_string()),
            }),
        };

        // Serialize to pretty JSON
        let template_json = serde_json::to_string_pretty(&template_config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Write template to file
        std::fs::write(&secret_path, template_json)?;

        // Set file permissions to 600 (user read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&secret_path, permissions)?;
        }

        Ok(secret_path)
    }

    // (3) Data/Content
    // ----------------------------------------

    /// Returns the path to the personas directory.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to personas directory
    /// - `Err(PathError)`: Could not determine path
    pub fn personas_dir() -> Result<PathBuf, PathError> {
        Ok(Self::data_dir()?.join("personas"))
    }

    /// Returns the path to the content directory.
    ///
    /// This is the base directory for all application content
    /// (workspaces, sessions, tasks, etc.).
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to content directory
    /// - `Err(PathError)`: Could not determine path
    pub fn content_dir() -> Result<PathBuf, PathError> {
        Ok(Self::data_dir()?.join("content"))
    }

    /// Returns the path to the workspaces directory.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to workspaces directory
    /// - `Err(PathError)`: Could not determine path
    pub fn workspaces_dir() -> Result<PathBuf, PathError> {
        Ok(Self::content_dir()?.join("workspaces"))
    }

    /// Returns the path to the sessions directory.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to sessions directory
    /// - `Err(PathError)`: Could not determine path
    pub fn sessions_dir() -> Result<PathBuf, PathError> {
        Ok(Self::content_dir()?.join("sessions"))
    }

    /// Returns the path to the tasks directory.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to tasks directory
    /// - `Err(PathError)`: Could not determine path
    pub fn tasks_dir() -> Result<PathBuf, PathError> {
        Ok(Self::content_dir()?.join("tasks"))
    }

    // (4) Logs
    // ----------------------------------------

    /// Returns the path to the logs directory.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to logs directory
    /// - `Err(PathError)`: Could not determine path
    pub fn logs_dir() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("logs"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // Base Layer Tests
    // ============================================

    #[test]
    fn test_config_dir() {
        let config_dir = OrcsPaths::config_dir().unwrap();
        // PrefPath returns platform-specific preference directory
        assert!(config_dir.to_string_lossy().contains("com.orcs-app"));

        #[cfg(target_os = "macos")]
        {
            // macOS: ~/Library/Preferences/com.orcs-app
            assert!(config_dir.to_string_lossy().contains("Library/Preferences"));
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Linux: ~/.config/com.orcs-app
            assert!(config_dir.to_string_lossy().contains(".config"));
        }
    }

    #[test]
    fn test_data_dir() {
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(data_dir.ends_with("orcs"));
    }

    #[test]
    fn test_secret_dir() {
        let secret_dir = OrcsPaths::secret_dir().unwrap();
        // Currently same as config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert_eq!(secret_dir, config_dir);
    }

    // ============================================
    // Logical Layer Tests
    // ============================================

    // (1) Config
    // ----------------------------------------

    #[test]
    fn test_config_file() {
        let config_file = OrcsPaths::config_file().unwrap();
        assert!(config_file.ends_with("config.toml"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(config_file.starts_with(&config_dir));
    }

    // (2) Secret
    // ----------------------------------------

    #[test]
    fn test_secret_file() {
        let secret_file = OrcsPaths::secret_file().unwrap();
        assert!(secret_file.ends_with("secret.json"));
        // Verify it's under secret_dir
        let secret_dir = OrcsPaths::secret_dir().unwrap();
        assert!(secret_file.starts_with(&secret_dir));
    }

    // (3) Data/Content
    // ----------------------------------------

    #[test]
    fn test_personas_dir() {
        let personas_dir = OrcsPaths::personas_dir().unwrap();
        assert!(personas_dir.ends_with("personas"));
        // Verify it's under data_dir
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(personas_dir.starts_with(&data_dir));
    }

    #[test]
    fn test_content_dir() {
        let content_dir = OrcsPaths::content_dir().unwrap();
        assert!(content_dir.ends_with("content"));
        // Verify it's under data_dir
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(content_dir.starts_with(&data_dir));
    }

    #[test]
    fn test_workspaces_dir() {
        let workspaces_dir = OrcsPaths::workspaces_dir().unwrap();
        assert!(workspaces_dir.ends_with("workspaces"));
        // Verify it's under content_dir
        let content_dir = OrcsPaths::content_dir().unwrap();
        assert!(workspaces_dir.starts_with(&content_dir));
    }

    #[test]
    fn test_sessions_dir() {
        let sessions_dir = OrcsPaths::sessions_dir().unwrap();
        assert!(sessions_dir.ends_with("sessions"));
        // Verify it's under content_dir
        let content_dir = OrcsPaths::content_dir().unwrap();
        assert!(sessions_dir.starts_with(&content_dir));
    }

    #[test]
    fn test_tasks_dir() {
        let tasks_dir = OrcsPaths::tasks_dir().unwrap();
        assert!(tasks_dir.ends_with("tasks"));
        // Verify it's under content_dir
        let content_dir = OrcsPaths::content_dir().unwrap();
        assert!(tasks_dir.starts_with(&content_dir));
    }

    // (4) Logs
    // ----------------------------------------

    #[test]
    fn test_logs_dir() {
        let logs_dir = OrcsPaths::logs_dir().unwrap();
        assert!(logs_dir.ends_with("logs"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(logs_dir.starts_with(&config_dir));
    }

    // ============================================
    // Integration Tests
    // ============================================

    #[test]
    fn test_path_hierarchy() {
        let data_dir = OrcsPaths::data_dir().unwrap();
        let content_dir = OrcsPaths::content_dir().unwrap();
        let workspaces_dir = OrcsPaths::workspaces_dir().unwrap();
        let sessions_dir = OrcsPaths::sessions_dir().unwrap();
        let tasks_dir = OrcsPaths::tasks_dir().unwrap();

        // Verify hierarchy: data_dir > content_dir > {workspaces,sessions,tasks}
        assert!(content_dir.starts_with(&data_dir));
        assert!(workspaces_dir.starts_with(&content_dir));
        assert!(sessions_dir.starts_with(&content_dir));
        assert!(tasks_dir.starts_with(&content_dir));
    }
}
