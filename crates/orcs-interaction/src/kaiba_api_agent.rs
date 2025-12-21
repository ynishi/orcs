//! KaibaApiAgent - REST API implementation for Kaiba (Autonomous personas with persistent memory).
//!
//! This agent integrates with the Kaiba service to provide personas with:
//! - Persistent memory across sessions
//! - Context-aware responses using RAG
//! - Ability to continue conversations from previous sessions
//!
//! Configuration priority: secret.json > environment variables

use async_trait::async_trait;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use orcs_core::secret::SecretService;
use orcs_infrastructure::SecretServiceImpl;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

const DEFAULT_KAIBA_URL: &str = "https://kaiba.shuttleapp.rs";
const DEFAULT_CLAUDE_MODEL: &str = "claude-sonnet-4-20250514"; // Kaiba generates prompts, Claude executes

/// Agent implementation that talks to the Kaiba API to retrieve Rei prompts.
#[derive(Clone)]
pub struct KaibaApiAgent {
    client: Client,
    kaiba_url: String,
    kaiba_api_key: Option<String>,
    rei_id: String,
    // For executing the prompt after fetching from Kaiba
    anthropic_api_key: String,
    model: String,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct KaibaPromptResponse {
    system_prompt: String,
    format: String,
    rei: ReiInfo,
    memories_included: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReiInfo {
    id: String,
    name: String,
    role: String,
    energy_level: u32,
    mood: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    system: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
}

impl KaibaApiAgent {
    /// Creates a new agent with the provided configuration.
    pub fn new(
        rei_id: impl Into<String>,
        kaiba_url: impl Into<String>,
        kaiba_api_key: Option<String>,
        anthropic_api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            kaiba_url: kaiba_url.into(),
            kaiba_api_key,
            rei_id: rei_id.into(),
            anthropic_api_key: anthropic_api_key.into(),
            model: model.into(),
            max_tokens: 4096,
        }
    }

    /// Loads configuration from secret.json or environment variables.
    ///
    /// Priority:
    /// 1. secret.json
    /// 2. Environment variables (KAIBA_URL, KAIBA_API_KEY, KAIBA_REI_ID, ANTHROPIC_API_KEY)
    ///
    /// Kaiba URL defaults to `https://kaiba.shuttleapp.rs` if not specified.
    pub async fn try_from_env() -> Result<Self, AgentError> {
        // Try loading from SecretService first
        let (kaiba_url, kaiba_api_key, rei_id, anthropic_api_key, model) =
            if let Ok(service) = SecretServiceImpl::new_default()
                && let Ok(secret_config) = service.load_secrets().await
            {
                let kaiba_url = secret_config.kaiba
                    .as_ref()
                    .and_then(|k| k.url.clone())
                    .unwrap_or_else(|| DEFAULT_KAIBA_URL.to_string());

                let kaiba_api_key = secret_config.kaiba
                    .as_ref()
                    .and_then(|k| k.api_key.clone());

                let rei_id = secret_config.kaiba
                    .as_ref()
                    .and_then(|k| k.default_rei_id.clone())
                    .or_else(|| env::var("KAIBA_REI_ID").ok())
                    .ok_or_else(|| {
                        AgentError::ExecutionFailed(
                            "KAIBA_REI_ID not found in secret.json or environment variables".into(),
                        )
                    })?;

                let anthropic_api_key = secret_config.claude
                    .as_ref()
                    .and_then(|c| Some(c.api_key.clone()))
                    .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| {
                        AgentError::ExecutionFailed(
                            "ANTHROPIC_API_KEY not found in secret.json or environment variables".into(),
                        )
                    })?;

                let model = DEFAULT_CLAUDE_MODEL.to_string();

                (kaiba_url, kaiba_api_key, rei_id, anthropic_api_key, model)
            } else {
                // Fallback to environment variables
                let kaiba_url = env::var("KAIBA_URL")
                    .unwrap_or_else(|_| DEFAULT_KAIBA_URL.to_string());

                let kaiba_api_key = env::var("KAIBA_API_KEY").ok();

                let rei_id = env::var("KAIBA_REI_ID").map_err(|_| {
                    AgentError::ExecutionFailed(
                        "KAIBA_REI_ID not found in environment variables".into(),
                    )
                })?;

                let anthropic_api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| {
                    AgentError::ExecutionFailed(
                        "ANTHROPIC_API_KEY not found in environment variables".into(),
                    )
                })?;

                let model = env::var("CLAUDE_MODEL_NAME")
                    .unwrap_or_else(|_| DEFAULT_CLAUDE_MODEL.to_string());

                (kaiba_url, kaiba_api_key, rei_id, anthropic_api_key, model)
            };

        Ok(Self::new(
            rei_id,
            kaiba_url,
            kaiba_api_key,
            anthropic_api_key,
            model,
        ))
    }

    /// Sets the Rei ID to use for this agent.
    pub fn with_rei_id(mut self, rei_id: impl Into<String>) -> Self {
        self.rei_id = rei_id.into();
        self
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Fetches the Rei prompt from Kaiba service.
    async fn fetch_rei_prompt(&self, context: &str) -> Result<KaibaPromptResponse, AgentError> {
        let url = format!(
            "{}/kaiba/rei/{}/prompt",
            self.kaiba_url,
            self.rei_id
        );

        let mut request = self.client
            .get(&url)
            .query(&[("format", "raw"), ("context", context)])
            .timeout(Duration::from_secs(30));

        if let Some(api_key) = &self.kaiba_api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| AgentError::ExecutionFailed(format!("Failed to fetch Rei prompt: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentError::ExecutionFailed(
                format!("Kaiba API error ({}): {}", status, error_text)
            ));
        }

        response.json::<KaibaPromptResponse>().await
            .map_err(|e| AgentError::ExecutionFailed(format!("Failed to parse Kaiba response: {}", e)))
    }

    /// Executes the prompt using Claude API.
    async fn execute_with_claude(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, AgentError> {
        let request_body = ClaudeRequest {
            model: self.model.clone(),
            system: system_prompt.to_string(),
            messages: vec![
                ClaudeMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            max_tokens: self.max_tokens,
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.anthropic_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .timeout(Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| AgentError::ExecutionFailed(format!("Claude API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentError::ExecutionFailed(
                format!("Claude API error ({}): {}", status, error_text)
            ));
        }

        let claude_response: ClaudeResponse = response.json().await
            .map_err(|e| AgentError::ExecutionFailed(format!("Failed to parse Claude response: {}", e)))?;

        // Extract text from response
        Ok(claude_response.content
            .into_iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text),
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }

    /// Stores the conversation memory back to Kaiba.
    async fn store_memory(&self, content: &str, importance: f32) -> Result<(), AgentError> {
        #[derive(Serialize)]
        struct MemoryRequest {
            content: String,
            memory_type: String,
            importance: f32,
        }

        let request_body = MemoryRequest {
            content: content.to_string(),
            memory_type: "conversation".to_string(),
            importance,
        };

        let url = format!("{}/personas/{}/memories", self.kaiba_url, self.rei_id);

        let mut request = self.client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(10));

        if let Some(api_key) = &self.kaiba_api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await
            .map_err(|e| AgentError::ExecutionFailed(format!("Failed to store memory: {}", e)))?;

        if !response.status().is_success() {
            // Log but don't fail - memory storage is best-effort
            eprintln!("Warning: Failed to store memory to Kaiba (status: {})", response.status());
        }

        Ok(())
    }
}

#[async_trait]
impl Agent for KaibaApiAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        "Kaiba API agent with persistent memory and context awareness"
    }

    /// Executes the agent with the given payload.
    ///
    /// This implementation:
    /// 1. Fetches the Rei prompt from Kaiba (with RAG memories)
    /// 2. Executes the prompt using Claude API
    /// 3. Stores the conversation back to Kaiba for future memory
    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        let user_message = payload.to_text();

        // Fetch Rei prompt with context-aware memories
        let kaiba_response = self.fetch_rei_prompt(&user_message).await?;

        // Log Rei state for debugging
        eprintln!(
            "Rei activated: {} ({}) - Energy: {}%, Mood: {}, Memories: {}",
            kaiba_response.rei.name,
            kaiba_response.rei.role,
            kaiba_response.rei.energy_level,
            kaiba_response.rei.mood,
            kaiba_response.memories_included
        );

        // Execute with Claude using the Rei prompt
        let response = self.execute_with_claude(
            &kaiba_response.system_prompt,
            &user_message,
        ).await?;

        // Store the conversation as memory (best-effort, don't fail on error)
        let memory_content = format!("User: {}\n\nAssistant: {}", user_message, response);
        let _ = self.store_memory(&memory_content, 0.7).await;

        Ok(response)
    }
}