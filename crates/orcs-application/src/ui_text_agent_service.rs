//! UI Text Agent Service
//!
//! Provides AI-powered text generation and refinement for UI components
//! like text fields, forms, and content editors using Gemini Flash API.

use anyhow::Result;
use llm_toolkit::ToPrompt;
use llm_toolkit::agent::Agent;
use serde::{Deserialize, Serialize};

/// Context information for AI text operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiTextContext {
    /// Scope of the content (e.g., "UserProfile.Bio", "MessageItem")
    pub scope: String,
    /// Type of content (e.g., "string", "long_text", "markdown", "code")
    pub content_type: String,
    /// Maximum length constraint (optional)
    pub max_length: Option<usize>,
    /// Additional metadata for context (optional)
    pub metadata: Option<serde_json::Value>,
}

/// Request for generating UI text content
#[derive(Debug, Clone, Serialize, ToPrompt)]
#[prompt(
    mode = "full",
    template = r#"You are a text generation assistant for a UI application.

Scope: {{ scope }}
Content type: {{ content_type }}
{% if max_length -%}
Maximum length: {{ max_length }} characters
{% endif %}

User instruction: {{ user_prompt }}

Guidelines:
- Generate natural, contextually appropriate text
- Match the tone to the content type and scope
- Be concise and clear
- Provide only the generated text without explanations
{% if content_type == "code" -%}
- Output valid code without markdown code blocks
{% endif %}

Generate the text now:"#
)]
struct UiTextGenerationRequest {
    scope: String,
    content_type: String,
    max_length: Option<usize>,
    user_prompt: String,
}

/// Request for refining existing UI text content
#[derive(Debug, Clone, Serialize, ToPrompt)]
#[prompt(
    mode = "full",
    template = r#"You are a text refinement assistant for a UI application.

Scope: {{ scope }}
Content type: {{ content_type }}
{% if max_length -%}
Maximum length: {{ max_length }} characters
{% endif %}

Current text:
{{ current_text }}

Refinement instruction: {{ user_prompt }}

Guidelines:
- Refine the text according to the user's instruction
- Maintain the original meaning and context
- Improve clarity, grammar, and style as needed
- Respect length constraints if specified
- Provide only the refined text without explanations
{% if content_type == "code" -%}
- Output valid code without markdown code blocks
{% endif %}

Provide the refined text now:"#
)]
struct UiTextRefinementRequest {
    scope: String,
    content_type: String,
    max_length: Option<usize>,
    current_text: String,
    user_prompt: String,
}

/// Lightweight agent for generating UI text content
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Generate natural, contextually appropriate text for UI components. Focus on clarity, conciseness, and matching the expected tone.",
    output = "String",
    inner = "orcs_interaction::GeminiApiAgent"
)]
struct UiTextGeneratorAgent;

/// Lightweight agent for refining UI text content
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Refine text for UI components while maintaining meaning and context. Improve clarity, grammar, and style.",
    output = "String",
    inner = "orcs_interaction::GeminiApiAgent"
)]
struct UiTextRefinerAgent;

/// Service providing AI-powered text operations for UI components
pub struct UiTextAgentService {
    generator: UiTextGeneratorAgent,
    refiner: UiTextRefinerAgent,
}

impl UiTextAgentService {
    /// Creates a new UiTextAgentService instance
    pub fn new() -> Self {
        Self {
            generator: UiTextGeneratorAgent,
            refiner: UiTextRefinerAgent,
        }
    }

    /// Generate text content based on user prompt and context
    ///
    /// # Arguments
    ///
    /// * `prompt` - User instruction for text generation
    /// * `context` - Context information about the target UI field
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated text content
    /// * `Err(anyhow::Error)` - If generation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let service = UiTextAgentService::new();
    /// let context = UiTextContext {
    ///     scope: "UserProfile.Bio".to_string(),
    ///     content_type: "long_text".to_string(),
    ///     max_length: Some(500),
    ///     metadata: None,
    /// };
    /// let result = service.generate("Write a professional bio for a software engineer", context).await?;
    /// ```
    pub async fn generate(&self, prompt: impl Into<String>, context: UiTextContext) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        let request = UiTextGenerationRequest {
            scope: context.scope.clone(),
            content_type: context.content_type.clone(),
            max_length: context.max_length,
            user_prompt: prompt.into(),
        };

        tracing::info!(
            "[UiTextAgentService] Generating text for scope: {}, type: {}",
            context.scope,
            context.content_type
        );

        // Generate prompt using ToPrompt derive
        let generated_prompt = request.to_prompt();

        // Execute with generated prompt
        let result: String = self.generator.execute(generated_prompt.as_str().into()).await?;
        let trimmed = result.trim().to_string();

        tracing::info!(
            "[UiTextAgentService] Generated {} characters for scope: {}",
            trimmed.len(),
            context.scope
        );

        Ok(trimmed)
    }

    /// Refine existing text content based on user prompt and context
    ///
    /// # Arguments
    ///
    /// * `prompt` - User instruction for text refinement
    /// * `current_text` - The existing text to be refined
    /// * `context` - Context information about the target UI field
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Refined text content
    /// * `Err(anyhow::Error)` - If refinement fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let service = UiTextAgentService::new();
    /// let context = UiTextContext {
    ///     scope: "MessageItem".to_string(),
    ///     content_type: "string".to_string(),
    ///     max_length: Some(200),
    ///     metadata: None,
    /// };
    /// let result = service.refine(
    ///     "make it more concise",
    ///     "This is a very long message that needs to be shortened...",
    ///     context
    /// ).await?;
    /// ```
    pub async fn refine(
        &self,
        prompt: impl Into<String>,
        current_text: impl Into<String>,
        context: UiTextContext,
    ) -> Result<String> {
        use llm_toolkit::prompt::ToPrompt;

        let current = current_text.into();
        let request = UiTextRefinementRequest {
            scope: context.scope.clone(),
            content_type: context.content_type.clone(),
            max_length: context.max_length,
            current_text: current.clone(),
            user_prompt: prompt.into(),
        };

        tracing::info!(
            "[UiTextAgentService] Refining text for scope: {}, current length: {}",
            context.scope,
            current.len()
        );

        // Generate prompt using ToPrompt derive
        let generated_prompt = request.to_prompt();

        // Execute with generated prompt
        let result: String = self.refiner.execute(generated_prompt.as_str().into()).await?;
        let trimmed = result.trim().to_string();

        tracing::info!(
            "[UiTextAgentService] Refined from {} to {} characters for scope: {}",
            current.len(),
            trimmed.len(),
            context.scope
        );

        Ok(trimmed)
    }
}

impl Default for UiTextAgentService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_text_context_creation() {
        let context = UiTextContext {
            scope: "Test.Field".to_string(),
            content_type: "string".to_string(),
            max_length: Some(100),
            metadata: None,
        };

        assert_eq!(context.scope, "Test.Field");
        assert_eq!(context.content_type, "string");
        assert_eq!(context.max_length, Some(100));
    }

    #[test]
    fn test_service_creation() {
        let service = UiTextAgentService::new();
        // Just verify it can be created
        let _default_service = UiTextAgentService::default();
    }
}
