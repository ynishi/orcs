//! Unified path management for orcs configuration files.
//!
//! All orcs configuration, secrets, and session data are managed via AppPaths
//! from the version-migrate crate for consistency across all storage.
//!
//! This ensures consistency across all platforms (Linux, macOS, Windows).
//!
//! # Architecture
//!
//! The path system is divided into three layers:
//!
//! ## 1. Base Layer (Platform-dependent) - PRIVATE
//! Internal functions that provide platform-specific directories:
//! - `config_dir()`: Platform-specific config directory via PrefPath
//! - `data_dir()`: Platform-specific data directory via AppPaths
//! - `secret_dir()`: Currently same as config_dir, future Keychain migration
//!
//! **These are private and should not be called directly by services.**
//!
//! ## 2. Service Layer (Centralized Path Management) - PUBLIC API
//! Services should use `get_path(ServiceType)` to obtain their base directory:
//! ```rust
//! use orcs_infrastructure::paths::{OrcsPaths, ServiceType};
//!
//! // In AppStateService
//! let base_path = OrcsPaths::get_path(ServiceType::AppState)?;
//! ```
//!
//! This centralizes path resolution logic and makes it easy to change paths
//! for specific services without modifying service code.
//!
//! ## 3. Logical Layer (Application structure)
//! All paths are now accessed via `get_path(ServiceType)`:
//! - Config: `get_path(ServiceType::Config)` → File
//! - Secret: `get_path(ServiceType::Secret)` → File
//! - Session: `get_path(ServiceType::Session)` → Dir
//! - Workspace: `get_path(ServiceType::Workspace)` → Dir
//! - Logs: `get_path(ServiceType::Logs)` → Dir
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
//!
//! # Usage Guidelines for Services
//!
//! When implementing a new service that needs file storage:
//!
//! 1. Add a new variant to `ServiceType` enum
//! 2. Update `get_path()` to handle the new service type
//! 3. Use `OrcsPaths::get_path(ServiceType::YourService)` in your service
//!
//! Example:
//! ```rust
//! // In your service implementation
//! pub async fn new() -> Result<Self, String> {
//!     let base_path = OrcsPaths::get_path(ServiceType::YourService)
//!         .map_err(|e| e.to_string())?;
//!     // ... setup storage with base_path
//! }
//! ```

use std::path::PathBuf;
use version_migrate::{AppPaths, PrefPath};

/// Represents whether a path points to a file or directory.
///
/// This distinction is important because:
/// - **Dir**: Used by services that manage multiple files (e.g., Session, Workspace)
/// - **File**: Used by services that manage a single file (e.g., AppState -> app_state.toml)
///
/// The actual file/directory may or may not exist on the filesystem.
/// This enum describes the *intended* usage, not the current state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathType {
    /// Path represents a file
    File(PathBuf),
    /// Path represents a directory
    Dir(PathBuf),
}

impl PathType {
    /// Returns the underlying PathBuf regardless of type.
    pub fn as_path_buf(&self) -> &PathBuf {
        match self {
            PathType::File(p) | PathType::Dir(p) => p,
        }
    }

    /// Consumes self and returns the underlying PathBuf.
    pub fn into_path_buf(self) -> PathBuf {
        match self {
            PathType::File(p) | PathType::Dir(p) => p,
        }
    }

    /// Returns true if this is a File path.
    pub fn is_file(&self) -> bool {
        matches!(self, PathType::File(_))
    }

    /// Returns true if this is a Dir path.
    pub fn is_dir(&self) -> bool {
        matches!(self, PathType::Dir(_))
    }
}

/// Service types that require path resolution.
///
/// Each service type maps to a specific directory where that service
/// stores its data. This centralized enum makes it easy to manage
/// and modify paths for different services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    /// Application state service (state.toml)
    AppState,
    /// Configuration service (config.toml)
    Config,
    /// Secret configuration file (secret.json)
    Secret,
    /// Session service (sessions/)
    Session,
    /// Workspace service (workspaces/)
    Workspace,
    /// Task service (tasks/)
    Task,
    /// Persona service (personas/)
    Persona,
    /// Slash command service (slash_commands/)
    SlashCommand,
    /// Workspace metadata service (workspace_metadata/)
    WorkspaceMetadata,
    /// Logs directory (logs/)
    Logs,
}

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
    // Base Layer (Platform-dependent via AppPaths) - PRIVATE
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
    /// **PRIVATE**: Services should use `get_path(ServiceType)` instead.
    ///
    /// # Platform Behavior
    ///
    /// - Linux: `~/.config/com.orcs-app/`
    /// - macOS: `~/Library/Preferences/com.orcs-app/`
    /// - Windows: `%APPDATA%\com.orcs-app\`
    fn config_dir() -> Result<PathBuf, PathError> {
        Self::pref_path()
            .pref_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the orcs data directory.
    ///
    /// Uses AppPaths to determine the correct data directory for the platform.
    /// This is the base directory for all application data.
    ///
    /// **PRIVATE**: Services should use `get_path(ServiceType)` instead.
    ///
    /// # Platform Behavior
    ///
    /// - Linux: `~/.local/share/orcs/`
    /// - macOS: `~/Library/Application Support/orcs/`
    /// - Windows: `%LOCALAPPDATA%\orcs\`
    fn data_dir() -> Result<PathBuf, PathError> {
        Self::app_paths()
            .data_dir()
            .map_err(|_| PathError::HomeDirNotFound)
    }

    /// Returns the orcs secret directory.
    ///
    /// Currently returns the same as config_dir().
    /// Future versions may migrate to platform-specific secure storage (e.g., macOS Keychain).
    ///
    /// **PRIVATE**: Services should use `get_path(ServiceType)` instead.
    fn secret_dir() -> Result<PathBuf, PathError> {
        // TODO: Migrate to platform-specific secure storage
        Self::config_dir()
    }

    // ============================================
    // Service Layer (Centralized Path Management) - PUBLIC API
    // ============================================

    /// Gets the base path for a specific service type.
    ///
    /// This is the primary API for services to obtain their base path.
    /// It centralizes path resolution logic and makes it easy to change paths
    /// for specific services without modifying service code.
    ///
    /// Returns `PathType` which indicates whether the path is a file or directory:
    /// - **Dir**: Service manages multiple files (e.g., Session, Workspace)
    /// - **File**: Service uses a single file location (e.g., AppState)
    ///
    /// # Arguments
    ///
    /// * `service_type` - The type of service requesting a path
    ///
    /// # Returns
    ///
    /// * `Ok(PathType)`: Base path for the service (File or Dir)
    /// * `Err(PathError)`: Could not determine path
    ///
    /// # Example
    ///
    /// ```rust
    /// use orcs_infrastructure::paths::{OrcsPaths, ServiceType, PathType};
    ///
    /// let path_type = OrcsPaths::get_path(ServiceType::AppState)?;
    /// match path_type {
    ///     PathType::Dir(dir) => {
    ///         // Use dir for multi-file storage
    ///     }
    ///     PathType::File(file) => {
    ///         // Use file path directly
    ///     }
    /// }
    /// ```
    pub fn get_path(service_type: ServiceType) -> Result<PathType, PathError> {
        match service_type {
            // Single-file services (return File path)
            ServiceType::AppState => {
                Ok(PathType::File(Self::config_dir()?.join("app_state.toml")))
            }
            ServiceType::Config => Ok(PathType::File(Self::config_dir()?.join("config.toml"))),
            ServiceType::Secret => Ok(PathType::File(Self::secret_dir()?.join("secret.json"))),

            // Multi-file services (return Dir for storage management)
            ServiceType::Session => {
                Ok(PathType::Dir(Self::content_dir()?.join("sessions")))
            }
            ServiceType::Workspace => {
                Ok(PathType::Dir(Self::content_dir()?.join("workspaces")))
            }
            ServiceType::Task => Ok(PathType::Dir(Self::content_dir()?.join("tasks"))),
            ServiceType::Persona => Ok(PathType::Dir(Self::data_dir()?.join("personas"))),
            ServiceType::SlashCommand => {
                Ok(PathType::Dir(Self::config_dir()?.join("slash_commands")))
            }
            ServiceType::WorkspaceMetadata => {
                Ok(PathType::Dir(Self::config_dir()?.join("workspace_metadata")))
            }
            ServiceType::Logs => Ok(PathType::Dir(Self::config_dir()?.join("logs"))),
        }
    }

    // (2) Secret
    // ----------------------------------------
    // Secret file is now managed via ServiceType::Secret
    // Use OrcsPaths::get_path(ServiceType::Secret) instead

    /// Returns the path to the content directory.
    ///
    /// This is the base directory for all application content
    /// (workspaces, sessions, tasks, etc.).
    ///
    /// **PRIVATE**: Used internally by get_path() for content-based services.
    fn content_dir() -> Result<PathBuf, PathError> {
        Ok(Self::data_dir()?.join("content"))
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

    #[test]
    fn test_content_dir() {
        let content_dir = OrcsPaths::content_dir().unwrap();
        assert!(content_dir.ends_with("content"));
        // Verify it's under data_dir
        let data_dir = OrcsPaths::data_dir().unwrap();
        assert!(content_dir.starts_with(&data_dir));
    }
}
