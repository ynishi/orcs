//! Infrastructure layer for ORCS.
//!
//! This crate provides concrete implementations of repositories, services,
//! and storage mechanisms for the ORCS application.
//!
//! # Path Management Architecture
//!
//! All services should use the centralized path management system via `ServiceType`:
//!
//! ```rust
//! use orcs_infrastructure::paths::{OrcsPaths, ServiceType};
//!
//! // In your service implementation
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let base_path = OrcsPaths::new(None).get_path(ServiceType::AppState)?;
//! # Ok(())
//! # }
//! ```
//!
//! This approach ensures:
//! - Centralized path configuration
//! - Easy path changes without service modifications
//! - Consistent directory structure across platforms
//!
//! See [`paths`] module for detailed documentation on the path management system.

pub mod async_dir_dialogue_preset_repository;
pub mod async_dir_persona_repository;
pub mod async_dir_session_repository;
pub mod async_dir_slash_command_repository;
pub mod async_dir_task_repository;
pub mod async_dir_workspace_repository;
pub mod dto;
pub mod paths;
pub mod quick_action_repository;
pub mod search;
pub mod secret_service;
pub mod state_repository;
pub mod storage_repository;
pub mod user_service;
pub mod workspace_storage_service;

pub use crate::async_dir_dialogue_preset_repository::AsyncDirDialoguePresetRepository;
pub use crate::async_dir_persona_repository::AsyncDirPersonaRepository;
pub use crate::async_dir_session_repository::AsyncDirSessionRepository;
pub use crate::async_dir_slash_command_repository::AsyncDirSlashCommandRepository;
pub use crate::async_dir_task_repository::AsyncDirTaskRepository;
pub use crate::async_dir_workspace_repository::AsyncDirWorkspaceRepository;
pub use crate::paths::{OrcsPaths, PathType, ServiceType};
pub use crate::quick_action_repository::FileQuickActionRepository;
pub use crate::secret_service::SecretServiceImpl;
pub use crate::state_repository::AppStateService;
