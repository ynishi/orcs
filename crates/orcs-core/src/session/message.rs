//! Conversation message types.
//!
//! This module contains types for representing messages in a conversation,
//! including roles and message content.

use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};
use version_migrate::DeriveQueryable as Queryable;

/// Represents the role of a message in a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
pub enum MessageRole {
    /// Message from the user.
    User,
    /// Message from the AI assistant.
    Assistant,
    /// System-generated message.
    System,
}

/// Type of system event being recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum SystemEventType {
    /// A participant joined the conversation.
    ParticipantJoined,
    /// A participant left the conversation.
    ParticipantLeft,
    /// Execution strategy was changed (broadcast/sequential).
    ExecutionStrategyChanged,
    /// Application mode changed (idle/planning/etc).
    ModeChanged,
    /// Workspace was switched.
    WorkspaceSwitched,
    /// Generic system notification.
    Notification,
}

/// Severity level for error messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ErrorSeverity {
    /// Critical error - show in thread + toast.
    Critical,
    /// Warning - show in thread only.
    Warning,
    /// Info - toast only, not in thread.
    Info,
}

/// Metadata for conversation messages.
///
/// This provides additional context about the message that helps
/// the frontend determine how to display it and helps agents
/// understand the conversation context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, SchemaBridge)]
#[serde(rename_all = "camelCase")]
pub struct MessageMetadata {
    /// For System messages: the type of system event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_event_type: Option<SystemEventType>,

    /// For System messages with errors: the severity level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_severity: Option<ErrorSeverity>,

    /// Optional UI message type hint (e.g., command, shell_output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message_type: Option<String>,

    /// Whether this message should be included in agent dialogue context.
    /// Defaults to true for backward compatibility.
    #[serde(default = "default_true")]
    pub include_in_dialogue: bool,
}

fn default_true() -> bool {
    true
}

/// A single message in a conversation history.
///
/// Each message has a role (user, assistant, or system), content,
/// and a timestamp indicating when it was created.
///
/// Version 2 adds metadata field for extended information.
/// Version 3 adds attachments field for file attachments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, SchemaBridge)]
#[serde(rename_all = "camelCase")]
#[queryable(entity = "conversation_message")]
pub struct ConversationMessage {
    /// The role of the message sender.
    pub role: MessageRole,
    /// The content of the message.
    pub content: String,
    /// Timestamp when the message was created (ISO 8601 format).
    pub timestamp: String,
    /// Additional metadata about the message.
    #[serde(default)]
    pub metadata: MessageMetadata,
    /// Attached files (file paths in workspace).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<String>,
}
