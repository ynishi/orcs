//! AI utility commands for text generation and refinement
//!
//! Provides Tauri commands for generating and refining text using AI models.
//! These commands are designed for UI text fields and content editing features.

use orcs_interaction::gemini_api_agent::GeminiApiAgent;
use llm_toolkit::agent::{Agent, Payload};
use llm_toolkit::agent::dialogue::Speaker;
use serde::{Deserialize, Serialize};

/// Context information for AI operations
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

/// Generates text content based on a prompt and context.
///
/// This command uses AI to generate text suitable for various UI contexts
/// like user profiles, message fields, or form inputs.
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
    tracing::info!(
        "[ai_generate] Generating content for scope: {}, type: {}",
        context.scope,
        context.content_type
    );

    // Create AI agent
    let agent = GeminiApiAgent::try_from_env()
        .await
        .map_err(|e| format!("Failed to initialize AI agent: {}", e))?;

    // Build system instruction based on context
    let system_instruction = build_generation_instruction(&context);
    let agent = agent.with_system_instruction(system_instruction);

    // Execute generation
    let speaker = Speaker::user("User", "User");
    let payload = Payload::new().with_message(speaker, &prompt);
    let result = agent
        .execute(payload)
        .await
        .map_err(|e| format!("AI generation failed: {}", e))?;

    tracing::info!(
        "[ai_generate] Generated {} characters for scope: {}",
        result.len(),
        context.scope
    );

    Ok(result.trim().to_string())
}

/// Refines existing text content based on a prompt and context.
///
/// This command uses AI to improve, modify, or enhance existing text
/// while maintaining the appropriate tone and format for the context.
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
    tracing::info!(
        "[ai_refine] Refining content for scope: {}, current length: {}",
        context.scope,
        current_text.len()
    );

    // Create AI agent
    let agent = GeminiApiAgent::try_from_env()
        .await
        .map_err(|e| format!("Failed to initialize AI agent: {}", e))?;

    // Build system instruction for refinement
    let system_instruction = build_refinement_instruction(&context);
    let agent = agent.with_system_instruction(system_instruction);

    // Build refinement prompt
    let full_prompt = format!(
        "Current text:\n{}\n\nRefinement instruction: {}\n\nProvide only the refined text without any explanations.",
        current_text, prompt
    );

    // Execute refinement
    let speaker = Speaker::user("User", "User");
    let payload = Payload::new().with_message(speaker, &full_prompt);
    let result = agent
        .execute(payload)
        .await
        .map_err(|e| format!("AI refinement failed: {}", e))?;

    tracing::info!(
        "[ai_refine] Refined from {} to {} characters for scope: {}",
        current_text.len(),
        result.len(),
        context.scope
    );

    Ok(result.trim().to_string())
}

/// Builds a system instruction for text generation based on context
fn build_generation_instruction(context: &AIContext) -> String {
    let mut instruction = format!(
        "You are a text generation assistant for a UI application.\n\
        Scope: {}\n\
        Content type: {}\n",
        context.scope, context.content_type
    );

    if let Some(max_len) = context.max_length {
        instruction.push_str(&format!("Maximum length: {} characters\n", max_len));
    }

    instruction.push_str(
        "\nGuidelines:\n\
        - Generate natural, contextually appropriate text\n\
        - Match the tone to the content type and scope\n\
        - Be concise and clear\n\
        - Provide only the generated text without explanations\n"
    );

    if context.content_type == "code" {
        instruction.push_str("- Output valid code without markdown code blocks\n");
    }

    instruction
}

/// Builds a system instruction for text refinement based on context
fn build_refinement_instruction(context: &AIContext) -> String {
    let mut instruction = format!(
        "You are a text refinement assistant for a UI application.\n\
        Scope: {}\n\
        Content type: {}\n",
        context.scope, context.content_type
    );

    if let Some(max_len) = context.max_length {
        instruction.push_str(&format!("Maximum length: {} characters\n", max_len));
    }

    instruction.push_str(
        "\nGuidelines:\n\
        - Refine the text according to the user's instruction\n\
        - Maintain the original meaning and context\n\
        - Improve clarity, grammar, and style as needed\n\
        - Respect length constraints if specified\n\
        - Provide only the refined text without explanations\n"
    );

    if context.content_type == "code" {
        instruction.push_str("- Output valid code without markdown code blocks\n");
    }

    instruction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_generation_instruction_basic() {
        let context = AIContext {
            scope: "Test.Field".to_string(),
            content_type: "string".to_string(),
            max_length: None,
            metadata: None,
        };

        let instruction = build_generation_instruction(&context);

        assert!(instruction.contains("Test.Field"));
        assert!(instruction.contains("string"));
        assert!(instruction.contains("Guidelines:"));
    }

    #[test]
    fn test_build_generation_instruction_with_max_length() {
        let context = AIContext {
            scope: "UserProfile.Bio".to_string(),
            content_type: "long_text".to_string(),
            max_length: Some(500),
            metadata: None,
        };

        let instruction = build_generation_instruction(&context);

        assert!(instruction.contains("Maximum length: 500"));
    }

    #[test]
    fn test_build_generation_instruction_code() {
        let context = AIContext {
            scope: "CodeEditor".to_string(),
            content_type: "code".to_string(),
            max_length: None,
            metadata: None,
        };

        let instruction = build_generation_instruction(&context);

        assert!(instruction.contains("code"));
        assert!(instruction.contains("without markdown code blocks"));
    }

    #[test]
    fn test_build_refinement_instruction() {
        let context = AIContext {
            scope: "MessageItem".to_string(),
            content_type: "string".to_string(),
            max_length: Some(200),
            metadata: None,
        };

        let instruction = build_refinement_instruction(&context);

        assert!(instruction.contains("MessageItem"));
        assert!(instruction.contains("Maximum length: 200"));
        assert!(instruction.contains("Refine the text"));
    }
}
