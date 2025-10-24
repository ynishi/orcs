//! Conversation message types.
//!
//! This module contains types for representing messages in a conversation,
//! including roles and message content.

use serde::{Deserialize, Serialize};

/// Represents the role of a message in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// Message from the user.
    User,
    /// Message from the AI assistant.
    Assistant,
    /// System-generated message.
    System,
}

/// A single message in a conversation history.
///
/// Each message has a role (user, assistant, or system), content,
/// and a timestamp indicating when it was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The content of the message.
    pub content: String,
    /// Timestamp when the message was created (ISO 8601 format).
    pub timestamp: String,
}
