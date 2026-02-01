//! KaibaMemorySyncService - Memory synchronization with Kaiba API.
//!
//! This module provides memory synchronization functionality between Orcs sessions
//! and the Kaiba memory system (Qdrant + Neo4j).

use async_trait::async_trait;
use orcs_core::memory::{MemoryMessage, MemorySyncService, SyncResult};
use orcs_core::secret::SecretService;
use orcs_infrastructure::SecretServiceImpl;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

const DEFAULT_KAIBA_URL: &str = "https://kaiba.shuttleapp.rs";

/// Memory synchronization service that talks to the Kaiba API.
#[derive(Clone)]
pub struct KaibaMemorySyncService {
    client: Client,
    kaiba_url: String,
    kaiba_api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateMemoryRequest {
    content: String,
    memory_type: String,
    importance: f32,
    tags: Vec<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct SearchMemoriesRequest {
    query: String,
    limit: usize,
}

#[derive(Debug, Deserialize)]
struct MemoryResponse {
    id: String,
    content: String,
    memory_type: String,
    #[allow(dead_code)]
    importance: f32,
    #[allow(dead_code)]
    tags: Vec<String>,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct CreateReiRequest {
    name: String,
    role: String,
    manifest: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ReiResponse {
    id: String,
}

impl KaibaMemorySyncService {
    /// Creates a new KaibaMemorySyncService with explicit configuration.
    pub fn new(kaiba_url: impl Into<String>, kaiba_api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            kaiba_url: kaiba_url.into(),
            kaiba_api_key,
        }
    }

    /// Loads configuration from secret.json or environment variables.
    ///
    /// Priority:
    /// 1. secret.json
    /// 2. Environment variables (KAIBA_URL, KAIBA_API_KEY)
    ///
    /// Returns error if no Kaiba configuration is found (API key is required).
    pub async fn try_from_env() -> Result<Self, String> {
        tracing::debug!("[KaibaMemorySync] try_from_env: Starting configuration lookup");

        // Try secret.json first
        let from_secret_json = match SecretServiceImpl::new_default() {
            Ok(service) => {
                tracing::debug!("[KaibaMemorySync] SecretServiceImpl created successfully");
                match service.load_secrets().await {
                    Ok(secret_config) => {
                        tracing::debug!(
                            "[KaibaMemorySync] Secrets loaded, kaiba config present: {}",
                            secret_config.kaiba.is_some()
                        );
                        secret_config.kaiba.map(|kaiba_config| {
                            let kaiba_url = kaiba_config
                                .url
                                .clone()
                                .unwrap_or_else(|| DEFAULT_KAIBA_URL.to_string());
                            let kaiba_api_key = kaiba_config.api_key.clone();
                            (kaiba_url, kaiba_api_key)
                        })
                    }
                    Err(e) => {
                        tracing::debug!("[KaibaMemorySync] Failed to load secrets: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::debug!(
                    "[KaibaMemorySync] Failed to create SecretServiceImpl: {}",
                    e
                );
                None
            }
        };

        let (kaiba_url, kaiba_api_key) = if let Some((url, api_key)) = from_secret_json {
            (url, api_key)
        } else if let Ok(kaiba_api_key) = env::var("KAIBA_API_KEY") {
            // Fallback to environment variables
            tracing::debug!("[KaibaMemorySync] Using environment variables for configuration");
            let kaiba_url = env::var("KAIBA_URL").unwrap_or_else(|_| DEFAULT_KAIBA_URL.to_string());
            (kaiba_url, Some(kaiba_api_key))
        } else {
            tracing::debug!(
                "[KaibaMemorySync] No configuration found in secret.json or environment variables"
            );
            return Err(
                "No Kaiba configuration found in secret.json or environment variables".to_string(),
            );
        };

        tracing::info!(
            "[KaibaMemorySync] Initialized with URL: {}, API key: {}",
            kaiba_url,
            if kaiba_api_key.is_some() {
                "present"
            } else {
                "none"
            }
        );

        Ok(Self::new(kaiba_url, kaiba_api_key))
    }

    /// Makes an authenticated request to the Kaiba API.
    fn auth_request(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.kaiba_api_key {
            request.header("Authorization", format!("Bearer {}", api_key))
        } else {
            request
        }
    }
}

#[async_trait]
impl MemorySyncService for KaibaMemorySyncService {
    async fn ensure_rei_exists(&self, rei_id: &str, workspace_name: &str) -> Result<(), String> {
        // First, check if Rei exists
        let check_url = format!("{}/kaiba/rei/{}", self.kaiba_url, rei_id);
        let check_request =
            self.auth_request(self.client.get(&check_url).timeout(Duration::from_secs(10)));

        match check_request.send().await {
            Ok(response) if response.status().is_success() => {
                // Rei already exists
                tracing::debug!("[KaibaMemorySync] Rei {} already exists", rei_id);
                return Ok(());
            }
            Ok(response) if response.status().as_u16() == 404 => {
                // Rei doesn't exist, create it
                tracing::info!(
                    "[KaibaMemorySync] Rei {} not found, creating for workspace {}",
                    rei_id,
                    workspace_name
                );
            }
            Ok(response) => {
                // Other error
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("Failed to check Rei existence: {}", error_text));
            }
            Err(e) => {
                return Err(format!("Failed to check Rei existence: {}", e));
            }
        }

        // Create the Rei with the predetermined ID using PUT (upsert)
        let create_url = format!("{}/kaiba/rei/{}", self.kaiba_url, rei_id);

        let manifest = serde_json::json!({
            "source": "orcs",
            "type": "workspace_memory",
            "auto_created": true,
        });

        let request_body = CreateReiRequest {
            name: format!("orcs-{}", workspace_name),
            role: format!("Memory store for ORCS workspace: {}", workspace_name),
            manifest,
        };

        let create_request = self.auth_request(
            self.client
                .put(&create_url)
                .json(&request_body)
                .timeout(Duration::from_secs(10)),
        );

        match create_request.send().await {
            Ok(response) if response.status().is_success() => {
                tracing::info!(
                    "[KaibaMemorySync] Created Rei {} for workspace {}",
                    rei_id,
                    workspace_name
                );
                Ok(())
            }
            Ok(response) => {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Failed to create Rei: {}", error_text))
            }
            Err(e) => Err(format!("Failed to create Rei: {}", e)),
        }
    }

    async fn sync_messages(&self, rei_id: &str, messages: Vec<MemoryMessage>) -> SyncResult {
        if messages.is_empty() {
            return SyncResult::success(0);
        }

        let mut synced_count = 0;
        let mut failed_count = 0;
        let mut last_error: Option<String> = None;

        for msg in messages {
            let url = format!("{}/kaiba/rei/{}/memories", self.kaiba_url, rei_id);

            // Build metadata with source information
            let metadata = serde_json::json!({
                "source": "orcs",
                "session_id": msg.session_id,
                "workspace_id": msg.workspace_id,
                "role": msg.role,
                "persona_id": msg.persona_id,
                "original_timestamp": msg.timestamp,
            });

            let request_body = CreateMemoryRequest {
                content: msg.content,
                memory_type: "conversation".to_string(),
                importance: 0.6, // Default importance for conversation messages
                tags: msg.tags,
                metadata: Some(metadata),
            };

            let request = self
                .client
                .post(&url)
                .json(&request_body)
                .timeout(Duration::from_secs(10));

            let request = self.auth_request(request);

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        synced_count += 1;
                    } else {
                        failed_count += 1;
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());
                        last_error = Some(format!("Kaiba API error: {}", error_text));
                        tracing::warn!(
                            "[KaibaMemorySync] Failed to sync message: {:?}",
                            last_error
                        );
                    }
                }
                Err(e) => {
                    failed_count += 1;
                    last_error = Some(format!("Request failed: {}", e));
                    tracing::warn!("[KaibaMemorySync] Request failed: {}", e);
                }
            }
        }

        SyncResult {
            synced_count,
            failed_count,
            error: last_error,
        }
    }

    async fn search_memories(
        &self,
        rei_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryMessage>, String> {
        let url = format!("{}/kaiba/rei/{}/memories/search", self.kaiba_url, rei_id);

        let request_body = SearchMemoriesRequest {
            query: query.to_string(),
            limit,
        };

        let request = self
            .client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(30));

        let request = self.auth_request(request);

        let response = request.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Kaiba search failed: {}", error_text));
        }

        let memories: Vec<MemoryResponse> = response.json().await.map_err(|e| e.to_string())?;

        Ok(memories
            .into_iter()
            .map(|m| MemoryMessage {
                id: m.id,
                session_id: String::new(), // Not returned from search
                workspace_id: String::new(),
                role: m.memory_type,
                content: m.content,
                timestamp: m.created_at,
                persona_id: None,
                tags: vec![],
            })
            .collect())
    }

    async fn get_or_create_rei(
        &self,
        workspace_id: &str,
        workspace_name: &str,
    ) -> Result<String, String> {
        // First, try to find existing Rei by searching for workspace_id in manifest
        // For now, we'll create a new Rei with workspace-specific naming
        // In the future, we could add a lookup endpoint to Kaiba

        let url = format!("{}/kaiba/rei", self.kaiba_url);

        let manifest = serde_json::json!({
            "source": "orcs",
            "workspace_id": workspace_id,
            "type": "workspace_memory",
        });

        let request_body = CreateReiRequest {
            name: format!("orcs-{}", workspace_name),
            role: format!("Memory for ORCS workspace: {}", workspace_name),
            manifest,
        };

        let request = self
            .client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(10));

        let request = self.auth_request(request);

        let response = request.send().await.map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Failed to create Rei: {}", error_text));
        }

        let rei: ReiResponse = response.json().await.map_err(|e| e.to_string())?;

        tracing::info!(
            "[KaibaMemorySync] Created/retrieved Rei {} for workspace {}",
            rei.id,
            workspace_name
        );

        Ok(rei.id)
    }
}
