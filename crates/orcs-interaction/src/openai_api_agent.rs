//! OpenAIApiAgent - Direct REST API implementation for OpenAI GPT.
//!
//! This agent calls the OpenAI Chat Completions API directly without CLI dependency.
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

const DEFAULT_OPENAI_MODEL: &str = "gpt-4o";
const BASE_URL: &str = "https://api.openai.com/v1/chat/completions";

/// Agent implementation that talks to the OpenAI HTTP API.
#[derive(Clone)]
pub struct OpenAIApiAgent {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: Option<u32>,
}

impl OpenAIApiAgent {
    /// Creates a new agent with the provided API key and model.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: model.into(),
            max_tokens: None,
        }
    }

    /// Loads configuration from ~/.config/orcs/secret.json or environment variables.
    ///
    /// Priority:
    /// 1. ~/.config/orcs/secret.json
    /// 2. Environment variables (OPENAI_API_KEY, OPENAI_MODEL_NAME)
    ///
    /// Model name defaults to `gpt-4o` if not specified.
    pub fn try_from_env() -> Result<Self, AgentError> {
        // Try loading from SecretStorage first
        if let Ok(storage) = SecretStorage::new() {
            if let Ok(secret_config) = storage.load() {
                if let Some(openai_config) = secret_config.openai {
                    let model = openai_config
                        .model_name
                        .unwrap_or_else(|| DEFAULT_OPENAI_MODEL.into());
                    return Ok(Self::new(openai_config.api_key, model));
                }
            }
        }

        // Fallback to environment variables
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            AgentError::ExecutionFailed(
                "OPENAI_API_KEY not found in ~/.config/orcs/secret.json or environment variables"
                    .into(),
            )
        })?;

        let model = env::var("OPENAI_MODEL_NAME").unwrap_or_else(|_| DEFAULT_OPENAI_MODEL.into());
        Ok(Self::new(api_key, model))
    }

    /// Overrides the model after construction.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    async fn build_messages(&self, payload: &Payload) -> Result<Vec<ChatMessage>, AgentError> {
        let mut content_parts = Vec::new();

        // Add text content
        let text = payload.to_text();
        if !text.trim().is_empty() {
            content_parts.push(MessageContent::Text { text });
        }

        // Add image attachments
        for attachment in payload.attachments() {
            if let Some(content) = Self::attachment_to_content(attachment).await? {
                content_parts.push(content);
            }
        }

        if content_parts.is_empty() {
            return Err(AgentError::ExecutionFailed(
                "OpenAI payload must include text or supported attachments".into(),
            ));
        }

        Ok(vec![ChatMessage {
            role: "user".to_string(),
            content: content_parts,
        }])
    }

    async fn attachment_to_content(
        attachment: &Attachment,
    ) -> Result<Option<MessageContent>, AgentError> {
        match attachment {
            Attachment::Remote(url) => {
                // OpenAI supports image URLs directly
                return Ok(Some(MessageContent::ImageUrl {
                    image_url: ImageUrl {
                        url: url.to_string(),
                    },
                }));
            }
            _ => {}
        }

        let bytes = attachment.load_bytes().await.map_err(|err| {
            AgentError::ExecutionFailed(format!("Failed to load attachment for OpenAI API: {err}"))
        })?;

        let mime_type = attachment
            .mime_type()
            .unwrap_or_else(|| "image/jpeg".to_string());

        // OpenAI expects data URLs for base64 images
        let data_url = format!(
            "data:{};base64,{}",
            mime_type,
            BASE64_STANDARD.encode(bytes)
        );

        Ok(Some(MessageContent::ImageUrl {
            image_url: ImageUrl { url: data_url },
        }))
    }

    async fn send_request(&self, body: &ChatCompletionRequest) -> Result<String, AgentError> {
        let response = self
            .client
            .post(BASE_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|err| AgentError::ProcessError {
                status_code: None,
                message: format!("OpenAI API request failed: {err}"),
                is_retryable: err.is_connect() || err.is_timeout(),
                retry_after: None,
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let retry_after = parse_retry_after(response.headers().get("retry-after"));
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read OpenAI error body".to_string());
            return Err(map_http_error(status, body_text, retry_after));
        }

        let parsed: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|err| AgentError::Other(format!("Failed to parse OpenAI response: {err}")))?;

        extract_text_response(parsed)
    }
}

#[async_trait]
impl Agent for OpenAIApiAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        "OpenAI GPT agent for general-purpose reasoning and coding tasks"
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        let messages = self.build_messages(&payload).await?;

        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
        };

        self.send_request(&request).await
    }
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: Vec<MessageContent>,
}

enum MessageContent {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

// Custom serialization for MessageContent
impl Serialize for MessageContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        match self {
            MessageContent::Text { text } => {
                map.serialize_entry("type", "text")?;
                map.serialize_entry("text", text)?;
            }
            MessageContent::ImageUrl { image_url } => {
                map.serialize_entry("type", "image_url")?;
                map.serialize_entry("image_url", image_url)?;
            }
        }

        map.end()
    }
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    message: String,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    code: Option<String>,
}

fn extract_text_response(response: ChatCompletionResponse) -> Result<String, AgentError> {
    response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| {
            AgentError::ExecutionFailed("OpenAI API returned no content in the response".into())
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
