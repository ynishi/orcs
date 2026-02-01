//! Google Web Search agent that leverages Gemini's `google_search` tool.
//!
//! The agent sends `generateContent` requests with the google_search tool enabled
//! and extracts both the synthesized answer and grounded references, allowing
//! downstream consumers to display a short answer plus supporting links.

use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use llm_toolkit::agent::{Agent, AgentError, Payload};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";

/// Agent capable of calling Gemini with the google_search tool.
#[derive(Clone)]
pub struct WebSearchAgent {
    client: Client,
    api_key: String,
    model: String,
}

impl WebSearchAgent {
    /// Creates a new agent using the provided API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Overrides the Gemini model name if needed.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    async fn perform_search(&self, query: &str) -> Result<WebSearchResponse, AgentError> {
        let url = format!(
            "{}/{model}:generateContent?key={api_key}",
            BASE_URL,
            model = self.model,
            api_key = self.api_key
        );

        let request = GenerateContentRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: query.to_string(),
                }],
            }],
            tools: vec![Tool::default()],
        };

        let response = self
            .client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|err| AgentError::ProcessError {
                status_code: None,
                message: format!("Google Search request failed: {err}"),
                is_retryable: err.is_connect() || err.is_timeout(),
                retry_after: None,
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(parse_retry_after);
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read Google Search error body".to_string());
            return Err(map_http_error(status, body, retry_after));
        }

        let payload: Value = response.json().await.map_err(|err| {
            AgentError::Other(format!("Failed to parse Google Search response: {err}"))
        })?;

        let answer = extract_answer(&payload)
            .unwrap_or_else(|| "Google Search returned no answer".to_string());
        let references = extract_references(&payload);

        Ok(WebSearchResponse {
            query: query.to_string(),
            answer,
            references,
        })
    }
}

#[async_trait]
impl Agent for WebSearchAgent {
    type Output = WebSearchResponse;
    type Expertise = String;

    fn expertise(&self) -> &String {
        use std::sync::OnceLock;
        static EXPERTISE: OnceLock<String> = OnceLock::new();
        EXPERTISE.get_or_init(|| "Google Web Search agent".to_string())
    }

    async fn execute(&self, payload: Payload) -> Result<Self::Output, AgentError> {
        let query = payload.to_text();
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(AgentError::ExecutionFailed(
                "WebSearch query cannot be empty".into(),
            ));
        }

        self.perform_search(trimmed).await
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateContentRequest {
    contents: Vec<Content>,
    tools: Vec<Tool>,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize, Default)]
struct Tool {
    #[serde(rename = "google_search")]
    google_search: GoogleSearchConfig,
}

#[derive(Serialize, Default)]
struct GoogleSearchConfig {}

/// Structured reference returned by Gemini's grounding metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchReference {
    pub title: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Search response returned to the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub query: String,
    pub answer: String,
    pub references: Vec<WebSearchReference>,
}

fn extract_answer(root: &Value) -> Option<String> {
    let candidates = root.get("candidates")?.as_array()?;

    let mut collected = Vec::new();
    for candidate in candidates {
        if let Some(parts) = candidate
            .get("content")
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
        {
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        collected.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    if collected.is_empty() {
        None
    } else {
        Some(collected.join("\n\n"))
    }
}

fn extract_references(root: &Value) -> Vec<WebSearchReference> {
    let mut seen = HashSet::new();
    let mut references = Vec::new();

    let candidates = match root.get("candidates").and_then(|c| c.as_array()) {
        Some(list) => list,
        None => return references,
    };

    for candidate in candidates {
        let metadata = match candidate.get("groundingMetadata") {
            Some(value) => value,
            None => continue,
        };

        let chunks = match metadata
            .get("groundingChunks")
            .and_then(|chunks| chunks.as_array())
        {
            Some(list) => list,
            None => continue,
        };

        for chunk in chunks {
            let web = chunk
                .get("web")
                .or_else(|| chunk.get("webSearch"))
                .or_else(|| chunk.get("retrievedReference"));

            let Some(web_obj) = web else {
                continue;
            };

            let url = web_obj
                .get("uri")
                .or_else(|| web_obj.get("url"))
                .and_then(|v| v.as_str());
            let Some(url) = url else {
                continue;
            };

            if !seen.insert(url.to_string()) {
                continue;
            }

            let title = web_obj
                .get("title")
                .or_else(|| web_obj.get("pageTitle"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| url.to_string());

            let snippet = web_obj
                .get("snippet")
                .or_else(|| web_obj.get("text"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let source = web_obj
                .get("siteName")
                .or_else(|| web_obj.get("source"))
                .or_else(|| web_obj.get("displayUri"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            references.push(WebSearchReference {
                title,
                url: url.to_string(),
                snippet,
                source,
            });
        }
    }

    references
}

fn map_http_error(status: StatusCode, body: String, retry_after: Option<Duration>) -> AgentError {
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|json| {
            json.get("error")
                .and_then(|err| err.get("message"))
                .and_then(|msg| msg.as_str())
                .map(|msg| msg.to_string())
        })
        .unwrap_or_else(|| body.clone());

    if let Some(delay) = retry_after {
        return AgentError::process_error_with_retry_after(
            status.as_u16(),
            message,
            is_retryable_status(status),
            delay,
        );
    }

    AgentError::ProcessError {
        status_code: Some(status.as_u16()),
        message,
        is_retryable: is_retryable_status(status),
        retry_after: None,
    }
}

fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::TOO_MANY_REQUESTS
            | StatusCode::INTERNAL_SERVER_ERROR
            | StatusCode::BAD_GATEWAY
            | StatusCode::SERVICE_UNAVAILABLE
            | StatusCode::GATEWAY_TIMEOUT
    )
}

fn parse_retry_after(header: &reqwest::header::HeaderValue) -> Option<Duration> {
    let value = header.to_str().ok()?;
    value.parse::<u64>().ok().map(Duration::from_secs)
}
