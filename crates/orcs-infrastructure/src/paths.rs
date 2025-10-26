//! Unified path management for orcs configuration files.
//!
//! All orcs configuration, secrets, and session data are stored under:
//! `~/.config/orcs/`
//!
//! This ensures consistency across all platforms (Linux, macOS, Windows).

use std::path::PathBuf;

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
/// All paths are resolved relative to `~/.config/orcs/` for consistency
/// across platforms.
///
/// # Directory Structure
///
/// ```text
/// ~/.config/orcs/
/// ├── config.toml      # Application configuration (persona, user_profile)
/// ├── secret.json      # API keys and secrets
/// └── sessions/        # Session files
///     ├── session-1.toml
///     └── session-2.toml
/// ```
pub struct OrcsPaths;

impl OrcsPaths {
    /// Returns the orcs configuration directory: `~/.config/orcs/`
    ///
    /// This directory is used for all orcs configuration, secrets, and session data.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to `~/.config/orcs/`
    /// - `Err(PathError::HomeDirNotFound)`: Could not determine home directory
    pub fn config_dir() -> Result<PathBuf, PathError> {
        let home = dirs::home_dir().ok_or(PathError::HomeDirNotFound)?;
        Ok(home.join(".config").join("orcs"))
    }

    /// Returns the path to the main configuration file: `~/.config/orcs/config.toml`
    ///
    /// This file contains application configuration such as personas and user profile.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to config.toml
    /// - `Err(PathError)`: Could not determine path
    pub fn config_file() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Returns the path to the secrets file: `~/.config/orcs/secret.json`
    ///
    /// This file contains API keys and other sensitive information.
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

    /// Returns the path to the sessions directory: `~/.config/orcs/sessions/`
    ///
    /// This directory contains session files.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)`: Path to sessions directory
    /// - `Err(PathError)`: Could not determine path
    pub fn sessions_dir() -> Result<PathBuf, PathError> {
        Ok(Self::config_dir()?.join("sessions"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let config_dir = OrcsPaths::config_dir().unwrap();
        assert!(config_dir.ends_with(".config/orcs"));
    }

    #[test]
    fn test_config_file() {
        let config_file = OrcsPaths::config_file().unwrap();
        assert!(config_file.ends_with(".config/orcs/config.toml"));
    }

    #[test]
    fn test_secret_file() {
        let secret_file = OrcsPaths::secret_file().unwrap();
        assert!(secret_file.ends_with(".config/orcs/secret.json"));
    }

    #[test]
    fn test_sessions_dir() {
        let sessions_dir = OrcsPaths::sessions_dir().unwrap();
        assert!(sessions_dir.ends_with(".config/orcs/sessions"));
    }
}
