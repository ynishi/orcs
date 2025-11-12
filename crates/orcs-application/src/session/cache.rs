use orcs_core::session::InteractionManagerTrait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory cache for InteractionManager instances.
///
/// This cache stores loaded InteractionManager instances to avoid
/// repeated deserialization and reconstruction from Session data.
pub struct SessionCache<T: InteractionManagerTrait> {
    /// In-memory session cache
    sessions: Arc<RwLock<HashMap<String, Arc<T>>>>,
}

impl<T: InteractionManagerTrait> SessionCache<T> {
    /// Creates a new empty SessionCache.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets a cached InteractionManager by session ID.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to retrieve
    ///
    /// # Returns
    ///
    /// `Some(manager)` if the session is cached, `None` otherwise.
    pub async fn get(&self, session_id: &str) -> Option<Arc<T>> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Inserts an InteractionManager into the cache.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session
    /// * `manager` - The InteractionManager to cache
    pub async fn insert(&self, session_id: String, manager: Arc<T>) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id, manager);
    }

    /// Removes an InteractionManager from the cache.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to remove
    pub async fn remove(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
    }

    /// Clears all cached sessions.
    pub async fn clear(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.clear();
    }
}

impl<T: InteractionManagerTrait> Default for SessionCache<T> {
    fn default() -> Self {
        Self::new()
    }
}
