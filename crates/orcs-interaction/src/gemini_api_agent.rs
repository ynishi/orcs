//! GeminiApiAgent - Direct REST API implementation for Gemini.
//!
//! This agent calls the Gemini REST API directly without CLI dependency.
//! Configuration is loaded from secret.json

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use llm_toolkit::attachment::Attachment;
use orcs_core::secret::SecretService;
use orcs_infrastructure::SecretServiceImpl;
use reqwest::{Client, StatusCode, header::HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_GEMINI_MODEL: &str = "gemini-2.5-flash";
const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Agent implementation that talks to the Gemini HTTP API.
#[derive(Clone)]
pub struct GeminiApiAgent {
    client: Client,
    api_key: String,
    model: String,
    system_instruction: Option<String>,
}

impl GeminiApiAgent {
    /// Creates a new agent with the provided API key and model.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: model.into(),
            system_instruction: None,
        }
    }

    /// Loads configuration from secret.json
    ///
    /// Model name defaults to `gemini-2.5-flash` if not specified.
    pub async fn try_from_env() -> Result<Self, AgentError> {
        let service = SecretServiceImpl::default().map_err(|e| {
            AgentError::ExecutionFailed(format!("Failed to initialize SecretService: {}", e))
        })?;

        let secret_config = service.load_secrets().await.map_err(|e| {
            AgentError::ExecutionFailed(format!("Failed to load secret.json: {}", e))
        })?;

        let gemini_config = secret_config.gemini.ok_or_else(|| {
            AgentError::ExecutionFailed("Gemini configuration not found in secret.json".to_string())
        })?;

        // Use default model (model settings now in config.toml)
        let model = DEFAULT_GEMINI_MODEL.to_string();

        Ok(Self::new(gemini_config.api_key, model))
    }

    /// Overrides the model after construction.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Adds a system instruction that will be sent alongside every request.
    pub fn with_system_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.system_instruction = Some(instruction.into());
        self
    }

    async fn build_parts(&self, payload: &Payload) -> Result<Vec<Part>, AgentError> {
        let mut parts = Vec::new();
        let text = payload.to_text();
        if !text.trim().is_empty() {
            parts.push(Part::Text { text });
        }

        for attachment in payload.attachments() {
            if let Some(part) = Self::attachment_to_part(attachment).await? {
                parts.push(part);
            }
        }

        if parts.is_empty() {
            return Err(AgentError::ExecutionFailed(
                "Gemini payload must include text or supported attachments".into(),
            ));
        }

        Ok(parts)
    }

    async fn attachment_to_part(attachment: &Attachment) -> Result<Option<Part>, AgentError> {
        match attachment {
            Attachment::Remote(_) => {
                return Err(AgentError::ExecutionFailed(
                    "Remote attachments are not supported for Gemini API".into(),
                ));
            }
            _ => {}
        }

        let bytes = attachment.load_bytes().await.map_err(|err| {
            AgentError::ExecutionFailed(format!("Failed to load attachment for Gemini API: {err}"))
        })?;

        let mime_type = attachment
            .mime_type()
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let data = BASE64_STANDARD.encode(bytes);
        Ok(Some(Part::InlineData {
            inline_data: InlineDataPayload { mime_type, data },
        }))
    }

    async fn send_request(&self, body: &GenerateContentRequest) -> Result<String, AgentError> {
        let url = format!(
            "{}/{model}:generateContent?key={api_key}",
            BASE_URL,
            model = self.model,
            api_key = self.api_key
        );

        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(|err| AgentError::ProcessError {
                status_code: None,
                message: format!("Gemini API request failed: {err}"),
                is_retryable: err.is_connect() || err.is_timeout(),
                retry_after: None,
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let retry_after = parse_retry_after(response.headers().get("retry-after"));
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read Gemini error body".to_string());
            return Err(map_http_error(status, body_text, retry_after));
        }

        let parsed: GenerateContentResponse = response
            .json()
            .await
            .map_err(|err| AgentError::Other(format!("Failed to parse Gemini response: {err}")))?;

        extract_text_response(parsed)
    }
}

#[async_trait]
impl Agent for GeminiApiAgent {
    type Output = String;

    fn expertise(&self) -> &str {
        "Gemini API agent for multimodal reasoning"
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        let contents = vec![Content {
            role: "user".to_string(),
            parts: self.build_parts(&payload).await?,
        }];

        let system_instruction = self.system_instruction.as_ref().map(|text| Content {
            role: "system".to_string(),
            parts: vec![Part::Text {
                text: text.to_string(),
            }],
        });

        let request = GenerateContentRequest {
            contents,
            system_instruction,
        };
        self.send_request(&request).await
    }
}

#[derive(Serialize)]
struct GenerateContentRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<Content>,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Part {
    Text {
        text: String,
    },
    InlineData {
        #[serde(rename = "inlineData")]
        inline_data: InlineDataPayload,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineDataPayload {
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GenerateContentResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<ContentResponse>,
}

#[derive(Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
struct PartResponse {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ErrorWrapper {
    error: ErrorBody,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[allow(dead_code)]
    code: Option<i32>,
    message: Option<String>,
    status: Option<String>,
}

fn extract_text_response(response: GenerateContentResponse) -> Result<String, AgentError> {
    response
        .candidates
        .and_then(|mut candidates| candidates.pop())
        .and_then(|candidate| candidate.content)
        .and_then(|content| content.parts.into_iter().find_map(|part| part.text))
        .ok_or_else(|| {
            AgentError::ExecutionFailed(
                "Gemini API returned no text in the response candidates".into(),
            )
        })
}

fn map_http_error(status: StatusCode, body: String, retry_after: Option<Duration>) -> AgentError {
    let message = serde_json::from_str::<ErrorWrapper>(&body)
        .map(|wrapper| {
            let status_text = wrapper.error.status.unwrap_or_default();
            let msg = wrapper.error.message.unwrap_or_else(|| body.clone());
            if status_text.is_empty() {
                msg
            } else {
                format!("{status_text}: {msg}")
            }
        })
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
