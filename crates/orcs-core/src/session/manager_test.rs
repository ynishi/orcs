#[cfg(test)]
mod tests {
    use crate::session::app_mode::{AppMode, ConversationMode};
    use crate::session::manager::{InteractionManagerTrait, SessionManager};
    use crate::session::model::Session;
    use crate::session::repository::SessionRepository;
    use crate::state::repository::StateRepository;
    use llm_toolkit::agent::dialogue::ExecutionModel;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock InteractionManager for testing
    struct MockInteractionManager {
        session_id: String,
    }

    impl MockInteractionManager {
        fn new(session_id: String) -> Self {
            Self { session_id }
        }

        fn from_data(data: Session) -> Self {
            Self {
                session_id: data.id,
            }
        }
    }

    impl InteractionManagerTrait for MockInteractionManager {
        fn session_id(&self) -> &str {
            &self.session_id
        }

        async fn to_session(&self, app_mode: AppMode, workspace_id: String) -> Session {
            Session {
                id: self.session_id.clone(),
                title: format!("Session {}", self.session_id),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                current_persona_id: "mai".to_string(),
                persona_histories: HashMap::new(),
                app_mode,
                workspace_id,
                active_participant_ids: Vec::new(),
                execution_strategy: ExecutionModel::Broadcast,
                system_messages: Vec::new(),
                participants: HashMap::new(),
                participant_icons: HashMap::new(),
                conversation_mode: ConversationMode::Normal,
                talk_style: None,
                participant_colors: HashMap::new(),
                is_favorite: false,
                is_archived: false,
                sort_order: None,
                auto_chat_config: None,
            }
        }

        async fn set_workspace_id(
            &self,
            _workspace_id: Option<String>,
            _workspace_root: Option<std::path::PathBuf>,
        ) {
            // Mock implementation - no-op
        }
    }

    // Mock SessionRepository for testing
    struct MockSessionRepository {
        sessions: Mutex<HashMap<String, Session>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl SessionRepository for MockSessionRepository {
        async fn find_by_id(&self, session_id: &str) -> crate::error::Result<Option<Session>> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.get(session_id).cloned())
        }

        async fn save(&self, session: &Session) -> crate::error::Result<()> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session.id.clone(), session.clone());
            Ok(())
        }

        async fn delete(&self, session_id: &str) -> crate::error::Result<()> {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(session_id);
            Ok(())
        }

        async fn list_all(&self) -> crate::error::Result<Vec<Session>> {
            let sessions = self.sessions.lock().unwrap();
            Ok(sessions.values().cloned().collect())
        }
    }

    // Mock StateRepository for testing
    struct MockStateRepository {
        active_session_id: Mutex<Option<String>>,
    }

    impl MockStateRepository {
        fn new() -> Self {
            Self {
                active_session_id: Mutex::new(None),
            }
        }
    }

    #[async_trait::async_trait]
    impl StateRepository for MockStateRepository {
        async fn save_state(&self, _state: crate::state::model::AppState) -> crate::error::Result<()> {
            Ok(())
        }

        async fn get_state(&self) -> crate::error::Result<crate::state::model::AppState> {
            Ok(crate::state::model::AppState::default())
        }

        async fn get_last_selected_workspace(&self) -> Option<String> {
            None
        }

        async fn set_last_selected_workspace(&self, _workspace_id: String) -> crate::error::Result<()> {
            Ok(())
        }

        async fn clear_last_selected_workspace(&self) -> crate::error::Result<()> {
            Ok(())
        }

        async fn get_default_workspace(&self) -> String {
            crate::state::model::PLACEHOLDER_DEFAULT_WORKSPACE_ID.to_string()
        }

        async fn set_default_workspace(&self, _workspace_id: String) -> crate::error::Result<()> {
            Ok(())
        }

        async fn get_active_session(&self) -> Option<String> {
            self.active_session_id.lock().unwrap().clone()
        }

        async fn set_active_session(&self, session_id: String) -> crate::error::Result<()> {
            *self.active_session_id.lock().unwrap() = Some(session_id);
            Ok(())
        }

        async fn clear_active_session(&self) -> crate::error::Result<()> {
            *self.active_session_id.lock().unwrap() = None;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        let _session = manager
            .create_session("test-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        assert_eq!(
            manager.active_session_id().await,
            Some("test-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository.clone(), state_repository.clone());

        // Create and save
        let _session = manager
            .create_session("test-save".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        // Create new manager and restore
        let manager2: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        let restored = manager2
            .restore_last_session(MockInteractionManager::from_data)
            .await
            .unwrap();

        assert!(restored.is_some());
        assert_eq!(restored.unwrap().session_id(), "test-save");
    }

    #[tokio::test]
    async fn test_switch_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        // Create two sessions
        manager
            .create_session("session-1".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.save_active_session(AppMode::Idle).await.unwrap();

        manager
            .create_session("session-2".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        // Active should be session-2
        assert_eq!(
            manager.active_session_id().await,
            Some("session-2".to_string())
        );

        // Switch back to session-1
        manager
            .switch_session("session-1", MockInteractionManager::from_data)
            .await
            .unwrap();

        assert_eq!(
            manager.active_session_id().await,
            Some("session-1".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_session() {
        let session_repository = Arc::new(MockSessionRepository::new());
        let state_repository = Arc::new(MockStateRepository::new());
        let manager: SessionManager<MockInteractionManager> =
            SessionManager::new(session_repository, state_repository);

        manager
            .create_session("to-delete".to_string(), MockInteractionManager::new)
            .await
            .unwrap();

        manager.delete_session("to-delete").await.unwrap();

        assert_eq!(manager.active_session_id().await, None);
    }
}

