//! Utility Agent Service
//!
//! Provides lightweight LLM operations for UI enhancements like title generation,
//! summarization, and icon suggestions using fast models (Gemini Flash API).

use anyhow::Result;
use llm_toolkit::agent::Agent;
use llm_toolkit::ToPrompt;
use serde::{Deserialize, Serialize};

/// Generic title/metadata response from lightweight LLM
///
/// This structure is used across the application for generating concise,
/// user-friendly metadata from arbitrary content (task descriptions, session
/// conversations, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, ToPrompt)]
#[prompt(mode = "full")]
pub struct TitleResponse {
    /// Concise, descriptive title (recommended: 3-8 words)
    pub title: String,

    /// Optional brief description (1-2 sentences)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional representative emoji/icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Typed request for title generation using Jinja2 template
#[derive(Debug, Clone, Serialize, ToPrompt, Default)]
#[prompt(
    mode = "full",
    template = r#"Generate title and metadata for this {{ context }} content:

{{ content }}

Requirements:
{% for req in requirements -%}
- {{ req }}
{% endfor %}

Context: This is a {{ context }} - choose appropriate tone and brevity.

Output a JSON object matching this schema:
{{ output_schema }}

IMPORTANT: Output ONLY valid JSON, no markdown formatting or code blocks."#
)]
struct TitleGenerationRequest {
    /// The content to generate title from (truncated to 500 chars)
    content: String,

    /// Context type (e.g., "task", "session", "article")
    context: String,

    /// List of requirements
    requirements: Vec<String>,

    /// Output schema for TitleResponse
    output_schema: String,
}

/// Lightweight agent for generating titles and metadata using Gemini Flash API
#[derive(llm_toolkit::Agent)]
#[agent(
    expertise = "Generate concise, descriptive titles and metadata from content. Focus on clarity and brevity.",
    output = "TitleResponse",
    inner = "orcs_interaction::GeminiApiAgent"
)]
struct TitleGeneratorAgent;

/// Service providing lightweight LLM utilities
pub struct UtilityAgentService {
    title_agent: TitleGeneratorAgent,
}

impl UtilityAgentService {
    pub fn new() -> Self {
        let title_agent = TitleGeneratorAgent;
        Self { title_agent }
    }

    /// Generate title and metadata from content using Gemini Flash
    ///
    /// # Arguments
    ///
    /// * `content` - The content to generate metadata for (task description, conversation, etc.)
    /// * `context` - Additional context (e.g., "task", "session", "article")
    /// * `include_description` - Whether to generate description field
    /// * `include_icon` - Whether to generate icon field
    ///
    /// # Returns
    ///
    /// * `Ok(TitleResponse)` - Generated metadata
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Task title generation
    /// let metadata = service.generate_title(
    ///     "承知しました！RustのStructに関する記事作成...",
    ///     "task",
    ///     false, // no description needed
    ///     true,  // include icon
    /// ).await?;
    /// // metadata.title = "Rust Struct記事作成"
    /// // metadata.icon = Some("📝")
    ///
    /// // Session title from conversation
    /// let metadata = service.generate_title(
    ///     "[Last 500 chars of conversation]",
    ///     "session",
    ///     true,  // include description
    ///     false, // no icon
    /// ).await?;
    /// // metadata.title = "映画制作プロセス相談"
    /// // metadata.description = Some("映画制作の各フェーズについて専門家と議論")
    /// ```
    pub async fn generate_title(
        &self,
        content: &str,
        context: &str,
        include_description: bool,
        include_icon: bool,
    ) -> Result<TitleResponse> {
        use llm_toolkit::prompt::ToPrompt;

        let mut requirements = vec![
            "Generate a concise, descriptive title (3-8 words recommended)".to_string(),
        ];

        if include_description {
            requirements.push("Generate a brief description (1-2 sentences)".to_string());
        } else {
            requirements.push("Set description to null (not needed)".to_string());
        }

        if include_icon {
            requirements.push(
                "Suggest an appropriate emoji/icon that represents the content".to_string(),
            );
        } else {
            requirements.push("Set icon to null (not needed)".to_string());
        }

        // Create typed request with Jinja2 template
        let request = TitleGenerationRequest {
            content: if content.len() > 500 {
                content[..500].to_string()
            } else {
                content.to_string()
            },
            context: context.to_string(),
            requirements,
            output_schema: TitleResponse::prompt_schema(),
        };

        // Generate prompt using ToPrompt derive
        let prompt = request.to_prompt();

        let response: TitleResponse = self.title_agent.execute(prompt.as_str().into()).await?;
        Ok(response)
    }

    /// Generate a concise task title (optimized for task execution)
    ///
    /// # Arguments
    ///
    /// * `task_description` - The full task description/message content
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Concise task title (3-8 words)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let title = service.generate_task_title(
    ///     "承知しました！RustのStructに関する記事を作成して..."
    /// ).await?;
    /// // "Rust Struct記事作成"
    /// ```
    pub async fn generate_task_title(&self, task_description: &str) -> Result<String> {
        let response = self
            .generate_title(task_description, "task", false, false)
            .await?;
        Ok(response.title)
    }

    /// Generate session title from conversation history
    ///
    /// # Arguments
    ///
    /// * `conversation_summary` - Summary or recent messages from session
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Session title
    pub async fn generate_session_title(&self, conversation_summary: &str) -> Result<String> {
        let response = self
            .generate_title(conversation_summary, "session", false, false)
            .await?;
        Ok(response.title)
    }
}

impl Default for UtilityAgentService {
    fn default() -> Self {
        Self::new()
    }
}
