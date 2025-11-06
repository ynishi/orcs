use async_trait::async_trait;
use chrono::Utc;
use llm_toolkit::agent::impls::ClaudeCodeAgent;
use llm_toolkit::agent::{Agent, AgentError, AgentOutput, Payload};
use llm_toolkit::orchestrator::{BlueprintWorkflow, ParallelOrchestrator};
use orcs_core::repository::TaskRepository;
use orcs_core::task::{Task, TaskContext, TaskStatus};
use orcs_core::OrcsError;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod tracing_layer;

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
}

impl TaskExecutor {
    /// Creates a new `TaskExecutor` instance with ClaudeCodeAgent.
    pub fn new() -> Self {
        Self {
            agent: Arc::new(ClaudeCodeAgent::new()),
            task_repository: None,
            event_sender: None,
        }
    }

    /// Creates a new `TaskExecutor` instance with a custom agent.
    pub fn with_agent(agent: Arc<dyn Agent<Output = String> + Send + Sync>) -> Self {
        Self {
            agent,
            task_repository: None,
            event_sender: None,
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
    ///
    /// # Returns
    ///
    /// * `Ok(String)` with the execution result summary
    /// * `Err(OrcsError)` if an error occurs during execution
    pub async fn execute_from_message(
        &self,
        session_id: String,
        message_content: String,
    ) -> Result<String, OrcsError> {
        tracing::info!("TaskExecutor: Executing task from message with ParallelOrchestrator");
        tracing::debug!("Task content: {}", message_content.chars().take(200).collect::<String>());

        // Generate task ID and timestamps
        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let title = message_content
            .chars()
            .take(100)
            .collect::<String>()
            .trim()
            .to_string();

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
                }).as_object().unwrap().clone().into_iter()
                    .map(|(k, v)| (k, v))
                    .collect(),
                span: std::collections::HashMap::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = sender.send(event);
        }

        // Create a blueprint using the message content as the workflow description
        let blueprint = BlueprintWorkflow::new(message_content.clone());

        // Initialize ParallelOrchestrator with default internal agents
        let mut orchestrator = ParallelOrchestrator::new(blueprint);

        // Register our executor agent as a DynamicAgent
        let executor_agent = Arc::new(DynamicAgentAdapter::new(
            self.agent.clone(),
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
                    }).as_object().unwrap().clone().into_iter()
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
                    }).as_object().unwrap().clone().into_iter()
                        .map(|(k, v)| (k, v))
                        .collect(),
                    span: std::collections::HashMap::new(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let _ = sender.send(event);
            }

            Err(OrcsError::Execution(format!("Task execution failed: {}", error_msg)))
        }
    }
}
