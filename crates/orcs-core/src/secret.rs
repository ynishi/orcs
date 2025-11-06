//! Secret management service trait.
//!
//! Defines the interface for loading and managing secret configuration (API keys).

use crate::config::SecretConfig;

/// Service for managing secret configuration.
///
/// This trait defines the interface for loading API keys and other sensitive
/// configuration data from secure storage.
///
/// # Security Note
///
/// Implementations should ensure that:
/// - Secret files have appropriate permissions (e.g., 600 on Unix)
/// - Secrets are never logged or exposed in error messages
/// - Secrets are loaded from secure locations
#[async_trait::async_trait]
pub trait SecretService: Send + Sync {
    /// Loads the secret configuration.
    ///
    /// # Returns
    ///
    /// - `Ok(SecretConfig)`: Successfully loaded secrets
    /// - `Err(String)`: Failed to load (error message should not contain secrets)
    async fn load_secrets(&self) -> Result<SecretConfig, String>;

    /// Checks if the secret file exists.
    ///
    /// # Returns
    ///
    /// `true` if the secret file exists, `false` otherwise.
    async fn secret_file_exists(&self) -> bool;
}
