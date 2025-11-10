use serde::{Deserialize, Serialize};

use super::{ConversationMode, ErrorSeverity};

/// High-level events that can be published to a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEvent {
    /// User-submitted input (text + optional attachments).
    UserInput {
        content: String,
        #[serde(default)]
        attachments: Vec<String>,
    },
    /// System-side message that should be persisted.
    SystemEvent {
        content: String,
        #[serde(default)]
        message_type: Option<String>,
        #[serde(default)]
        severity: Option<ErrorSeverity>,
    },
    /// Action produced by an internal moderator.
    ModeratorAction {
        action: ModeratorAction,
    },
}

/// Actions the moderator can trigger within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModeratorAction {
    /// Update conversation mode.
    SetConversationMode {
        mode: ConversationMode,
    },
    /// Append a structured system message.
    AppendSystemMessage {
        content: String,
        #[serde(default)]
        message_type: Option<String>,
        #[serde(default)]
        severity: Option<ErrorSeverity>,
    },
}
