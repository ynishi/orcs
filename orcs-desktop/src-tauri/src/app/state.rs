use std::sync::Arc;

use orcs_application::session::SessionMetadataService;
use orcs_application::{AdhocPersonaService, SessionUseCase};
use orcs_core::{
    persona::PersonaRepository, secret::SecretService, session::AppMode,
    slash_command::SlashCommandRepository, task::TaskRepository, user::UserService,
};
use orcs_execution::TaskExecutor;
use orcs_infrastructure::{
    AppStateService, AsyncDirPersonaRepository, AsyncDirSessionRepository,
    AsyncDirSlashCommandRepository, AsyncDirTaskRepository,
    workspace_manager::FileSystemWorkspaceManager,
};
use tokio::sync::Mutex;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub session_usecase: Arc<SessionUseCase>,
    pub session_repository: Arc<AsyncDirSessionRepository>,
    pub session_metadata_service: Arc<SessionMetadataService>,
    pub app_mode: Mutex<AppMode>,
    pub persona_repository: Arc<dyn PersonaRepository>,
    pub persona_repository_concrete: Arc<AsyncDirPersonaRepository>,
    pub adhoc_persona_service: Arc<AdhocPersonaService>,
    pub user_service: Arc<dyn UserService>,
    pub secret_service: Arc<dyn SecretService>,
    pub workspace_manager: Arc<FileSystemWorkspaceManager>,
    pub slash_command_repository: Arc<dyn SlashCommandRepository>,
    pub slash_command_repository_concrete: Arc<AsyncDirSlashCommandRepository>,
    pub app_state_service: Arc<AppStateService>,
    pub task_repository: Arc<dyn TaskRepository>,
    pub task_repository_concrete: Arc<AsyncDirTaskRepository>,
    pub task_executor: Arc<TaskExecutor>,
}
