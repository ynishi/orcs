use std::sync::Arc;

use orcs_application::{AdhocPersonaService, SessionUseCase, UtilityAgentService};
use orcs_core::{
    persona::{get_default_presets, PersonaRepository},
    session::{AppMode, SessionManager},
    slash_command::SlashCommandRepository,
    task::TaskRepository,
    user::UserService,
    workspace::manager::WorkspaceManager,
};
use orcs_execution::{
    tracing_layer::OrchestratorEvent,
    TaskExecutor,
};
use orcs_infrastructure::{
    paths::{OrcsPaths, ServiceType},
    user_service::ConfigBasedUserService,
    workspace_manager::FileSystemWorkspaceManager,
    AppStateService, AsyncDirPersonaRepository, AsyncDirSessionRepository,
    AsyncDirSlashCommandRepository, AsyncDirTaskRepository,
};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::app::AppState;

pub struct AppBootstrap {
    pub app_state: AppState,
    pub session_manager: Arc<SessionManager<orcs_interaction::InteractionManager>>,
    pub app_state_service: Arc<AppStateService>,
}

pub async fn bootstrap(event_tx: UnboundedSender<OrchestratorEvent>) -> AppBootstrap {
    // Composition Root: Create the concrete repository instances
    let persona_repository_concrete = Arc::new(
        AsyncDirPersonaRepository::default_location()
            .await
            .expect("Failed to initialize persona repository"),
    );
    let persona_repository: Arc<dyn PersonaRepository> = persona_repository_concrete.clone();

    // Create AdhocPersonaService
    let adhoc_persona_service = Arc::new(AdhocPersonaService::new(persona_repository.clone()));

    let user_service: Arc<dyn UserService> = Arc::new(ConfigBasedUserService::new());

    // Initialize FileSystemWorkspaceManager with unified path
    let path_type = OrcsPaths::new(None)
        .get_path(ServiceType::Workspace)
        .expect("Failed to get workspaces directory");
    let workspace_root = path_type.into_path_buf();
    let workspace_manager = Arc::new(
        FileSystemWorkspaceManager::new(workspace_root)
            .await
            .expect("Failed to initialize workspace manager"),
    );

    // Initialize AsyncDirSlashCommandRepository
    let slash_command_repository_concrete = Arc::new(
        AsyncDirSlashCommandRepository::new()
            .await
            .expect("Failed to initialize slash command repository"),
    );
    let slash_command_repository: Arc<dyn SlashCommandRepository> =
        slash_command_repository_concrete.clone();

    // Seed the personas directory with default personas if it's empty on first run.
    if let Ok(personas) = persona_repository.get_all() {
        if personas.is_empty() {
            let default_presets = get_default_presets();
            if let Err(e) = persona_repository.save_all(&default_presets) {
                // This is a critical failure on startup, so we panic.
                panic!("Failed to seed default personas: {}", e);
            }
        }
    }

    // Create AsyncDirSessionRepository at default location
    let session_repository = Arc::new(
        AsyncDirSessionRepository::default_location(persona_repository.clone())
            .await
            .expect("Failed to create session repository"),
    );

    // Initialize AppStateService
    let app_state_service = Arc::new(
        AppStateService::new()
            .await
            .expect("Failed to initialize AppStateService"),
    );

    // Initialize SessionManager with both repositories
    let session_manager: Arc<SessionManager<orcs_interaction::InteractionManager>> = Arc::new(
        SessionManager::new(session_repository.clone(), app_state_service.clone()),
    );

    // Create SessionUseCase for coordinated session-workspace management
    let session_usecase = Arc::new(SessionUseCase::new(
        session_manager.clone(),
        workspace_manager.clone(),
        app_state_service.clone(),
        persona_repository.clone(),
        user_service.clone(),
    ));

    // Create Task Repository
    let task_repository_concrete = Arc::new(
        AsyncDirTaskRepository::default_location()
            .await
            .expect("Failed to initialize Task Repository"),
    );
    let task_repository = task_repository_concrete.clone() as Arc<dyn TaskRepository>;

    // Create UtilityAgentService for lightweight LLM operations
    let utility_service = Arc::new(UtilityAgentService::new());

    // Create TaskExecutor with all services
    let task_executor = Arc::new(
        TaskExecutor::new()
            .with_task_repository(task_repository.clone())
            .with_event_sender(event_tx.clone())
            .with_utility_service(utility_service.clone()),
    );

    // Try to restore last session using SessionUseCase
    let restored = session_usecase
        .restore_last_session()
        .await
        .ok()
        .flatten();

    if restored.is_none() {
        // Try to restore workspace from last selected workspace ID
        let workspace_selected = if let Some(last_workspace_id) =
            app_state_service.get_last_selected_workspace().await
        {
            tracing::info!(
                "[Startup] Found last selected workspace: {}",
                last_workspace_id
            );

            // Try to switch to that workspace
            match session_usecase.switch_workspace(&last_workspace_id).await {
                Ok(_) => {
                    tracing::info!(
                        "[Startup] Successfully restored workspace: {}",
                        last_workspace_id
                    );
                    true
                }
                Err(e) => {
                    tracing::warn!(
                        "[Startup] Failed to restore last workspace {}: {}",
                        last_workspace_id,
                        e
                    );
                    false
                }
            }
        } else {
            false
        };

        if !workspace_selected {
            // No last workspace or failed to restore - try to find any workspace
            match workspace_manager.list_all_workspaces().await {
                Ok(workspaces) if !workspaces.is_empty() => {
                    // Use most recently accessed workspace
                    let most_recent = &workspaces[0];
                    tracing::info!(
                        "[Startup] Using most recent workspace: {} ({})",
                        most_recent.name,
                        most_recent.id
                    );

                    if let Err(e) = session_usecase.switch_workspace(&most_recent.id).await {
                        tracing::warn!(
                            "[Startup] Failed to switch to most recent workspace: {}",
                            e
                        );
                        tracing::info!("[Startup] Creating new session without workspace");
                    }
                }
                Ok(_) => {
                    tracing::info!(
                        "[Startup] No workspaces found, starting with empty session"
                    );
                }
                Err(e) => {
                    tracing::warn!("[Startup] Failed to list workspaces: {}", e);
                }
            }
        }
    }

    let app_mode = Mutex::new(AppMode::Idle);

    let app_state = AppState {
        session_usecase,
        session_manager: session_manager.clone(),
        session_repository: session_repository.clone(),
        app_mode,
        persona_repository,
        persona_repository_concrete,
        adhoc_persona_service,
        user_service,
        workspace_manager: workspace_manager.clone(),
        slash_command_repository,
        slash_command_repository_concrete,
        app_state_service: app_state_service.clone(),
        task_repository,
        task_repository_concrete,
        task_executor,
    };

    AppBootstrap {
        app_state,
        session_manager,
        app_state_service,
    }
}

