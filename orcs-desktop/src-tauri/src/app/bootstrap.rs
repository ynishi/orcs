use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use anyhow::{Result, anyhow};
use orcs_application::session::{SessionMetadataService, SessionUpdater};
use orcs_application::{AdhocPersonaService, SessionUseCase, UtilityAgentService};
use orcs_core::{
    dialogue::DialoguePresetRepository,
    persona::{PersonaRepository, get_default_presets},
    quick_action::QuickActionRepository,
    repository::SessionRepository,
    secret::SecretService,
    session::{AppMode, PLACEHOLDER_WORKSPACE_ID},
    slash_command::SlashCommandRepository,
    state::repository::StateRepository,
    task::TaskRepository,
    user::UserService,
    workspace::manager::WorkspaceStorageService,
};
use orcs_execution::{TaskExecutor, tracing_layer::OrchestratorEvent};
use orcs_infrastructure::{
    AppStateService, AsyncDirDialoguePresetRepository, AsyncDirPersonaRepository,
    AsyncDirSessionRepository, AsyncDirSlashCommandRepository, AsyncDirTaskRepository,
    ConfigService, FileQuickActionRepository, SecretServiceImpl, paths::OrcsPaths,
    user_service::ConfigBasedUserService, workspace_storage_service::FileSystemWorkspaceManager,
};
use tokio::sync::{Mutex, mpsc::UnboundedSender};

use crate::app::AppState;

pub struct AppBootstrap {
    pub app_state: AppState,
}

/// Ensures a default workspace exists and returns its ID.
///
/// This function:
/// 1. Checks if a default workspace is already set in AppState
/// 2. If yes and it exists, returns the ID
/// 3. If no or it doesn't exist, creates ~/orcs as the default workspace
/// 4. Saves the ID to AppState
///
/// # Returns
///
/// The workspace ID of the default workspace
async fn ensure_default_workspace(
    workspace_storage_service: &Arc<FileSystemWorkspaceManager>,
    app_state_service: &Arc<AppStateService>,
) -> Result<String> {
    // 1. Check existing default_workspace_id
    let current_default_id = app_state_service.get_default_workspace().await;

    // Skip if it exists and the workspace is valid
    if let Some(workspace_id) = current_default_id
        && let Ok(Some(_)) = workspace_storage_service.get_workspace(&workspace_id).await
    {
        tracing::info!(
            "[Bootstrap] Using existing default workspace: {}",
            workspace_id
        );
        return Ok(workspace_id);
    }

    // 2. Get default user workspace path from Infrastructure
    let orcs_paths = OrcsPaths::new(None);
    let default_path = orcs_paths
        .default_user_workspace_path()
        .map_err(|e| anyhow!("Failed to get default workspace path: {}", e))?;

    tracing::info!(
        "[Bootstrap] Creating default workspace at: {:?}",
        default_path
    );

    // 2.5. Ensure the directory exists before creating workspace
    tokio::fs::create_dir_all(&default_path)
        .await
        .map_err(|e| anyhow!("Failed to create default workspace directory: {}", e))?;

    // 3. Create workspace (ID will be deterministically generated from path)
    let workspace = workspace_storage_service
        .get_or_create_workspace(&default_path)
        .await
        .map_err(|e| anyhow!("Failed to create default workspace: {}", e))?;

    tracing::info!(
        "[Bootstrap] Default workspace created with ID: {}",
        workspace.id
    );

    // 4. Save to AppState
    app_state_service
        .set_default_workspace(workspace.id.clone())
        .await
        .map_err(|e| anyhow!("Failed to save default workspace ID: {}", e))?;

    Ok(workspace.id)
}

/// Replaces placeholder workspace IDs in existing sessions with the actual default workspace ID.
///
/// This is necessary after migration from v2.9.0 to v3.0.0 where workspace_id became required.
async fn replace_placeholder_sessions(
    session_repository: &Arc<AsyncDirSessionRepository>,
    default_workspace_id: &str,
) -> Result<()> {
    // Cast to trait to use list_all method
    let repo: &dyn SessionRepository = session_repository.as_ref();
    let sessions = repo
        .list_all()
        .await
        .map_err(|e| anyhow!("Failed to list sessions: {}", e))?;

    let mut updated_count = 0;
    for mut session in sessions {
        if session.workspace_id == PLACEHOLDER_WORKSPACE_ID {
            tracing::info!(
                "[Bootstrap] Replacing placeholder workspace_id in session: {}",
                session.id
            );
            session.workspace_id = default_workspace_id.to_string();
            repo.save(&session)
                .await
                .map_err(|e| anyhow!("Failed to save session: {}", e))?;
            updated_count += 1;
        }
    }

    if updated_count > 0 {
        tracing::info!(
            "[Bootstrap] Replaced placeholder workspace_id in {} session(s)",
            updated_count
        );
    }

    Ok(())
}

pub async fn bootstrap(event_tx: UnboundedSender<OrchestratorEvent>) -> AppBootstrap {
    // Composition Root: Create the concrete repository instances
    let persona_repository_concrete = Arc::new(
        AsyncDirPersonaRepository::new(None)
            .await
            .expect("Failed to initialize persona repository"),
    );
    let persona_repository: Arc<dyn PersonaRepository> = persona_repository_concrete.clone();

    // Create AdhocPersonaService
    let adhoc_persona_service = Arc::new(AdhocPersonaService::new(persona_repository.clone()));

    // Initialize UserService and ensure config.toml exists by loading profile
    let user_service_impl = ConfigBasedUserService::new();
    let user_service: Arc<dyn UserService> = Arc::new(user_service_impl);

    // Initialize SecretService and ensure secret.json exists by loading secrets
    let secret_service_impl =
        SecretServiceImpl::new_default().expect("Failed to initialize secret service");
    let _ = secret_service_impl.load_secrets().await; // Trigger file creation if missing
    let secret_service: Arc<dyn SecretService> = Arc::new(secret_service_impl);

    let workspace_storage_service = Arc::new(
        FileSystemWorkspaceManager::default()
            .await
            .expect("Failed to initialize workspace manager"),
    );

    // Initialize AsyncDirSlashCommandRepository
    let slash_command_repository_concrete = Arc::new(
        AsyncDirSlashCommandRepository::new(None)
            .await
            .expect("Failed to initialize slash command repository"),
    );
    let slash_command_repository: Arc<dyn SlashCommandRepository> =
        slash_command_repository_concrete.clone();

    // Initialize AsyncDirDialoguePresetRepository
    let dialogue_preset_repository_concrete = Arc::new(
        AsyncDirDialoguePresetRepository::new(None)
            .await
            .expect("Failed to initialize dialogue preset repository"),
    );
    let dialogue_preset_repository: Arc<dyn DialoguePresetRepository> =
        dialogue_preset_repository_concrete.clone();

    // Seed the personas directory with default personas if it's empty on first run.
    if let Ok(personas) = persona_repository.get_all().await
        && personas.is_empty()
    {
        let default_presets = get_default_presets();
        if let Err(e) = persona_repository.save_all(&default_presets).await {
            // This is a critical failure on startup, so we panic.
            panic!("Failed to seed default personas: {}", e);
        }
    }

    // Create AsyncDirSessionRepository at default location
    let session_repository = Arc::new(
        AsyncDirSessionRepository::new(None)
            .await
            .expect("Failed to create session repository"),
    );

    // Initialize AppStateService
    let app_state_service = Arc::new(
        AppStateService::new()
            .await
            .expect("Failed to initialize AppStateService"),
    );

    // Initialize ConfigService
    let config_service = Arc::new(ConfigService::new());

    // Ensure default workspace exists (before session restoration)
    let default_workspace_id =
        ensure_default_workspace(&workspace_storage_service, &app_state_service)
            .await
            .expect("Failed to ensure default workspace");

    tracing::info!("[Bootstrap] Default workspace ID: {}", default_workspace_id);

    // Replace placeholder workspace IDs in existing sessions
    replace_placeholder_sessions(&session_repository, &default_workspace_id)
        .await
        .expect("Failed to replace placeholder sessions");

    // Create SessionMetadataService for session metadata operations
    let session_updater = SessionUpdater::new(session_repository.clone());
    let session_metadata_service = Arc::new(SessionMetadataService::new(session_updater));

    // Create SessionUseCase for coordinated session-workspace management
    let session_usecase = Arc::new(SessionUseCase::new(
        session_repository.clone(),
        workspace_storage_service.clone(),
        app_state_service.clone(),
        persona_repository.clone(),
        user_service.clone(),
    ));

    // Create Task Repository
    let task_repository_concrete = Arc::new(
        AsyncDirTaskRepository::new(None)
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

    // Create QuickAction Repository
    let quick_action_repository_concrete = Arc::new(
        FileQuickActionRepository::new()
            .await
            .expect("Failed to initialize Quick Action Repository"),
    );
    let quick_action_repository =
        quick_action_repository_concrete.clone() as Arc<dyn QuickActionRepository>;

    // Try to restore last session using SessionUseCase
    let restored = session_usecase.restore_last_session().await.ok().flatten();

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
            match workspace_storage_service.list_all_workspaces().await {
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
                    tracing::info!("[Startup] No workspaces found, starting with empty session");
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
        session_repository: session_repository.clone(),
        session_metadata_service,
        app_mode,
        persona_repository,
        persona_repository_concrete,
        adhoc_persona_service,
        user_service,
        secret_service,
        workspace_storage_service: workspace_storage_service.clone(),
        slash_command_repository,
        slash_command_repository_concrete,
        dialogue_preset_repository,
        dialogue_preset_repository_concrete,
        app_state_service: app_state_service.clone(),
        config_service,
        task_repository,
        task_repository_concrete,
        task_executor,
        cancel_flag: Arc::new(AtomicBool::new(false)),
        quick_action_repository,
        quick_action_repository_concrete,
    };

    AppBootstrap { app_state }
}
