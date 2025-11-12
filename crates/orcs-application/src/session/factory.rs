use orcs_core::repository::PersonaRepository;
use orcs_core::session::{AppMode, Session};
use orcs_core::user::UserService;
use orcs_interaction::InteractionManager;
use std::sync::Arc;

/// Factory for creating InteractionManager instances from Session data.
///
/// This factory handles the conversion between Session (persistent data)
/// and InteractionManager (runtime state).
pub struct SessionFactory {
    /// Repository for persona configurations
    persona_repository: Arc<dyn PersonaRepository>,
    /// Service for user information
    user_service: Arc<dyn UserService>,
}

impl SessionFactory {
    /// Creates a new SessionFactory.
    ///
    /// # Arguments
    ///
    /// * `persona_repository` - Repository for accessing persona configurations
    /// * `user_service` - Service for retrieving user information
    pub fn new(
        persona_repository: Arc<dyn PersonaRepository>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            persona_repository,
            user_service,
        }
    }

    /// Creates a new InteractionManager for a new session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the new session
    ///
    /// # Returns
    ///
    /// A new InteractionManager instance.
    pub fn create_interaction_manager(&self, session_id: String) -> InteractionManager {
        InteractionManager::new_session(
            session_id,
            self.persona_repository.clone(),
            self.user_service.clone(),
        )
    }

    /// Creates an InteractionManager from Session data.
    ///
    /// # Arguments
    ///
    /// * `session` - The session data to load
    ///
    /// # Returns
    ///
    /// An InteractionManager instance restored from the session data.
    pub fn from_session(&self, session: Session) -> InteractionManager {
        InteractionManager::from_session(
            session,
            self.persona_repository.clone(),
            self.user_service.clone(),
        )
    }

    /// Converts an InteractionManager to Session data.
    ///
    /// # Arguments
    ///
    /// * `manager` - The InteractionManager to convert
    /// * `app_mode` - The current application mode
    /// * `workspace_id` - The workspace ID associated with the session
    ///
    /// # Returns
    ///
    /// Session data representing the current state of the InteractionManager.
    pub async fn to_session(
        &self,
        manager: &InteractionManager,
        app_mode: AppMode,
        workspace_id: String,
    ) -> Session {
        manager.to_session(app_mode, workspace_id).await
    }
}
