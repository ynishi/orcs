//! ClaudeApiAgent - Direct REST API implementation for Claude.
//!
//! This agent calls the Claude REST API directly without CLI dependency.
//! Configuration priority: ~/.config/orcs/secret.json > environment variables

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use llm_toolkit::attachment::Attachment;
use orcs_infrastructure::storage::SecretStorage;
use reqwest::{Client, StatusCode, header::HeaderValue};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

const DEFAULT_CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";
const BASE_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Agent implementation that talks to the Claude HTTP API.
#[derive(Clone)]
pub struct ClaudeApiAgent {
    client: Client,
    api_key: String,
    model: String,
    system: Option<String>,
    max_tokens: u32,
}

impl ClaudeApiAgent {
    /// Creates a new agent with the provided API key and model.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: model.into(),
            system: None,
            max_tokens: 4096,
        }
    }

    /// Loads configuration from ~/.config/orcs/secret.json or environment variables.
    ///
    /// Priority:
    /// 1. ~/.config/orcs/secret.json
    /// 2. Environment variables (ANTHROPIC_API_KEY, CLAUDE_MODEL_NAME)
    ///
    /// Model name defaults to `claude-sonnet-4-20250514` if not specified.
    pub fn try_from_env() -> Result<Self, AgentError> {
        // Try loading from SecretStorage first
        if let Ok(storage) = SecretStorage::new() {
            if let Ok(secret_config) = storage.load() {
                if let Some(claude_config) = secret_config.claude {
                    // Use default model (model settings now in config.toml)
                    let model = DEFAULT_CLAUDE_MODEL.to_string();
                    return Ok(Self::new(claude_config.api_key, model));
                }
            }
        }

        // Fallback to environment variables
        let api_key = env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AgentError::ExecutionFailed(
                "ANTHROPIC_API_KEY not found in ~/.config/orcs/secret.json or environment variables"
                    .into(),
            )
        })?;

        let model = env::var("CLAUDE_MODEL_NAME").unwrap_or_else(|_| DEFAULT_CLAUDE_MODEL.into());
        Ok(Self::new(api_key, model))
    }

    /// Overrides the model after construction.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Adds a system prompt that will be sent alongside every request.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    async fn build_content(&self, payload: &Payload) -> Result<Vec<ContentBlock>, AgentError> {
        let mut content_blocks = Vec::new();

        // Add text content
        let text = payload.to_text();
        if !text.trim().is_empty() {
            content_blocks.push(ContentBlock::Text { text });
        }

        // Add attachments
        for attachment in payload.attachments() {
            if let Some(block) = Self::attachment_to_content_block(attachment).await? {
                content_blocks.push(block);
            }
        }

        if content_blocks.is_empty() {
            return Err(AgentError::ExecutionFailed(
                "Claude payload must include text or supported attachments".into(),
            ));
        }

        Ok(content_blocks)
    }

    async fn attachment_to_content_block(
        attachment: &Attachment,
    ) -> Result<Option<ContentBlock>, AgentError> {
        match attachment {
            Attachment::Remote(_) => {
                return Err(AgentError::ExecutionFailed(
                    "Remote attachments are not supported for Claude API".into(),
                ));
            }
            _ => {}
        }

        let bytes = attachment.load_bytes().await.map_err(|err| {
            AgentError::ExecutionFailed(format!("Failed to load attachment for Claude API: {err}"))
        })?;

        let media_type = attachment
            .mime_type()
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = BASE64_STANDARD.encode(bytes);

        Ok(Some(ContentBlock::Image {
            source: ImageSource {
                r#type: "base64".to_string(),
                media_type,
                data,
            },
        }))
    }

    async fn send_request(&self, body: &CreateMessageRequest) -> Result<String, AgentError> {
        let response = self
            .client
            .post(BASE_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|err| AgentError::ProcessError {
                status_code: None,
                message: format!("Claude API request failed: {err}"),
                is_retryable: err.is_connect() || err.is_timeout(),
                retry_after: None,
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let retry_after = parse_retry_after(response.headers().get("retry-after"));
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read Claude error body".to_string());
            return Err(map_http_error(status, body_text, retry_after));
        }

        let parsed: CreateMessageResponse = response
            .json()
            .await
            .map_err(|err| AgentError::Other(format!("Failed to parse Claude response: {err}")))?;

        extract_text_response(parsed)
    }
}

#[async_trait]
impl Agent for ClaudeApiAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        "Claude API agent for advanced reasoning and coding tasks"
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        let content = self.build_content(&payload).await?;

        let messages = vec![Message {
            role: "user".to_string(),
            content,
        }];

        let request = CreateMessageRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            system: self.system.clone(),
        };

        self.send_request(&request).await
    }
}

#[derive(Serialize)]
struct CreateMessageRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentBlock>,
}

enum ContentBlock {
    Text { text: String },
    Image { source: ImageSource },
}

// Custom serialization for ContentBlock to match Claude API format
impl Serialize for ContentBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        match self {
            ContentBlock::Text { text } => {
                map.serialize_entry("type", "text")?;
                map.serialize_entry("text", text)?;
            }
            ContentBlock::Image { source } => {
                map.serialize_entry("type", "image")?;
                map.serialize_entry("source", source)?;
            }
        }

        map.end()
    }
}

#[derive(Serialize)]
struct ImageSource {
    r#type: String,
    media_type: String,
    data: String,
}

#[derive(Deserialize)]
struct CreateMessageResponse {
    content: Vec<ContentBlockResponse>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ContentBlockResponse {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[allow(dead_code)]
    r#type: String,
    message: String,
}

fn extract_text_response(response: CreateMessageResponse) -> Result<String, AgentError> {
    response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlockResponse::Text { text } => Some(text),
        })
        .ok_or_else(|| {
            AgentError::ExecutionFailed(
                "Claude API returned no text in the response content".into(),
            )
        })
}

fn map_http_error(status: StatusCode, body: String, retry_after: Option<Duration>) -> AgentError {
    let message = serde_json::from_str::<ErrorResponse>(&body)
        .map(|wrapper| wrapper.error.message)
        .unwrap_or_else(|_| body.clone());

    let is_retryable = matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    );

    if let Some(delay) = retry_after {
        AgentError::process_error_with_retry_after(status.as_u16(), message, is_retryable, delay)
    } else {
        AgentError::ProcessError {
            status_code: Some(status.as_u16()),
            message,
            is_retryable,
            retry_after: None,
        }
    }
}

fn parse_retry_after(header: Option<&HeaderValue>) -> Option<Duration> {
    let value = header?.to_str().ok()?;
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    // Retry-After HTTP-date parsing is omitted for simplicity
    None
}
