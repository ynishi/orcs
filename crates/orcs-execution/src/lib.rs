use async_trait::async_trait;
use chrono::Utc;
use llm_toolkit::agent::impls::claude_code::{ClaudeCodeAgent, ClaudeCodeJsonAgent};
use llm_toolkit::agent::impls::RetryAgent;
use llm_toolkit::agent::{Agent, AgentError, AgentOutput, Payload};
use llm_toolkit::orchestrator::{BlueprintWorkflow, ParallelOrchestrator};
use orcs_application::UtilityAgentService;
use orcs_core::OrcsError;
use orcs_core::repository::TaskRepository;
use orcs_core::task::{Task, TaskContext, TaskStatus};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod tracing_layer;

/// Builds an enhanced PATH environment variable that includes workspace-specific tool directories
/// and system binary paths.
fn build_enhanced_path(workspace_root: &Path) -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    let mut path_components = Vec::new();

    // 1. Add workspace-specific tool directories (highest priority)
    let workspace_tool_dirs = vec![
        workspace_root.join("node_modules/.bin"), // npm/yarn
        workspace_root.join(".venv/bin"),         // Python venv
        workspace_root.join("target/debug"),      // Rust debug builds
        workspace_root.join("target/release"),    // Rust release builds
        workspace_root.join("bin"),               // Generic bin
    ];

    for dir in workspace_tool_dirs {
        if dir.exists() {
            if let Some(dir_str) = dir.to_str() {
                if !path_components.contains(&dir_str.to_string()) {
                    path_components.push(dir_str.to_string());
                }
            }
        }
    }

    // 2. Read system paths from /etc/paths (macOS/Linux)
    #[cfg(unix)]
    {
        if let Ok(contents) = std::fs::read_to_string("/etc/paths") {
            for line in contents.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !path_components.contains(&trimmed.to_string()) {
                    path_components.push(trimmed.to_string());
                }
            }
        }

        // Read from /etc/paths.d/*
        if let Ok(entries) = std::fs::read_dir("/etc/paths.d") {
            for entry in entries.flatten() {
                if let Ok(contents) = std::fs::read_to_string(entry.path()) {
                    for line in contents.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() && !path_components.contains(&trimmed.to_string()) {
                            path_components.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Add common binary locations
    let common_paths = vec![
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
        "/opt/homebrew/bin", // Apple Silicon Homebrew
        "/usr/local/opt",
    ];

    for path in common_paths {
        if !path_components.contains(&path.to_string()) {
            path_components.push(path.to_string());
        }
    }

    // 4. Add user's home bin directories
    if let Ok(home) = std::env::var("HOME") {
        use std::path::PathBuf;
        let home_paths = vec![
            PathBuf::from(&home).join(".local/bin"),
            PathBuf::from(&home).join("bin"),
        ];

        for path in home_paths {
            if path.exists() {
                if let Some(path_str) = path.to_str() {
                    if !path_components.contains(&path_str.to_string()) {
                        path_components.push(path_str.to_string());
                    }
                }
            }
        }
    }

    // 5. Preserve any existing PATH entries that aren't already included
    if !current_path.is_empty() {
        for existing in current_path.split(':') {
            if !existing.is_empty() && !path_components.contains(&existing.to_string()) {
                path_components.push(existing.to_string());
            }
        }
    }

    path_components.join(":")
}

/// Dynamic agent adapter for ParallelOrchestrator.
///
/// Wraps any Agent<Output = String> to make it compatible with DynamicAgent trait.
struct DynamicAgentAdapter {
    agent: Arc<dyn Agent<Output = String> + Send + Sync>,
    name: String,
}

impl DynamicAgentAdapter {
    fn new(agent: Arc<dyn Agent<Output = String> + Send + Sync>, name: String) -> Self {
        Self { agent, name }
    }
}

#[async_trait]
impl llm_toolkit::agent::DynamicAgent for DynamicAgentAdapter {
    async fn execute_dynamic(&self, intent: Payload) -> Result<AgentOutput, AgentError> {
        let result = self.agent.execute(intent).await?;
        Ok(AgentOutput::Success(JsonValue::String(result)))
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn expertise(&self) -> &str {
        self.agent.expertise()
    }
}

/// Responsible for executing a single task.
///
/// This struct implements task execution logic using ParallelOrchestrator.
pub struct TaskExecutor {
    agent: Arc<dyn Agent<Output = String> + Send + Sync>,
    task_repository: Option<Arc<dyn TaskRepository>>,
    event_sender: Option<mpsc::UnboundedSender<tracing_layer::OrchestratorEvent>>,
    utility_service: Option<Arc<UtilityAgentService>>,
}

impl TaskExecutor {
    /// Creates a new `TaskExecutor` instance with ClaudeCodeAgent.
    pub fn new() -> Self {
        Self {
            agent: Arc::new(ClaudeCodeAgent::new()),
            task_repository: None,
            event_sender: None,
            utility_service: None,
        }
    }

    /// Creates a new `TaskExecutor` instance with a custom agent.
    pub fn with_agent(agent: Arc<dyn Agent<Output = String> + Send + Sync>) -> Self {
        Self {
            agent,
            task_repository: None,
            event_sender: None,
            utility_service: None,
        }
    }

    /// Sets the task repository for persisting task execution records.
    pub fn with_task_repository(mut self, repository: Arc<dyn TaskRepository>) -> Self {
        self.task_repository = Some(repository);
        self
    }

    /// Sets the event sender for streaming orchestrator events.
    pub fn with_event_sender(
        mut self,
        sender: mpsc::UnboundedSender<tracing_layer::OrchestratorEvent>,
    ) -> Self {
        self.event_sender = Some(sender);
        self
    }

    /// Sets the utility agent service for lightweight LLM operations.
    pub fn with_utility_service(mut self, service: Arc<UtilityAgentService>) -> Self {
        self.utility_service = Some(service);
        self
    }

    /// Executes a task based on the provided context.
    ///
    /// # Arguments
    ///
    /// * `_task_context` - Reference to the task context containing execution details
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the task executes successfully
    /// * `Err(OrcsError)` if an error occurs during execution
    pub async fn execute(&self, _task_context: &TaskContext) -> Result<(), OrcsError> {
        // TODO: Implement actual ParallelOrchestrator logic here
        // This will include:
        // - Parsing task steps
        // - Managing parallel execution
        // - Handling dependencies
        // - Error recovery

        Ok(())
    }

    /// Executes a message content as a task using ParallelOrchestrator.
    ///
    /// This Phase 2 implementation uses real ParallelOrchestrator for workflow execution.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID where this task is being executed
    /// * `message_content` - The message content to execute as a task
    /// * `workspace_root` - Optional workspace root path where the task should execute
    ///
    /// # Returns
    ///
    /// * `Ok(String)` with the execution result summary
    /// * `Err(OrcsError)` if an error occurs during execution
    pub async fn execute_from_message(
        &self,
        session_id: String,
        message_content: String,
        workspace_root: Option<std::path::PathBuf>,
    ) -> Result<String, OrcsError> {
        tracing::info!("TaskExecutor: Executing task from message with ParallelOrchestrator");
        tracing::debug!(
            "Task content: {}",
            message_content.chars().take(200).collect::<String>()
        );

        if let Some(ref root) = workspace_root {
            tracing::info!("Task will execute in workspace: {}", root.display());
        } else {
            tracing::info!("Task will execute without specific workspace root");
        }

        // Create agent with workspace_root and enhanced PATH if provided
        let agent = if let Some(ref workspace) = workspace_root {
            tracing::info!(
                "[TaskExecutor] Creating ClaudeCodeAgent with workspace_root: {}",
                workspace.display()
            );
            let enhanced_path = build_enhanced_path(workspace);
            Arc::new(
                ClaudeCodeAgent::new()
                    .with_cwd(workspace.clone())
                    .with_env("PATH", enhanced_path),
            ) as Arc<dyn Agent<Output = String> + Send + Sync>
        } else {
            self.agent.clone()
        };

        // Generate task ID and timestamps
        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        // Generate title using utility service if available, otherwise fallback
        let title = if let Some(utility) = &self.utility_service {
            utility
                .generate_task_title(&message_content)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to generate task title: {}, using fallback", e);
                    message_content
                        .chars()
                        .take(100)
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
        } else {
            message_content
                .chars()
                .take(100)
                .collect::<String>()
                .trim()
                .to_string()
        };

        // Create initial task record
        let mut task = Task {
            id: task_id.clone(),
            session_id,
            title,
            description: message_content.clone(),
            status: TaskStatus::Running,
            created_at: now.clone(),
            updated_at: now.clone(),
            completed_at: None,
            steps_executed: 0,
            steps_skipped: 0,
            context_keys: 0,
            error: None,
            result: None,
            execution_details: None,
        };

        // Save initial task record if repository is available
        if let Some(repo) = &self.task_repository {
            if let Err(e) = repo.save(&task).await {
                tracing::warn!("Failed to save initial task record: {}", e);
            }
        }

        // Send task started event
        if let Some(sender) = &self.event_sender {
            let event = tracing_layer::OrchestratorEvent {
                target: "orcs_execution::task_executor".to_string(),
                level: "INFO".to_string(),
                message: "Task execution started".to_string(),
                fields: serde_json::json!({
                    "task_id": &task_id,
                    "session_id": &task.session_id,
                    "title": &task.title,
                })
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v))
                .collect(),
                span: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = sender.send(event);
        }

        // Create a blueprint using the message content as the workflow description
        let blueprint = BlueprintWorkflow::new(message_content.clone());

        // Initialize ParallelOrchestrator with workspace-aware internal agents
        // This ensures Strategy generation happens in the correct workspace context
        let mut orchestrator = if let Some(ref workspace) = workspace_root {
            tracing::info!(
                "[TaskExecutor] Configuring ParallelOrchestrator internal agents with workspace: {}",
                workspace.display()
            );
            let enhanced_path = build_enhanced_path(workspace);

            // Configure internal_agent (String output, for redesign decisions)
            let internal_agent = ClaudeCodeAgent::new()
                .with_cwd(workspace.clone())
                .with_env("PATH", enhanced_path.clone());

            // Configure internal_json_agent (StrategyMap output, for strategy generation)
            let internal_json_agent = ClaudeCodeJsonAgent::new()
                .with_cwd(workspace.clone())
                .with_env("PATH", enhanced_path.clone());

            ParallelOrchestrator::with_internal_agents(
                blueprint,
                Box::new(RetryAgent::new(internal_agent, 3)),
                Box::new(RetryAgent::new(internal_json_agent, 3)),
            )
        } else {
            tracing::info!("[TaskExecutor] Using default ParallelOrchestrator (no workspace context)");
            ParallelOrchestrator::new(blueprint)
        };

        // Register our executor agent as a DynamicAgent (with workspace context if provided)
        let executor_agent = Arc::new(DynamicAgentAdapter::new(
            agent.clone(),
            "executor".to_string(),
        ));
        orchestrator.add_agent("executor", executor_agent);

        // Execute the task
        let cancellation_token = CancellationToken::new();
        let result = orchestrator
            .execute(&message_content, cancellation_token, None, None)
            .await
            .map_err(|e| OrcsError::Execution(format!("Orchestrator execution failed: {}", e)))?;

        // Update task record with result
        let completed_at = Utc::now().to_rfc3339();
        task.updated_at = completed_at.clone();
        task.steps_executed = result.steps_executed as u32;
        task.steps_skipped = result.steps_skipped as u32;
        task.context_keys = result.context.keys().len() as u32;

        if result.success {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(completed_at);

            let summary = format!(
                "✅ Task completed successfully!\n\
                 Steps executed: {}\n\
                 Steps skipped: {}\n\
                 Context keys: {}",
                result.steps_executed,
                result.steps_skipped,
                result.context.keys().len()
            );

            // Extract result from context if available
            let result_text = if let Some(execute_result) = result.context.get("execute") {
                tracing::debug!("Execution result: {:?}", execute_result);
                format!("{}\n\nResult: {}", summary, execute_result)
            } else {
                summary.clone()
            };

            task.result = Some(summary);

            // Save execution details with context outputs
            task.execution_details = Some(orcs_core::task::ExecutionDetails {
                steps: vec![], // TODO: Extract step info from orchestrator
                context: result.context.clone(),
            });

            // Save final task record
            if let Some(repo) = &self.task_repository {
                if let Err(e) = repo.save(&task).await {
                    tracing::warn!("Failed to save completed task record: {}", e);
                }
            }

            // Send task completed event
            if let Some(sender) = &self.event_sender {
                let event = tracing_layer::OrchestratorEvent {
                    target: "orcs_execution::task_executor".to_string(),
                    level: "INFO".to_string(),
                    message: "Task execution completed".to_string(),
                    fields: serde_json::json!({
                        "task_id": &task_id,
                        "session_id": &task.session_id,
                        "status": "Completed",
                        "steps_executed": task.steps_executed,
                        "steps_skipped": task.steps_skipped,
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
                    span: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let _ = sender.send(event);
            }

            Ok(result_text)
        } else {
            let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
            task.status = TaskStatus::Failed;
            task.error = Some(error_msg.clone());
            task.completed_at = Some(completed_at);

            // Save execution details with context outputs (even on failure)
            task.execution_details = Some(orcs_core::task::ExecutionDetails {
                steps: vec![], // TODO: Extract step info from orchestrator
                context: result.context.clone(),
            });

            // Save failed task record
            if let Some(repo) = &self.task_repository {
                if let Err(e) = repo.save(&task).await {
                    tracing::warn!("Failed to save failed task record: {}", e);
                }
            }

            // Send task failed event
            if let Some(sender) = &self.event_sender {
                let event = tracing_layer::OrchestratorEvent {
                    target: "orcs_execution::task_executor".to_string(),
                    level: "ERROR".to_string(),
                    message: "Task execution failed".to_string(),
                    fields: serde_json::json!({
                        "task_id": &task_id,
                        "session_id": &task.session_id,
                        "status": "Failed",
                        "error": &error_msg,
                        "steps_executed": task.steps_executed,
                        "steps_skipped": task.steps_skipped,
                    })
                    .as_object()
                    .unwrap()
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
                    span: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let _ = sender.send(event);
            }

            Err(OrcsError::Execution(format!(
                "Task execution failed: {}",
                error_msg
            )))
        }
    }
}
