//! Quick Action repository trait.

use async_trait::async_trait;

use super::model::QuickActionConfig;
use crate::OrcsError;

/// Repository trait for Quick Action configuration persistence.
#[async_trait]
pub trait QuickActionRepository: Send + Sync {
    /// Loads the quick action configuration for a workspace.
    /// Returns default config if none exists.
    async fn load(&self, workspace_id: &str) -> Result<QuickActionConfig, OrcsError>;

    /// Saves the quick action configuration for a workspace.
    async fn save(&self, workspace_id: &str, config: &QuickActionConfig) -> Result<(), OrcsError>;
}
