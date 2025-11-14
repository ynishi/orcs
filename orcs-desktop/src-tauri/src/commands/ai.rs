//! AI utility commands for text generation and refinement
//!
//! Provides thin Tauri command proxies that delegate to the application layer
//! UiTextAgentService for AI-powered text operations.

use orcs_application::{UiTextAgentService, UiTextContext};
use serde::{Deserialize, Serialize};

/// Context information for AI operations (Tauri IPC compatible)
///
/// This is a thin wrapper that maps between Tauri's camelCase JSON and
/// the application layer's UiTextContext.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIContext {
    /// Scope of the content (e.g., "UserProfile.Bio", "MessageItem")
    pub scope: String,
    /// Type of content (e.g., "string", "long_text", "markdown", "code")
    #[serde(rename = "type")]
    pub content_type: String,
    /// Maximum length constraint (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Additional metadata for context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl From<AIContext> for UiTextContext {
    fn from(ctx: AIContext) -> Self {
        UiTextContext {
            scope: ctx.scope,
            content_type: ctx.content_type,
            max_length: ctx.max_length,
            metadata: ctx.metadata,
        }
    }
}

/// Generates text content based on a prompt and context.
///
/// Thin proxy command that delegates to UiTextAgentService in the application layer.
///
/// # Arguments
///
/// * `prompt` - The instruction or content direction for generation
/// * `context` - Context information about the target field
///
/// # Returns
///
/// Generated text content as a String
///
/// # Errors
///
/// Returns error if AI generation fails or configuration is missing
#[tauri::command]
pub async fn ai_generate(
    prompt: String,
    context: AIContext,
) -> Result<String, String> {
    let service = UiTextAgentService::new();
    let ui_context: UiTextContext = context.into();

    service
        .generate(prompt, ui_context)
        .await
        .map_err(|e| e.to_string())
}

/// Refines existing text content based on a prompt and context.
///
/// Thin proxy command that delegates to UiTextAgentService in the application layer.
///
/// # Arguments
///
/// * `prompt` - The refinement instruction (e.g., "make it more concise")
/// * `current_text` - The existing text to be refined
/// * `context` - Context information about the target field
///
/// # Returns
///
/// Refined text content as a String
///
/// # Errors
///
/// Returns error if AI refinement fails or configuration is missing
#[tauri::command]
pub async fn ai_refine(
    prompt: String,
    current_text: String,
    context: AIContext,
) -> Result<String, String> {
    let service = UiTextAgentService::new();
    let ui_context: UiTextContext = context.into();

    service
        .refine(prompt, current_text, ui_context)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_context_conversion() {
        let ai_ctx = AIContext {
            scope: "Test.Field".to_string(),
            content_type: "string".to_string(),
            max_length: Some(100),
            metadata: None,
        };

        let ui_ctx: UiTextContext = ai_ctx.into();

        assert_eq!(ui_ctx.scope, "Test.Field");
        assert_eq!(ui_ctx.content_type, "string");
        assert_eq!(ui_ctx.max_length, Some(100));
    }
}
