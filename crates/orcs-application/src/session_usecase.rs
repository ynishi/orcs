//! Session use case implementation.
//!
//! This module provides the `SessionUseCase` which orchestrates interactions
//! between `SessionManager` and `WorkspaceStorageService` to ensure data consistency
//! and proper state management across workspace-session relationships.

use crate::session::{SessionCache, SessionFactory, SessionUpdater};
use anyhow::{Result, anyhow};
use orcs_core::memory::MemorySyncService;
use orcs_core::repository::PersonaRepository;
use orcs_core::session::{AppMode, PLACEHOLDER_WORKSPACE_ID, Session, SessionRepository};
use orcs_core::state::repository::StateRepository;
use orcs_core::user::UserService;
use orcs_core::workspace::manager::WorkspaceStorageService;
use orcs_interaction::InteractionManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Use case for managing sessions with workspace context.
///
/// `SessionUseCase` coordinates between `SessionRepository`, `WorkspaceStorageService`,
/// and `AppStateService` to handle all session-related operations while maintaining
/// consistency between sessions and their associated workspaces.
///
/// # Responsibilities
///
/// - Creating new sessions with proper workspace association
/// - Switching between sessions and restoring workspace context
/// - Managing workspace changes within sessions
/// - Validating and cleaning up orphaned workspace references
/// - Coordinating application state (selected workspace) with session state
///
/// # Thread Safety
///
/// All internal components are wrapped in `Arc` and use interior mutability
/// (`RwLock`, `Mutex`) for thread-safe concurrent access.
/// Callback type for memory sync errors.
pub type MemorySyncErrorCallback = Arc<dyn Fn(String) + Send + Sync>;

pub struct SessionUseCase {
    /// Repository for session data persistence
    session_repository: Arc<dyn SessionRepository>,
    /// Cache for in-memory InteractionManager instances
    session_cache: Arc<SessionCache<InteractionManager>>,
    /// Factory for creating InteractionManager instances
    session_factory: Arc<SessionFactory>,
    /// Manager for workspace operations
    workspace_storage_service: Arc<dyn WorkspaceStorageService>,
    /// Service for application-level state (e.g., last selected workspace)
    app_state_service: Arc<orcs_infrastructure::AppStateService>,
    /// Repository for persona configurations (for enrich_session_participants)
    persona_repository: Arc<dyn PersonaRepository>,
    /// Service for user information (for enrich_session_participants)
    user_service: Arc<dyn UserService>,
    /// Optional memory sync service for RAG integration
    memory_sync_service: Arc<RwLock<Option<Arc<dyn MemorySyncService>>>>,
    /// Optional callback for memory sync errors (for UI notifications)
    memory_sync_error_callback: Arc<RwLock<Option<MemorySyncErrorCallback>>>,
}

impl SessionUseCase {
    /// Creates a new `SessionUseCase` instance.
    ///
    /// # Arguments
    ///
    /// * `session_repository` - Repository for session data persistence
    /// * `workspace_storage_service` - Manager for workspace operations
    /// * `app_state_service` - Service for application-level state
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        workspace_storage_service: Arc<dyn WorkspaceStorageService>,
        app_state_service: Arc<orcs_infrastructure::AppStateService>,
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            session_repository: session_repository.clone(),
            session_cache: Arc::new(SessionCache::new()),
            session_factory: Arc::new(SessionFactory::new(
                persona_repository.clone(),
                user_service.clone(),
            )),
            workspace_storage_service,
            app_state_service,
            persona_repository,
            user_service,
            memory_sync_service: Arc::new(RwLock::new(None)),
            memory_sync_error_callback: Arc::new(RwLock::new(None)),
        }
    }

    /// Sets the memory sync service for RAG integration.
    ///
    /// When set, session saves will trigger background memory synchronization
    /// to the configured RAG backend (e.g., Kaiba).
    pub async fn set_memory_sync_service(&self, service: Arc<dyn MemorySyncService>) {
        *self.memory_sync_service.write().await = Some(service);
    }

    /// Sets a callback to be invoked when memory sync errors occur.
    ///
    /// This is used to notify the UI (e.g., show a toast notification).
    pub async fn set_memory_sync_error_callback(&self, callback: MemorySyncErrorCallback) {
        *self.memory_sync_error_callback.write().await = Some(callback);
    }

    /// Starts a background scheduler for memory synchronization.
    ///
    /// The scheduler runs at the specified interval and syncs sessions that have been
    /// updated since their last memory sync (`updated_at > last_memory_sync_at`).
    ///
    /// This approach avoids the parallel execution issues that occurred when
    /// triggering sync on every `save_active_session` call.
    ///
    /// # Arguments
    ///
    /// * `interval_secs` - Interval in seconds between sync batches (default: 60)
    pub fn start_memory_sync_scheduler(self: &Arc<Self>, interval_secs: u64) {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Duration;
        use tokio::time::interval;

        // Prevent multiple scheduler instances
        static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);
        if SCHEDULER_RUNNING.swap(true, Ordering::SeqCst) {
            tracing::warn!("[MemorySyncScheduler] Scheduler already running, skipping");
            return;
        }

        let usecase = Arc::clone(self);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));
            tracing::info!(target: "memory_sync", "Scheduler started ({}s interval)", interval_secs);

            loop {
                ticker.tick().await;
                tracing::debug!(target: "memory_sync", "Tick - checking for sessions to sync");

                if let Err(e) = usecase.run_memory_sync_batch().await {
                    tracing::error!(target: "memory_sync", "Batch sync failed: {}", e);
                }
            }
        });
    }

    /// Runs a single batch of memory synchronization.
    ///
    /// Scans all sessions and syncs those where `updated_at > last_memory_sync_at`.
    async fn run_memory_sync_batch(&self) -> Result<()> {
        use chrono::DateTime;

        // Check if memory sync service is configured
        let sync_service = {
            let guard = self.memory_sync_service.read().await;
            guard.clone()
        };

        let Some(sync_service) = sync_service else {
            tracing::debug!(target: "memory_sync", "No memory sync service configured, skipping");
            return Ok(());
        };

        // Get all sessions
        let sessions = self.session_repository.list_all().await?;
        tracing::debug!(target: "memory_sync", "Checking {} sessions for sync", sessions.len());

        // Sync result counters
        let mut sync_success = 0;
        let mut sync_failed = 0;
        // Skip reason counters
        let mut skipped_already_synced = 0;
        let mut skipped_workspace_not_found = 0;
        let mut skipped_workspace_error = 0;
        let mut skipped_rei_create_failed = 0;
        let mut skipped_no_messages = 0;

        for session in &sessions {
            // Check if session needs sync: updated_at > last_memory_sync_at
            // Use DateTime comparison for accuracy
            let needs_sync = match &session.last_memory_sync_at {
                None => true, // Never synced
                Some(last_sync) => {
                    match (
                        DateTime::parse_from_rfc3339(&session.updated_at),
                        DateTime::parse_from_rfc3339(last_sync),
                    ) {
                        (Ok(updated), Ok(last)) => updated > last,
                        // If parsing fails, fall back to string comparison
                        _ => session.updated_at.as_str() > last_sync.as_str(),
                    }
                }
            };

            if !needs_sync {
                skipped_already_synced += 1;
                continue;
            }

            tracing::debug!(
                target: "memory_sync",
                "Syncing session {} (updated_at: {}, last_sync: {:?})",
                session.id,
                session.updated_at,
                session.last_memory_sync_at
            );

            // Get workspace for Rei ID
            let workspace = match self
                .workspace_storage_service
                .get_workspace(&session.workspace_id)
                .await
            {
                Ok(Some(ws)) => ws,
                Ok(None) => {
                    tracing::warn!(
                        target: "memory_sync",
                        "Workspace {} not found for session {}, marking as synced",
                        session.workspace_id,
                        session.id
                    );
                    // Still update last_memory_sync_at to prevent infinite retry loop
                    let mut updated_session = session.clone();
                    updated_session.last_memory_sync_at = Some(session.updated_at.clone());
                    let _ = self.session_repository.save(&updated_session).await;
                    skipped_workspace_not_found += 1;
                    continue;
                }
                Err(e) => {
                    tracing::warn!(
                        target: "memory_sync",
                        "Error getting workspace {} for session {}: {}, marking as synced",
                        session.workspace_id,
                        session.id,
                        e
                    );
                    // Still update last_memory_sync_at to prevent infinite retry loop
                    let mut updated_session = session.clone();
                    updated_session.last_memory_sync_at = Some(session.updated_at.clone());
                    let _ = self.session_repository.save(&updated_session).await;
                    skipped_workspace_error += 1;
                    continue;
                }
            };

            // Get or create Rei ID
            let rei_id = match workspace.kaiba_rei_id.clone() {
                Some(id) => id,
                None => {
                    // Create new Rei
                    match sync_service
                        .get_or_create_rei(&workspace.id, &workspace.name)
                        .await
                    {
                        Ok(new_rei_id) => {
                            // Save Rei ID to workspace
                            let mut updated_workspace = workspace.clone();
                            updated_workspace.kaiba_rei_id = Some(new_rei_id.clone());
                            if let Err(e) = self
                                .workspace_storage_service
                                .save_workspace(&updated_workspace)
                                .await
                            {
                                tracing::warn!(
                                    "[MemorySyncScheduler] Failed to save kaiba_rei_id: {}",
                                    e
                                );
                            }
                            new_rei_id
                        }
                        Err(e) => {
                            tracing::warn!(
                                target: "memory_sync",
                                "Failed to create Rei for workspace {}: {}, marking session {} as synced",
                                workspace.name,
                                e,
                                session.id
                            );
                            // Still update last_memory_sync_at to prevent infinite retry loop
                            let mut updated_session = session.clone();
                            updated_session.last_memory_sync_at = Some(session.updated_at.clone());
                            let _ = self.session_repository.save(&updated_session).await;
                            skipped_rei_create_failed += 1;
                            continue;
                        }
                    }
                }
            };

            // Collect and sync messages
            let messages = Self::collect_messages_for_sync(&session);
            if messages.is_empty() {
                // Update last_memory_sync_at even if no messages to sync
                // (prevents re-checking unchanged sessions)
                let mut updated_session = session.clone();
                updated_session.last_memory_sync_at = Some(session.updated_at.clone());
                if let Err(e) = self.session_repository.save(&updated_session).await {
                    tracing::warn!(
                        target: "memory_sync",
                        "Failed to update last_memory_sync_at for session {}: {}",
                        session.id,
                        e
                    );
                }
                skipped_no_messages += 1;
                continue;
            }

            tracing::debug!(
                target: "memory_sync",
                "Syncing {} messages for session {}",
                messages.len(),
                session.id
            );

            let result = sync_service.sync_messages(&rei_id, messages).await;

            if let Some(error) = &result.error {
                tracing::warn!(
                    target: "memory_sync",
                    "Sync partially failed for session {}: {}",
                    session.id,
                    error
                );
                sync_failed += 1;
                // Notify via callback if configured
                if let Some(cb) = &*self.memory_sync_error_callback.read().await {
                    cb(format!("Memory sync failed: {}", error));
                }
            } else {
                tracing::debug!(
                    target: "memory_sync",
                    "Synced {} messages for session {}",
                    result.synced_count,
                    session.id
                );
                sync_success += 1;
            }

            // Update last_memory_sync_at
            let mut updated_session = session.clone();
            updated_session.last_memory_sync_at = Some(session.updated_at.clone());
            match self.session_repository.save(&updated_session).await {
                Ok(_) => {
                    tracing::debug!(
                        target: "memory_sync",
                        "Saved last_memory_sync_at for session {}",
                        session.id
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        target: "memory_sync",
                        "Failed to save last_memory_sync_at for session {}: {}",
                        session.id, e
                    );
                }
            }
        }

        tracing::info!(
            target: "memory_sync",
            "Batch complete: sync_success={}, sync_failed={}, skipped(already_synced={}, no_messages={}, workspace_not_found={}, workspace_error={}, rei_failed={})",
            sync_success,
            sync_failed,
            skipped_already_synced,
            skipped_no_messages,
            skipped_workspace_not_found,
            skipped_workspace_error,
            skipped_rei_create_failed
        );

        Ok(())
    }

    /// Creates a new session associated with the specified workspace.
    ///
    /// This method implements UC1 (Session Creation with Workspace Association):
    /// 1. Validates that the specified workspace exists
    /// 2. Creates a new session
    /// 3. Associates the session with the workspace
    /// 4. Persists the session with workspace_id
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The workspace ID to associate with the new session (required)
    ///
    /// # Returns
    ///
    /// Returns the newly created session with workspace association.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The specified workspace does not exist
    /// - The session creation fails
    /// - The session persistence fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let session = usecase.create_session("workspace-123").await?;
    /// println!("Created session {} in workspace {}", session.id, session.workspace_id);
    /// ```
    pub async fn create_session(&self, workspace_id: &str) -> Result<Session> {
        tracing::info!(
            "[SessionUseCase] Creating new session with workspace: {}",
            workspace_id
        );

        // 1. Validate workspace exists
        let workspace = self
            .workspace_storage_service
            .get_workspace(workspace_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found: {}", workspace_id))?;

        tracing::info!(
            "[SessionUseCase] Found workspace: {} at {}",
            workspace.name,
            workspace.root_path.display()
        );

        // 2. Create session
        let session_id = Uuid::new_v4().to_string();
        tracing::debug!("[SessionUseCase] Generated session ID: {}", session_id);

        // Create InteractionManager using factory
        let manager = Arc::new(
            self.session_factory
                .create_interaction_manager(session_id.clone()),
        );

        // 3. Associate with workspace
        manager
            .set_workspace_id(
                Some(workspace.id.clone()),
                Some(workspace.root_path.clone()),
            )
            .await;

        // Insert into cache
        self.session_cache
            .insert(session_id.clone(), manager.clone())
            .await;

        tracing::info!(
            "[SessionUseCase] Session {} created and associated with workspace {}",
            session_id,
            workspace.id
        );

        // 4. Persist session
        let session = self
            .session_factory
            .to_session(manager.as_ref(), AppMode::Idle, workspace_id.to_string())
            .await;
        self.session_repository.save(&session).await?;

        // 5. Set as active session
        self.app_state_service
            .set_active_session(session_id.clone())
            .await
            .map_err(|e| anyhow!("Failed to set active session: {}", e))?;

        // 6. Return session
        Ok(session)
    }

    /// Creates a new workspace and immediately creates a session associated with it.
    ///
    /// This is the recommended way to create workspaces, as a workspace without
    /// a session doesn't make sense - why create a workspace if you're not going
    /// to work in it?
    ///
    /// This method ensures atomicity: both workspace and session are created together,
    /// and the workspace is set as the currently selected workspace in AppStateService.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The root directory path for the new workspace
    ///
    /// # Returns
    ///
    /// Returns a tuple of (Workspace, Session) representing the newly created workspace
    /// and its associated session.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace cannot be created or retrieved
    /// - The workspace selection cannot be saved to AppStateService
    /// - The session creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::path::Path;
    /// # use orcs_application::session_usecase::SessionUseCase;
    /// # use anyhow::Result;
    /// # async fn example(session_usecase: &SessionUseCase) -> Result<()> {
    /// let (_workspace, _session) = session_usecase
    ///     .create_workspace_with_session(Path::new("/path/to/project"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_workspace_with_session(
        &self,
        root_path: &std::path::Path,
    ) -> Result<(orcs_core::workspace::Workspace, Session)> {
        tracing::info!(
            "[SessionUseCase] Creating workspace with session at: {}",
            root_path.display()
        );

        // 1. Get or create the workspace
        let workspace = self
            .workspace_storage_service
            .get_or_create_workspace(root_path)
            .await
            .map_err(|e| anyhow!("Failed to get/create workspace: {}", e))?;

        tracing::info!(
            "[SessionUseCase] Workspace created/retrieved: {} ({})",
            workspace.name,
            workspace.id
        );

        // 2. Update AppStateService to use this workspace
        self.app_state_service
            .set_last_selected_workspace(workspace.id.clone())
            .await
            .map_err(|e| anyhow!("Failed to set workspace selection: {}", e))?;

        tracing::debug!(
            "[SessionUseCase] Set last_selected_workspace to {}",
            workspace.id
        );

        // 3. Check if there are existing sessions for this workspace
        let existing_sessions = self
            .session_repository
            .list_all()
            .await
            .map_err(|e| anyhow!("Failed to list sessions: {}", e))?;

        let workspace_sessions: Vec<_> = existing_sessions
            .into_iter()
            .filter(|s| s.workspace_id == workspace.id)
            .collect();

        let session = if !workspace_sessions.is_empty() {
            // Restore the most recent session for this workspace
            let latest_session = workspace_sessions.first().unwrap();
            tracing::info!(
                "[SessionUseCase] Found {} existing session(s) for workspace {}, restoring latest: {}",
                workspace_sessions.len(),
                workspace.id,
                latest_session.id
            );

            // Switch to the existing session
            self.switch_session(&latest_session.id).await?;
            latest_session.clone()
        } else {
            // Create new session for this workspace
            tracing::info!(
                "[SessionUseCase] No existing sessions found, creating new session for workspace {}",
                workspace.id
            );
            self.create_session(&workspace.id).await?
        };

        tracing::info!(
            "[SessionUseCase] Session {} associated with workspace {}",
            session.id,
            workspace.id
        );

        Ok((workspace, session))
    }

    /// Creates a new config session with system prompt in a specific workspace.
    ///
    /// This is a specialized version of `create_session` for configuration assistance.
    /// It creates a session in the specified workspace (typically ~/.config/orcs) and
    /// adds a system prompt to guide the AI in configuration tasks.
    ///
    /// # Arguments
    ///
    /// * `workspace_root_path` - The root path for the admin workspace (e.g., ~/.config/orcs)
    /// * `system_prompt` - The system prompt containing configuration guidance
    ///
    /// # Returns
    ///
    /// Returns the newly created config session with the system prompt added.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace cannot be created or retrieved
    /// - The session creation fails
    /// - Adding the system prompt fails
    pub async fn create_config_session(
        &self,
        workspace_root_path: String,
        system_prompt: String,
    ) -> Result<Session> {
        tracing::info!(
            "[SessionUseCase] Creating config session at workspace: {}",
            workspace_root_path
        );

        // 1. Get or create the admin workspace
        let workspace = self
            .workspace_storage_service
            .get_or_create_workspace(&std::path::PathBuf::from(&workspace_root_path))
            .await
            .map_err(|e| anyhow!("Failed to get/create admin workspace: {}", e))?;

        tracing::info!(
            "[SessionUseCase] Admin workspace: {} ({})",
            workspace.name,
            workspace.id
        );

        // 2. Update AppStateService to use this workspace
        self.app_state_service
            .set_last_selected_workspace(workspace.id.clone())
            .await
            .map_err(|e| anyhow!("Failed to set workspace selection: {}", e))?;

        // 3. Create session
        let session_id = Uuid::new_v4().to_string();
        tracing::debug!(
            "[SessionUseCase] Generated config session ID: {}",
            session_id
        );

        // Create InteractionManager using factory
        let manager = Arc::new(
            self.session_factory
                .create_interaction_manager(session_id.clone()),
        );

        // 4. Associate with admin workspace
        manager
            .set_workspace_id(
                Some(workspace.id.clone()),
                Some(workspace.root_path.clone()),
            )
            .await;

        // Insert into cache
        self.session_cache
            .insert(session_id.clone(), manager.clone())
            .await;

        tracing::info!(
            "[SessionUseCase] Config session {} associated with workspace {}",
            session_id,
            workspace.id
        );

        // 5. Add system prompt as a system message
        manager
            .add_system_conversation_message(
                system_prompt,
                Some("config_assistant".to_string()),
                None,
            )
            .await;

        tracing::info!("[SessionUseCase] System prompt added to config session");

        // 6. Persist session
        let session = self
            .session_factory
            .to_session(manager.as_ref(), AppMode::Idle, workspace.id.clone())
            .await;
        self.session_repository.save(&session).await?;

        // 7. Set as active session
        self.app_state_service
            .set_active_session(session_id.clone())
            .await
            .map_err(|e| anyhow!("Failed to set active session: {}", e))?;

        // 8. Return session
        Ok(session)
    }

    /// Switches to an existing session and restores its workspace context.
    ///
    /// This method implements UC2 (Session Switching):
    /// 1. Loads the session from storage
    /// 2. Validates the workspace association
    /// 3. Resolves and sets the workspace context
    /// 4. Updates workspace access timestamp and last active session
    /// 5. Handles orphaned workspace references
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to switch to
    ///
    /// # Returns
    ///
    /// Returns the switched session with workspace context restored.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The session does not exist
    /// - The session cannot be loaded
    ///
    /// # Note
    ///
    /// If the session references a non-existent workspace (orphaned reference),
    /// the workspace_id will be automatically cleared and a warning will be logged.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let session = usecase.switch_session("abc-123").await?;
    /// if session.workspace_id.is_none() {
    ///     println!("Warning: Session was orphaned, workspace cleared");
    /// }
    /// ```
    pub async fn switch_session(&self, session_id: &str) -> Result<Session> {
        tracing::info!("[SessionUseCase] Switching to session: {}", session_id);

        // 1. Get or load session
        let manager = if let Some(cached) = self.session_cache.get(session_id).await {
            cached
        } else {
            // Load from storage
            let session = self
                .session_repository
                .find_by_id(session_id)
                .await?
                .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
            let manager = Arc::new(self.session_factory.from_session(session));
            self.session_cache
                .insert(session_id.to_string(), manager.clone())
                .await;
            manager
        };

        // 2. Set as active session
        self.app_state_service
            .set_active_session(session_id.to_string())
            .await
            .map_err(|e| anyhow!("Failed to set active session: {}", e))?;

        // 3. Get session data to check workspace_id - use placeholder for now, will be updated
        let session = self
            .session_factory
            .to_session(
                manager.as_ref(),
                AppMode::Idle,
                PLACEHOLDER_WORKSPACE_ID.to_string(),
            )
            .await;
        let workspace_id = &session.workspace_id;

        // 4. Validate and restore workspace context if workspace_id exists
        if workspace_id != PLACEHOLDER_WORKSPACE_ID {
            tracing::debug!(
                "[SessionUseCase] Session references workspace: {}",
                workspace_id
            );

            match self
                .workspace_storage_service
                .get_workspace(workspace_id)
                .await
            {
                Ok(Some(mut workspace)) => {
                    // Valid workspace - restore context
                    tracing::info!(
                        "[SessionUseCase] Restoring workspace context: {} at {}",
                        workspace.name,
                        workspace.root_path.display()
                    );

                    manager
                        .set_workspace_id(
                            Some(workspace_id.clone()),
                            Some(workspace.root_path.clone()),
                        )
                        .await;

                    // Check if session is in sandbox mode - if so, override workspace root
                    let sandbox_state = manager.get_sandbox_state().await;
                    if let Some(sandbox) = sandbox_state {
                        use std::path::PathBuf;
                        tracing::info!(
                            "[SessionUseCase] Session is in sandbox mode, setting agent CWD to: {}",
                            sandbox.worktree_path
                        );
                        manager
                            .set_agent_workspace_root(Some(PathBuf::from(&sandbox.worktree_path)))
                            .await;
                    }

                    // Update workspace last active session
                    workspace.last_active_session_id = Some(session_id.to_string());
                    if let Err(e) = self
                        .workspace_storage_service
                        .save_workspace(&workspace)
                        .await
                    {
                        tracing::warn!(
                            "[SessionUseCase] Failed to save workspace last active session: {}",
                            e
                        );
                    }

                    // Update workspace access timestamp
                    if let Err(e) = self
                        .workspace_storage_service
                        .touch_workspace(workspace_id)
                        .await
                    {
                        tracing::warn!(
                            "[SessionUseCase] Failed to update workspace access time: {}",
                            e
                        );
                    }
                }
                Ok(None) => {
                    // Orphaned session - workspace was deleted
                    tracing::warn!(
                        "[SessionUseCase] Session {} references non-existent workspace {}",
                        session_id,
                        workspace_id
                    );

                    // Clear the invalid workspace_id
                    // Update in-memory cache if present
                    if let Some(cached_manager) = self.session_cache.get(session_id).await {
                        cached_manager
                            .set_workspace_id(Some(PLACEHOLDER_WORKSPACE_ID.to_string()), None)
                            .await;
                    }
                    // Update in storage using SessionUpdater
                    let updater = SessionUpdater::new(self.session_repository.clone());
                    updater
                        .update(session_id, |session| {
                            session.workspace_id = PLACEHOLDER_WORKSPACE_ID.to_string();
                            Ok(())
                        })
                        .await?;

                    tracing::info!(
                        "[SessionUseCase] Cleared orphaned workspace reference from session {}",
                        session_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "[SessionUseCase] Error checking workspace {}: {}",
                        workspace_id,
                        e
                    );
                    // Continue without workspace context
                }
            }
        } else {
            tracing::debug!("[SessionUseCase] Session has no workspace association");
        }

        // Return the session (potentially with cleared workspace_id)
        let final_session = self
            .session_factory
            .to_session(
                manager.as_ref(),
                AppMode::Idle,
                PLACEHOLDER_WORKSPACE_ID.to_string(),
            )
            .await;
        tracing::info!("[SessionUseCase] Switched to session: {}", session_id);

        Ok(final_session)
    }

    /// Switches the current session to a different workspace.
    ///
    /// This method implements UC5 (Workspace Switching):
    /// 1. Validates the target workspace exists
    /// 2. Checks if workspace has a last active session
    /// 3. If yes, switches to that session; if no, updates current session
    /// 4. Updates workspace access timestamp and last active session
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The ID of the workspace to switch to
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workspace does not exist
    /// - The update operation fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// usecase.switch_workspace("ws-456").await?;
    /// println!("Switched to workspace ws-456");
    /// ```
    pub async fn switch_workspace(&self, workspace_id: &str) -> Result<()> {
        println!("[SessionUseCase] Switching to workspace: {}", workspace_id);
        tracing::info!("[SessionUseCase] Switching to workspace: {}", workspace_id);

        // 0. Save to AppStateService (user's explicit selection)
        self.app_state_service
            .set_last_selected_workspace(workspace_id.to_string())
            .await
            .map_err(|e| anyhow!("Failed to save last selected workspace: {}", e))?;
        println!(
            "[SessionUseCase] Saved to AppStateService: {}",
            workspace_id
        );

        // 1. Validate workspace exists
        let mut workspace = self
            .workspace_storage_service
            .get_workspace(workspace_id)
            .await?
            .ok_or_else(|| anyhow!("Workspace not found: {}", workspace_id))?;

        println!(
            "[SessionUseCase] Target workspace: {} at {}",
            workspace.name,
            workspace.root_path.display()
        );
        println!(
            "[SessionUseCase] Workspace last_active_session_id: {:?}",
            workspace.last_active_session_id
        );
        tracing::debug!(
            "[SessionUseCase] Target workspace: {} at {}",
            workspace.name,
            workspace.root_path.display()
        );

        // 2. Find a session for this workspace
        // Priority: last_active_session_id > most recent session > create new session

        // Get all sessions for this workspace
        let all_sessions = self
            .session_repository
            .list_all()
            .await
            .map_err(|e| anyhow!("Failed to list sessions: {}", e))?;

        let workspace_sessions: Vec<_> = all_sessions
            .into_iter()
            .filter(|s| s.workspace_id == workspace.id)
            .collect();

        println!(
            "[SessionUseCase] Found {} sessions for workspace {}",
            workspace_sessions.len(),
            workspace_id
        );

        // Try last_active_session_id first
        if let Some(ref last_session_id) = workspace.last_active_session_id {
            if workspace_sessions.iter().any(|s| &s.id == last_session_id) {
                println!(
                    "[SessionUseCase] Using last active session: {}",
                    last_session_id
                );
                match self.switch_session(last_session_id).await {
                    Ok(_) => {
                        // Update session's workspace_id to the new workspace
                        if let Some(active_session_id) = self.active_session_id().await
                            && let Some(manager) = self.session_cache.get(&active_session_id).await
                        {
                            manager
                                .set_workspace_id(
                                    Some(workspace.id.clone()),
                                    Some(workspace.root_path.clone()),
                                )
                                .await;
                            // Persist the updated workspace association
                            let session = self
                                .session_factory
                                .to_session(
                                    manager.as_ref(),
                                    orcs_core::session::AppMode::Idle,
                                    workspace.id.clone(),
                                )
                                .await;
                            let _ = self.session_repository.save(&session).await;
                        }
                        println!(
                            "[SessionUseCase] Successfully switched to workspace {} with last active session {}",
                            workspace_id, last_session_id
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        println!(
                            "[SessionUseCase] Failed to switch to last active session: {}",
                            e
                        );
                        // Fall through to try other sessions
                    }
                }
            } else {
                println!(
                    "[SessionUseCase] Last active session {} not found in workspace sessions, ignoring",
                    last_session_id
                );
            }
        }

        // Try most recent session
        if !workspace_sessions.is_empty() {
            let mut sorted_sessions = workspace_sessions;
            sorted_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            let most_recent = &sorted_sessions[0];

            println!(
                "[SessionUseCase] Using most recent session: {}",
                most_recent.id
            );
            match self.switch_session(&most_recent.id).await {
                Ok(_) => {
                    // Update session's workspace_id to the new workspace
                    if let Some(active_session_id) = self.active_session_id().await
                        && let Some(manager) = self.session_cache.get(&active_session_id).await
                    {
                        manager
                            .set_workspace_id(
                                Some(workspace.id.clone()),
                                Some(workspace.root_path.clone()),
                            )
                            .await;
                        // Persist the updated workspace association
                        let session = self
                            .session_factory
                            .to_session(
                                manager.as_ref(),
                                orcs_core::session::AppMode::Idle,
                                workspace.id.clone(),
                            )
                            .await;
                        let _ = self.session_repository.save(&session).await;
                    }
                    println!(
                        "[SessionUseCase] Successfully switched to workspace {} with recent session {}",
                        workspace_id, most_recent.id
                    );
                    return Ok(());
                }
                Err(e) => {
                    println!("[SessionUseCase] Failed to switch to recent session: {}", e);
                    // Fall through to create new session
                }
            }
        }

        println!(
            "[SessionUseCase] No valid session found, creating new session for workspace {}",
            workspace_id
        );

        // 3. Create new session for this workspace
        let session_id = Uuid::new_v4().to_string();
        println!(
            "[SessionUseCase] Creating new session {} for workspace {}",
            session_id, workspace_id
        );

        // Create InteractionManager using factory
        let manager = Arc::new(
            self.session_factory
                .create_interaction_manager(session_id.clone()),
        );

        // Associate with workspace
        manager
            .set_workspace_id(
                Some(workspace.id.clone()),
                Some(workspace.root_path.clone()),
            )
            .await;

        // Insert into cache
        self.session_cache
            .insert(session_id.clone(), manager.clone())
            .await;

        // Persist session
        let session = self
            .session_factory
            .to_session(
                manager.as_ref(),
                orcs_core::session::AppMode::Idle,
                workspace_id.to_string(),
            )
            .await;
        self.session_repository.save(&session).await?;

        // Set as active session
        self.app_state_service
            .set_active_session(session_id.clone())
            .await
            .map_err(|e| anyhow!("Failed to set active session: {}", e))?;

        // Update workspace last active session
        workspace.last_active_session_id = Some(session_id.clone());
        self.workspace_storage_service
            .save_workspace(&workspace)
            .await?;
        self.workspace_storage_service
            .touch_workspace(workspace_id)
            .await?;

        println!(
            "[SessionUseCase] Successfully created and switched to new session {} for workspace {}",
            session_id, workspace_id
        );
        tracing::info!(
            "[SessionUseCase] Successfully created and switched to new session {} for workspace {}",
            session_id,
            workspace_id
        );

        Ok(())
    }

    /// Restores the last active session on application startup.
    ///
    /// This method implements UC6 (Session Restoration):
    /// 1. Attempts to restore the last active session
    /// 2. Validates the workspace reference
    /// 3. Restores workspace context if valid
    /// 4. Clears orphaned workspace references
    /// 5. Updates workspace access timestamp
    ///
    /// # Returns
    ///
    /// Returns `Some(Session)` if a session was restored, `None` if no active
    /// session was found.
    ///
    /// # Errors
    ///
    /// Returns an error if the restoration process fails.
    ///
    /// # Note
    ///
    /// This method automatically handles orphaned workspace references by
    /// clearing them and logging warnings.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match usecase.restore_last_session().await? {
    ///     Some(session) => println!("Restored session: {}", session.title),
    ///     None => println!("No previous session found"),
    /// }
    /// ```
    pub async fn restore_last_session(&self) -> Result<Option<Session>> {
        tracing::info!("[SessionUseCase] Attempting to restore last session");

        // 1. Get active session ID from state
        let Some(session_id) = self.app_state_service.get_active_session().await else {
            tracing::info!("[SessionUseCase] No active session ID found");
            return Ok(None);
        };

        // 2. Attempt to restore session
        let manager = if let Some(cached) = self.session_cache.get(&session_id).await {
            Some(cached)
        } else {
            // Load from storage
            if let Some(session) = self.session_repository.find_by_id(&session_id).await? {
                let manager = Arc::new(self.session_factory.from_session(session));
                self.session_cache
                    .insert(session_id.clone(), manager.clone())
                    .await;
                Some(manager)
            } else {
                None
            }
        };

        let Some(manager) = manager else {
            tracing::info!("[SessionUseCase] Session {} not found", session_id);
            // Clear invalid active session ID
            self.app_state_service
                .clear_active_session()
                .await
                .map_err(|e| anyhow!("Failed to clear active session: {}", e))?;
            return Ok(None);
        };

        // 3. Get session data - use placeholder for now, will be updated
        let session = self
            .session_factory
            .to_session(
                manager.as_ref(),
                AppMode::Idle,
                PLACEHOLDER_WORKSPACE_ID.to_string(),
            )
            .await;
        tracing::info!("[SessionUseCase] Restored session: {}", session.id);
        let workspace_id = &session.workspace_id;

        // 3. Validate and restore workspace context
        if workspace_id != PLACEHOLDER_WORKSPACE_ID {
            tracing::debug!(
                "[SessionUseCase] Session references workspace: {}",
                workspace_id
            );

            match self
                .workspace_storage_service
                .get_workspace(workspace_id)
                .await
            {
                Ok(Some(workspace)) => {
                    // Valid workspace - restore context
                    tracing::info!(
                        "[SessionUseCase] Restoring workspace context: {} at {}",
                        workspace.name,
                        workspace.root_path.display()
                    );

                    manager
                        .set_workspace_id(
                            Some(workspace_id.clone()),
                            Some(workspace.root_path.clone()),
                        )
                        .await;

                    // Update workspace access timestamp
                    if let Err(e) = self
                        .workspace_storage_service
                        .touch_workspace(workspace_id)
                        .await
                    {
                        tracing::warn!(
                            "[SessionUseCase] Failed to update workspace access time: {}",
                            e
                        );
                    }
                }
                Ok(None) => {
                    // Orphaned session - workspace was deleted
                    tracing::warn!(
                        "[SessionUseCase] Restored session {} references non-existent workspace {}",
                        session.id,
                        workspace_id
                    );

                    // Clear the invalid workspace_id
                    // Update in-memory cache if present
                    if let Some(cached_manager) = self.session_cache.get(&session.id).await {
                        cached_manager
                            .set_workspace_id(Some(PLACEHOLDER_WORKSPACE_ID.to_string()), None)
                            .await;
                    }
                    // Update in storage using SessionUpdater
                    let updater = SessionUpdater::new(self.session_repository.clone());
                    updater
                        .update(&session.id, |s| {
                            s.workspace_id = PLACEHOLDER_WORKSPACE_ID.to_string();
                            Ok(())
                        })
                        .await?;

                    tracing::info!(
                        "[SessionUseCase] Cleared orphaned workspace reference from restored session"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "[SessionUseCase] Error checking workspace {}: {}",
                        workspace_id,
                        e
                    );
                    // Continue without workspace context
                }
            }
        } else {
            tracing::debug!("[SessionUseCase] Restored session has no workspace association");
        }

        // Return the final session state
        let final_session = self
            .session_factory
            .to_session(
                manager.as_ref(),
                AppMode::Idle,
                PLACEHOLDER_WORKSPACE_ID.to_string(),
            )
            .await;
        Ok(Some(final_session))
    }

    /// Returns the ID of the currently active session.
    ///
    /// # Returns
    ///
    /// `Some(session_id)` if there is an active session, `None` otherwise.
    pub async fn active_session_id(&self) -> Option<String> {
        self.app_state_service.get_active_session().await
    }

    /// Returns the currently active session manager.
    ///
    /// # Returns
    ///
    /// `Some(manager)` if there is an active session, `None` otherwise.
    pub async fn active_session(&self) -> Option<Arc<InteractionManager>> {
        let session_id = (self.active_session_id().await)?;
        self.session_cache.get(&session_id).await
    }

    /// Saves the currently active session to storage.
    ///
    /// # Arguments
    ///
    /// * `app_mode` - The current application mode
    ///
    /// # Errors
    ///
    /// Returns an error if there is no active session or if storage fails.
    pub async fn save_active_session(&self, app_mode: AppMode) -> Result<()> {
        let Some(session_id) = self.active_session_id().await else {
            return Err(anyhow!("No active session"));
        };

        // Get manager from cache
        let manager = self
            .session_cache
            .get(&session_id)
            .await
            .ok_or_else(|| anyhow!("Active session {} not found in cache", session_id))?;

        // Load existing session to preserve workspace_id and last_memory_sync_at
        let existing_session = self
            .session_repository
            .list_all()
            .await?
            .into_iter()
            .find(|s| s.id == session_id);

        let existing_workspace_id = existing_session
            .as_ref()
            .map(|s| s.workspace_id.clone())
            .unwrap_or_else(|| PLACEHOLDER_WORKSPACE_ID.to_string());

        let existing_last_memory_sync_at = existing_session
            .as_ref()
            .and_then(|s| s.last_memory_sync_at.clone());

        // Convert to session and save
        let mut session = self
            .session_factory
            .to_session(manager.as_ref(), app_mode, existing_workspace_id.clone())
            .await;

        // Preserve last_memory_sync_at from existing session (to_session always sets it to None)
        session.last_memory_sync_at = existing_last_memory_sync_at;

        self.session_repository
            .save(&session)
            .await
            .map_err(|e| anyhow!("Failed to save session: {}", e))?;

        // Memory sync is now handled by the background scheduler (start_memory_sync_scheduler)
        // instead of being triggered on every save

        Ok(())
    }

    /// Collects messages from a session for memory sync.
    ///
    /// Only collects messages with timestamps after `last_memory_sync_at` for differential sync.
    /// If `last_memory_sync_at` is None, collects all messages (initial sync).
    fn collect_messages_for_sync(session: &Session) -> Vec<orcs_core::memory::MemoryMessage> {
        use chrono::DateTime;
        use orcs_core::memory::MemoryMessage;

        let mut messages = Vec::new();

        // Parse last sync timestamp for accurate comparison
        let last_sync_datetime = session
            .last_memory_sync_at
            .as_deref()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok());

        for (persona_id, history) in &session.persona_histories {
            for (idx, msg) in history.iter().enumerate() {
                // Skip system messages that shouldn't be synced
                if matches!(msg.role, orcs_core::session::MessageRole::System)
                    && !msg.metadata.include_in_dialogue
                {
                    continue;
                }

                // Skip empty content messages
                if msg.content.trim().is_empty() {
                    continue;
                }

                // Differential sync: only include messages after last sync timestamp
                // Use proper DateTime comparison instead of string comparison
                if let Some(last_sync) = last_sync_datetime {
                    if let Ok(msg_datetime) = DateTime::parse_from_rfc3339(&msg.timestamp) {
                        if msg_datetime <= last_sync {
                            continue;
                        }
                    }
                    // If parsing fails, fall back to including the message (safer than excluding)
                }

                messages.push(MemoryMessage {
                    // Include persona_id and index for unique ID to prevent duplicates
                    id: format!("{}-{}-{}", session.id, persona_id, idx),
                    session_id: session.id.clone(),
                    workspace_id: session.workspace_id.clone(),
                    role: format!("{:?}", msg.role),
                    content: msg.content.clone(),
                    timestamp: msg.timestamp.clone(),
                    persona_id: Some(persona_id.clone()),
                    tags: vec![
                        format!("session:{}", session.id),
                        format!("workspace:{}", session.workspace_id),
                    ],
                });
            }
        }

        messages
    }

    /// Deletes a session and clears active session if it was the active one.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the session deletion fails.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        // Remove from cache
        self.session_cache.remove(session_id).await;

        // Remove from storage
        self.session_repository.delete(session_id).await?;

        // Clear active session if this was the active session
        if let Some(active_session_id) = self.active_session_id().await
            && active_session_id == session_id
        {
            self.app_state_service
                .clear_active_session()
                .await
                .map_err(|e| anyhow!("Failed to clear active session: {}", e))?;
        }

        Ok(())
    }

    /// Returns a reference to the workspace manager.
    ///
    /// This provides direct access to the underlying workspace manager for
    /// workspace-only operations.
    pub fn workspace_storage_service(&self) -> &Arc<dyn WorkspaceStorageService> {
        &self.workspace_storage_service
    }

    /// Enriches session participants field by resolving persona IDs to names.
    ///
    /// For sessions loaded from storage (especially after migration from older versions),
    /// the participants HashMap may be empty. This method populates it by:
    /// 1. Adding user name mapping (user_name -> user_name)
    /// 2. Resolving persona IDs from persona_histories keys to persona names
    ///
    /// # Arguments
    ///
    /// * `session` - The session to enrich
    ///
    /// # Returns
    ///
    /// The enriched session with populated participants field.
    pub async fn enrich_session_participants(&self, mut session: Session) -> Session {
        use std::collections::HashMap;

        // If participants is already populated, return as-is
        if !session.participants.is_empty() {
            return session;
        }

        let mut participants = HashMap::new();

        // Add user name
        let user_name = self.user_service.get_user_name();
        participants.insert(user_name.clone(), user_name);

        // Resolve persona IDs to names
        if let Ok(all_personas) = self.persona_repository.get_all().await {
            for persona_id in session.persona_histories.keys() {
                if let Some(persona) = all_personas.iter().find(|p| &p.id == persona_id) {
                    participants.insert(persona_id.clone(), persona.name.clone());
                }
            }
        }

        session.participants = participants;
        session
    }

    /// Adds a system message to the active session.
    ///
    /// This method is part of the refactored message handling architecture where
    /// business logic is centralized in SessionUseCase. It delegates to the
    /// InteractionManager to actually add the message.
    ///
    /// # Arguments
    ///
    /// * `content` - The message content
    /// * `message_type` - Optional message type (e.g., "context_info", "notification")
    /// * `severity` - Optional error severity
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if no active session exists.
    ///
    /// # Note
    ///
    /// This method does NOT save the session. The caller (Tauri layer) is responsible
    /// for calling `session_manager.save_active_session(app_mode)` after this method
    /// returns. This separation allows the Tauri layer to manage app_mode.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // In Tauri layer
    /// session_usecase.add_system_message(
    ///     "Command executed successfully".to_string(),
    ///     Some("context_info".to_string()),
    ///     None,
    /// ).await?;
    ///
    /// // Then save the session
    /// let app_mode = state.app_mode.lock().await.clone();
    /// state.session_manager.save_active_session(app_mode).await?;
    /// ```
    pub async fn add_system_message(
        &self,
        content: String,
        message_type: Option<String>,
        severity: Option<orcs_core::session::ErrorSeverity>,
    ) -> Result<()> {
        let Some(session_id) = self.active_session_id().await else {
            return Err(anyhow!("No active session"));
        };

        let manager = self
            .session_cache
            .get(&session_id)
            .await
            .ok_or_else(|| anyhow!("Active session {} not found in cache", session_id))?;

        manager
            .add_system_conversation_message(content, message_type, severity)
            .await;

        Ok(())
    }
}
