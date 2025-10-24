use orcs_core::config::PersonaConfig;
use orcs_core::repository::{PersonaRepository, SessionRepository};
use orcs_core::session::Session;
use orcs_core::session_storage::SessionStorage;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::toml_storage;

/// A repository implementation for storing persona configurations in a TOML file.
pub struct TomlPersonaRepository;

impl PersonaRepository for TomlPersonaRepository {
    fn get_all(&self) -> Result<Vec<PersonaConfig>, String> {
        toml_storage::load_personas()
    }

    fn save_all(&self, configs: &[PersonaConfig]) -> Result<(), String> {
        toml_storage::save_personas(configs)
    }
}

/// A repository implementation for storing session data using SessionStorage.
///
/// This implementation wraps the `SessionStorage` and implements the `SessionRepository` trait,
/// providing an async interface for session persistence operations.
pub struct TomlSessionRepository {
    storage: Arc<SessionStorage>,
}

impl TomlSessionRepository {
    /// Creates a new `TomlSessionRepository` with the given storage.
    ///
    /// # Arguments
    ///
    /// * `storage` - The `SessionStorage` instance to use for persistence
    pub fn new(storage: SessionStorage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }
}

#[async_trait]
impl SessionRepository for TomlSessionRepository {
    async fn find_by_id(&self, session_id: &str) -> Result<Option<Session>> {
        match self.storage.load_session(session_id) {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                // Check if it's a "not found" error
                if let Some(io_error) = e.downcast_ref::<std::io::Error>() {
                    if io_error.kind() == std::io::ErrorKind::NotFound {
                        return Ok(None);
                    }
                }
                Err(e)
            }
        }
    }

    async fn save(&self, session: &Session) -> Result<()> {
        self.storage.save_session(session)
    }

    async fn delete(&self, session_id: &str) -> Result<()> {
        self.storage.delete_session(session_id)
    }

    async fn list_all(&self) -> Result<Vec<Session>> {
        self.storage.list_sessions()
    }

    async fn get_active_session_id(&self) -> Result<Option<String>> {
        self.storage.load_active_session_id()
    }

    async fn set_active_session_id(&self, session_id: &str) -> Result<()> {
        self.storage.save_active_session_id(session_id)
    }
}
