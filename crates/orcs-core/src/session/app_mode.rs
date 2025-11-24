//! Application mode types for session state management.

use schema_bridge::SchemaBridge;
use serde::{Deserialize, Serialize};

/// Represents a proposed plan of action for user confirmation.
///
/// Plans are generated when the system needs to propose a series of steps
/// and wait for user approval before proceeding with execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
pub struct Plan {
    /// The individual steps that make up this plan.
    pub steps: Vec<String>,
}

/// Represents the current interaction mode within a session.
///
/// This tracks whether the session is in normal chat mode or waiting for
/// human-in-the-loop (HIL) confirmation of a proposed plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SchemaBridge)]
#[serde(tag = "type", content = "data")]
pub enum AppMode {
    /// The application is waiting for user input in a normal chat mode.
    Idle,
    /// The application has proposed a plan and is waiting for user confirmation.
    AwaitingConfirmation {
        /// The plan awaiting confirmation.
        plan: Plan,
    },
}

/// Controls the verbosity and style of conversation in multi-agent dialogues.
///
/// This mode affects how AI agents respond to each other, preventing the
/// "escalation" problem where each agent tries to be more verbose than
/// the previous one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ConversationMode {
    /// Detailed mode - comprehensive responses with thorough explanations.
    /// Agents provide in-depth analysis and detailed reasoning.
    Detailed,

    /// Normal mode - no special constraints (default).
    #[default]
    Normal,

    /// Concise mode - responses should be under 300 characters.
    /// Agents avoid repeating what others have already said.
    Concise,

    /// Brief mode - responses should be under 150 characters.
    /// Only the most essential points.
    Brief,

    /// Discussion mode - focus on new perspectives only.
    /// Agents avoid elaborating on points already covered by others.
    Discussion,
}

impl ConversationMode {
    /// Returns the system instruction for this mode, if any.
    pub fn system_instruction(&self) -> Option<&'static str> {
        match self {
            Self::Detailed => Some(
                "詳細モード: 包括的な説明と深い分析を提供。背景、理由、影響を含めて詳しく回答。",
            ),
            Self::Normal => None,
            Self::Concise => Some(
                "重要: 応答は簡潔に300文字以内で。他の参加者が既に述べた内容は繰り返さない。新しい視点のみ追加。",
            ),
            Self::Brief => Some("超重要: 応答は150文字以内。要点のみ簡潔に。"),
            Self::Discussion => {
                Some("議論モード: 他の参加者の意見に重複しない新しい視点のみ追加。簡潔に述べる。")
            }
        }
    }
}
