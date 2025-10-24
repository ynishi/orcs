//! Session domain model.
//!
//! This module contains the core Session entity that represents
//! a user session in the application's domain layer.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use orcs_types::{ConversationMessage, AppMode};

/// Represents the session concept in the application's core logic.
/// This is the "pure" model that the business logic layer operates on.
/// It is independent of any specific storage format or version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub current_persona_id: String,
    pub persona_histories: HashMap<String, Vec<ConversationMessage>>,
    pub app_mode: AppMode,
}
