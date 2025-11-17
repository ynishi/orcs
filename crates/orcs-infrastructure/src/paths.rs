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
//! ├── sessions/                # Session metadata (.toml)
//! ├── workspaces/              # Workspace metadata (.toml)
//! ├── tasks/                   # Task metadata (.toml)
//! ├── personas/                # Persona definitions (.toml)
//! └── files/                   # Storage (actual files)
//!     └── workspaces/          # Workspace resources (uploaded files, etc.)
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
    /// Workspace storage(storage/workspaces/)
    WorkspaceStorage,
    /// Task service (tasks/)
    Task,
    /// Persona service (personas/)
    Persona,
    /// Dialogue preset service (dialogue_presets/)
    DialoguePreset,
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

impl From<PathError> for orcs_core::OrcsError {
    fn from(err: PathError) -> Self {
        orcs_core::OrcsError::Config(err.to_string())
    }
}

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
    const APP_PATH: &str = "orcs";
    const DATA_PATH: &str = "data";

    /// Creates a new OrcsPaths instance.
    ///
    /// # Arguments
    ///
    /// * `base_path` - Optional custom base path for testing. None uses default platform paths.
    pub fn new(base_path: Option<&Path>) -> Self {
        Self {
            base_path: base_path.map(|p| p.to_path_buf()),
        }
    }

    // ============================================
    // Base Layer (Platform-dependent via AppPaths) - PRIVATE
    // ============================================

    /// Returns a configured AppPaths instance for orcs.
    fn app_paths(&self) -> AppPaths {
        if let Some(ref base) = self.base_path {
            AppPaths::new(Self::APP_PATH).data_strategy(PathStrategy::CustomBase(base.clone()))
        } else {
            AppPaths::new(Self::APP_PATH)
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

    /// Returns the orcs secret directory.
    fn secret_dir(&self) -> Result<PathBuf, PathError> {
        // TODO: Migrate to platform-specific secure storage
        self.config_dir()
    }

    /// Returns the orcs data directory.
    fn data_dir(&self) -> Result<PathBuf, PathError> {
        if let Some(ref base) = self.base_path {
            Ok(base.join(Self::DATA_PATH))
        } else {
            self.app_paths()
                .data_dir()
                .map_err(|_| PathError::HomeDirNotFound)
        }
    }

    /// Returns the path to the files storage directory.
    /// **PRIVATE**: Used internally by get_path() for file-based services.
    fn storage_dir(&self) -> Result<PathBuf, PathError> {
        Ok(self.data_dir()?.join("storage"))
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
            ServiceType::AppState => Ok(PathType::File(self.config_dir()?.join("app_state.json"))),
            ServiceType::Config => Ok(PathType::File(self.config_dir()?.join("config.toml"))),
            ServiceType::Secret => Ok(PathType::File(self.secret_dir()?.join("secret.json"))),

            // Multi-file services - Data (metadata/records)
            ServiceType::Session => Ok(PathType::Dir(self.data_dir()?.join("sessions"))),
            ServiceType::Workspace => Ok(PathType::Dir(self.data_dir()?.join("workspaces"))),
            ServiceType::WorkspaceStorage => {
                Ok(PathType::Dir(self.storage_dir()?.join("workspaces")))
            }
            ServiceType::Task => Ok(PathType::Dir(self.data_dir()?.join("tasks"))),
            ServiceType::Persona => Ok(PathType::Dir(self.data_dir()?.join("personas"))),
            ServiceType::DialoguePreset => {
                Ok(PathType::Dir(self.data_dir()?.join("dialogue_presets")))
            }
            ServiceType::SlashCommand => {
                Ok(PathType::Dir(self.config_dir()?.join("slash_commands")))
            }
            ServiceType::Logs => Ok(PathType::Dir(self.config_dir()?.join("logs"))),
        }
    }

    /// Returns the default user workspace path.
    ///
    /// This is a fallback workspace used when no user workspace is selected.
    /// Unlike other service paths (which are for app metadata), this path
    /// represents an actual user workspace directory.
    ///
    /// # Platform Paths
    ///
    /// - macOS: `~/orcs`
    /// - Linux: `~/orcs`
    /// - Windows: `%USERPROFILE%\orcs`
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)`: Default user workspace path
    /// * `Err(PathError)`: Could not determine home directory
    ///
    /// # Example
    ///
    /// ```rust
    /// use orcs_infrastructure::paths::OrcsPaths;
    ///
    /// let orcs_paths = OrcsPaths::new(None);
    /// let default_workspace_path = orcs_paths.default_user_workspace_path()?;
    /// // Returns: ~/orcs
    /// ```
    pub fn default_user_workspace_path(&self) -> Result<PathBuf, PathError> {
        if let Some(ref base) = self.base_path {
            // For testing: use custom base path
            return Ok(base.join("user_workspace"));
        }

        // Production: use home directory
        let home = dirs::home_dir().ok_or(PathError::HomeDirNotFound)?;
        Ok(home.join("orcs"))
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

        // Setup AppPaths with CustomBase strategy pointing to parent_dir
        // Use empty string as app_name to avoid creating an additional "orcs" directory
        let paths = AppPaths::new("").data_strategy(PathStrategy::CustomBase(parent_dir));

        // Setup default storage strategy: TOML format, Direct filename encoding
        let strategy = DirStorageStrategy::default()
            .with_format(FormatStrategy::Toml)
            .with_filename_encoding(FilenameEncoding::Direct);

        // Create AsyncDirStorage
        AsyncDirStorage::new(paths, &entity_name, migrator, strategy)
            .await
            .map_err(|e| format!("Failed to create AsyncDirStorage: {}", e))
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
        let orcs_paths = OrcsPaths::new(None);
        let config_dir = orcs_paths.config_dir().unwrap();
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
        let orcs_paths = OrcsPaths::new(None);
        let data_dir = orcs_paths.data_dir().unwrap();
        assert!(data_dir.ends_with(OrcsPaths::APP_PATH));
    }

    #[test]
    fn test_secret_dir() {
        let orcs_paths = OrcsPaths::new(None);
        let secret_dir = orcs_paths.secret_dir().unwrap();
        // Currently same as config_dir
        let config_dir = orcs_paths.config_dir().unwrap();
        assert_eq!(secret_dir, config_dir);
    }

    #[test]
    fn test_storage_dir() {
        let orcs_paths = OrcsPaths::new(None);
        let storage_dir = orcs_paths.storage_dir().unwrap();
        assert!(storage_dir.ends_with("files"));
        // Verify it's under data_dir
        let orcs_paths2 = OrcsPaths::new(None);
        let data_dir = orcs_paths2.data_dir().unwrap();
        assert!(storage_dir.starts_with(&data_dir));
    }

    // ============================================
    // Default User Workspace Path Tests
    // ============================================

    #[test]
    fn test_default_user_workspace_path_production() {
        let orcs_paths = OrcsPaths::new(None);
        let workspace_path = orcs_paths.default_user_workspace_path().unwrap();

        // Should end with "orcs"
        assert!(workspace_path.ends_with("orcs"));

        // Should be under home directory
        if let Some(home) = dirs::home_dir() {
            assert!(workspace_path.starts_with(&home));
            assert_eq!(workspace_path, home.join("orcs"));
        }
    }

    #[test]
    fn test_default_user_workspace_path_custom_base() {
        use std::path::PathBuf;

        let custom_base = PathBuf::from("/tmp/test_orcs");
        let orcs_paths = OrcsPaths::new(Some(&custom_base));
        let workspace_path = orcs_paths.default_user_workspace_path().unwrap();

        // Should use custom base path for testing
        assert_eq!(workspace_path, custom_base.join("user_workspace"));
    }
}
