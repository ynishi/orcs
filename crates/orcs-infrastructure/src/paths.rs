//! Unified path management for orcs configuration files.
//!
//! All orcs configuration, secrets, and session data are managed via AppPaths
//! from the version-migrate crate for consistency across all storage.
//!
//! This ensures consistency across all platforms (Linux, macOS, Windows).

use std::path::PathBuf;
use version_migrate::AppPaths;

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
/// All paths are resolved via AppPaths from version-migrate crate.
/// This ensures consistency with AsyncDirStorage and other storage mechanisms.
///
/// # Directory Structure
///
/// ```text
/// ~/.config/orcs/              # Config directory (AppPaths default)
/// ├── config.toml              # Application configuration
/// ├── secret.json              # API keys and secrets
/// ├── sessions/                # Session files (AsyncDirStorage)
/// ├── workspaces/              # Workspace metadata (AsyncDirStorage)
/// ├── workspace_data/          # Full workspace data (AsyncDirStorage)
/// └── logs/                    # Application logs
///     └── orcs-desktop.log.YYYY-MM-DD
///
/// ~/.local/share/orcs/         # Data directory (for large files)
/// └── workspaces/              # Actual workspace files
/// ```
pub struct OrcsPaths;

impl OrcsPaths {
    /// Returns a configured AppPaths instance for orcs.
    ///
    /// This uses the default PathStrategy (XDG on Linux/macOS, appropriate on Windows).
    fn app_paths() -> AppPaths {
        AppPaths::new("orcs")
    }

    /// Returns the orcs configuration directory.
    ///
    /// Uses AppPaths to determine the correct config directory for the platform.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to config directory (e.g., `~/.config/orcs/`)
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine directory
    pub fn config_dir() -> Result<PathBuf, PathError> {
        Self::app_paths()
            .config_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the orcs data directory.
    ///
    /// Uses AppPaths to determine the correct data directory for the platform.
    /// This is typically used for larger files (workspace files, etc.).
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to data directory (e.g., `~/.local/share/orcs/`)
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine directory
    pub fn data_dir() -> Result<PathBuf, PathError> {
        Self::app_paths()
            .data_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the path to the main configuration file.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to config.toml
    /// - `Err(PathError)`: Could not determine path
    pub fn config_file() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

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
        Ok(Self::config_dir()?.join("secret.json"))
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
        use orcs_core::config::{SecretConfig, ClaudeConfig, GeminiConfig, OpenAIConfig};

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

    /// Returns the path to the sessions directory.
    ///
    /// Note: This is primarily for compatibility. New code should use
    /// AsyncDirSessionRepository which manages this via AsyncDirStorage.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to sessions directory
    /// - `Err(PathError)`: Could not determine path
    pub fn sessions_dir() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("sessions"))
    }

    /// Returns the path to the workspaces directory (for actual files).
    ///
    /// This is where FileSystemWorkspaceManager stores actual workspace files.
    /// Uses data_dir for larger files.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to workspaces directory
    /// - `Err(PathError)`: Could not determine path
    pub fn workspaces_dir() -> Result<PathBuf, PathError> {
        Ok(Self::data_dir()?.join("workspaces"))
    }

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

    #[test]
    fn test_config_dir() {
        let config_dir = OrcsPaths::config_dir().unwrap();
        // AppPaths returns platform-specific config directory with "orcs" appended
        assert!(config_dir.ends_with("orcs"));
    }

    #[test]
    fn test_config_file() {
        let config_file = OrcsPaths::config_file().unwrap();
        assert!(config_file.ends_with("config.toml"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(config_file.starts_with(&config_dir));
    }

    #[test]
    fn test_secret_file() {
        let secret_file = OrcsPaths::secret_file().unwrap();
        assert!(secret_file.ends_with("secret.json"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(secret_file.starts_with(&config_dir));
    }

    #[test]
    fn test_sessions_dir() {
        let sessions_dir = OrcsPaths::sessions_dir().unwrap();
        assert!(sessions_dir.ends_with("sessions"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(sessions_dir.starts_with(&config_dir));
    }

    #[test]
    fn test_data_dir() {
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(data_dir.ends_with("orcs"));
    }

    #[test]
    fn test_workspaces_dir() {
        let workspaces_dir = OrcsPaths::workspaces_dir().unwrap();
        assert!(workspaces_dir.ends_with("workspaces"));
        // Verify it's under data_dir
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(workspaces_dir.starts_with(&data_dir));
    }

    #[test]
    fn test_logs_dir() {
        let logs_dir = OrcsPaths::logs_dir().unwrap();
        assert!(logs_dir.ends_with("logs"));
        // Verify it's under config_dir
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(logs_dir.starts_with(&config_dir));
    }
}
