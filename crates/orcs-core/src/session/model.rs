//! Session domain model.
//!
//! This module contains the core Session entity that represents
//! a user session in the application's domain layer.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use orcs_types::{ConversationMessage, AppMode};

/// Represents a user session in the application's domain layer.
///
/// A session contains:
/// - Conversation history for each participating persona
/// - The currently active persona
/// - Application mode (Idle, Planning, etc.)
/// - Timestamps for creation and last update
///
/// This is the "pure" domain model that business logic operates on,
/// independent of any specific storage format or version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier (UUID format)
    pub id: String,
    /// Human-readable session title
    pub title: String,
    /// Timestamp when the session was created (ISO 8601 format)
    pub created_at: String,
    /// Timestamp when the session was last updated (ISO 8601 format)
    pub updated_at: String,
    /// The currently active persona ID (UUID format)
    pub current_persona_id: String,
    /// Conversation history for each persona (keyed by persona ID)
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    /// Current application mode
    pub app_mode: AppMode,
}
