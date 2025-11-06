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

use std::path::{Path, PathBuf};
use serde::{Serialize, ser};
use version_migrate::{
    AppPaths, AsyncDirStorage, DirStorageStrategy, FilenameEncoding, FormatStrategy, Migrator,
    PathStrategy, PrefPath,
};

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
/// Supports both default paths and custom base paths for testing.
/// Create an instance with `OrcsPaths::new(None)` for default behavior,
/// or `OrcsPaths::new(Some(custom_path))` for testing.
pub struct OrcsPaths {
    /// Optional custom base path (for testing). None uses default platform paths.
    base_path: Option<PathBuf>,
}

impl OrcsPaths {
    /// Creates a new OrcsPaths instance.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Optional custom base path for testing. None uses default platform paths.
    pub fn new(base_path: Option<&Path>) -> Self {
        Self { base_path: base_path.map(|p|p.to_path_buf()) }
    }

    // ============================================
    // Base Layer (Platform-dependent via AppPaths) - PRIVATE
    // ============================================

    /// Returns a configured AppPaths instance for orcs.
    fn app_paths(&self) -> AppPaths {
        if let Some(ref base) = self.base_path {
            AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(base.clone()))
        } else {
            AppPaths::new("orcs")
        }
    }

    /// Returns a configured PrefPath instance for orcs.
    fn pref_path(&self) -> PrefPath {
        PrefPath::new("com.orcs-app")
    }

    /// Returns the orcs configuration directory.
    fn config_dir(&self) -> Result<PathBuf, PathError> {
        if let Some(ref base) = self.base_path {
            Ok(base.join("config"))
        } else {
            self.pref_path()
                .pref_dir()
                .map_err(|_| PathError::HomeDirNotFound)
        }
    }

    /// Returns the orcs data directory.
    fn data_dir(&self) -> Result<PathBuf, PathError> {
        if let Some(ref base) = self.base_path {
            Ok(base.join("data"))
        } else {
            self.app_paths()
                .data_dir()
                .map_err(|_| PathError::HomeDirNotFound)
        }
    }

    /// Returns the orcs secret directory.
    fn secret_dir(&self) -> Result<PathBuf, PathError> {
        // TODO: Migrate to platform-specific secure storage
        self.config_dir()
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
    pub fn get_path(self, service_type: ServiceType) -> Result<PathType, PathError> {
        if self.base_path.is_some() {
            if self.base_path.as_ref().unwrap().is_file() {
                return Ok(PathType::File(self.base_path.unwrap()));
            } else {
                return Ok(PathType::Dir(self.base_path.unwrap()));
            }
        }
        match service_type {
            // Single-file services (return File path)
            ServiceType::AppState => {
                Ok(PathType::File(self.config_dir()?.join("app_state.toml")))
            }
            ServiceType::Config => Ok(PathType::File(self.config_dir()?.join("config.toml"))),
            ServiceType::Secret => Ok(PathType::File(self.secret_dir()?.join("secret.json"))),

            // Multi-file services (return Dir for storage management)
            ServiceType::Session => {
                Ok(PathType::Dir(self.content_dir()?.join("sessions")))
            }
            ServiceType::Workspace => {
                Ok(PathType::Dir(self.content_dir()?.join("workspaces")))
            }
            ServiceType::Task => Ok(PathType::Dir(self.content_dir()?.join("tasks"))),
            ServiceType::Persona => Ok(PathType::Dir(self.data_dir()?.join("personas"))),
            ServiceType::SlashCommand => {
                Ok(PathType::Dir(self.config_dir()?.join("slash_commands")))
            }
            ServiceType::Logs => Ok(PathType::Dir(self.config_dir()?.join("logs"))),
        }
    }

    /// Creates an AsyncDirStorage instance for a given service type.
    ///
    /// This is a helper method for repositories to create storage with proper configuration.
    /// It handles:
    /// - Path resolution via ServiceType
    /// - AppPaths setup with CustomBase strategy
    /// - Directory creation
    /// - Default storage strategy (TOML, Direct encoding)
    ///
    /// # Arguments
    ///
    /// * `service_type` - The type of service
    /// * `migrator` - Migrator instance (injected by repository)
    ///
    /// # Returns
    ///
    /// * `Ok(AsyncDirStorage)`: Configured storage instance
    /// * `Err(String)`: Failed to create storage
    ///
    /// # Example
    ///
    /// ```rust
    /// let migrator = create_persona_migrator();
    /// let storage = OrcsPaths::create_async_dir_storage(
    ///     ServiceType::Persona,
    ///     migrator
    /// ).await?;
    /// ```
    pub async fn create_async_dir_storage(
        self,
        service_type: ServiceType,
        migrator: Migrator,
    ) -> Result<AsyncDirStorage, String> {
        // Get directory path and extract parent + entity_name
        let path_type = self.get_path(service_type).map_err(|e| e.to_string())?;
        let full_dir = path_type.into_path_buf();

        // Extract parent directory and entity name
        let entity_name = full_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid directory name")?
            .to_string();

        let parent_dir = full_dir
            .parent()
            .ok_or("No parent directory")?
            .to_path_buf();

        // Ensure directory exists
        tokio::fs::create_dir_all(&full_dir)
            .await
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        // Setup AppPaths with CustomBase strategy
        let paths = AppPaths::new("orcs").data_strategy(PathStrategy::CustomBase(parent_dir));

        // Setup default storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        AsyncDirStorage::new(paths, &entity_name, migrator, strategy)
            .await
            .map_err(|e| format!("Failed to create AsyncDirStorage: {}", e))
    }

    // ============================================
    // Logical Layer (Private helpers)
    // ============================================

    /// Returns the path to the content directory.
    ///
    /// This is the base directory for all application content
    /// (workspaces, sessions, tasks, etc.).
    ///
    /// **PRIVATE**: Used internally by get_path() for content-based services.
    fn content_dir(&self) -> Result<PathBuf, PathError> {
        Ok(self.data_dir()?.join("content"))
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
