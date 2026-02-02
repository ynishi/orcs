use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use orcs_application::session::SessionMetadataService;
use orcs_application::{AdhocPersonaService, SessionUseCase};
use orcs_core::{
    dialogue::DialoguePresetRepository, persona::PersonaRepository,
    quick_action::QuickActionRepository, secret::SecretService, session::AppMode,
    slash_command::SlashCommandRepository, task::TaskRepository, user::UserService,
};
use orcs_execution::tracing_layer::OrchestratorEvent;
use orcs_execution::TaskExecutor;
use orcs_infrastructure::{
    workspace_storage_service::FileSystemWorkspaceManager, AppStateService,
    AsyncDirDialoguePresetRepository, AsyncDirPersonaRepository, AsyncDirSessionRepository,
    AsyncDirSlashCommandRepository, AsyncDirTaskRepository, ConfigService,
    FileQuickActionRepository,
};
use tokio::sync::mpsc::UnboundedSender;
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
    pub workspace_storage_service: Arc<FileSystemWorkspaceManager>,
    pub slash_command_repository: Arc<dyn SlashCommandRepository>,
    pub slash_command_repository_concrete: Arc<AsyncDirSlashCommandRepository>,
    pub dialogue_preset_repository: Arc<dyn DialoguePresetRepository>,
    #[allow(dead_code)]
    pub dialogue_preset_repository_concrete: Arc<AsyncDirDialoguePresetRepository>,
    pub app_state_service: Arc<AppStateService>,
    pub config_service: Arc<ConfigService>,
    pub task_repository: Arc<dyn TaskRepository>,
    pub task_repository_concrete: Arc<AsyncDirTaskRepository>,
    pub task_executor: Arc<TaskExecutor>,
    pub event_sender: UnboundedSender<OrchestratorEvent>,
    pub cancel_flag: Arc<AtomicBool>,
    pub quick_action_repository: Arc<dyn QuickActionRepository>,
    #[allow(dead_code)]
    pub quick_action_repository_concrete: Arc<FileQuickActionRepository>,
}
