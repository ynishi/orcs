//! Trait for interaction managers.
//!
//! This module defines the `InteractionManagerTrait` which is implemented
//! by interaction managers in the `orcs-interaction` crate.

use super::app_mode::AppMode;
use super::model::Session;

// Forward declaration - orcs-interaction will provide this
// We use dynamic dispatch to avoid circular dependencies
pub trait InteractionManagerTrait: Send + Sync {
    fn session_id(&self) -> &str;
    fn to_session(
        &self,
        app_mode: AppMode,
        workspace_id: String,
    ) -> impl std::future::Future<Output = Session> + Send;
    fn set_workspace_id(
        &self,
        workspace_id: Option<String>,
        workspace_root: Option<std::path::PathBuf>,
    ) -> impl std::future::Future<Output = ()> + Send;
}
