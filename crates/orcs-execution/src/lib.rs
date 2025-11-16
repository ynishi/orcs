use async_trait::async_trait;
use chrono::Utc;
use llm_toolkit::agent::impls::RetryAgent;
use llm_toolkit::agent::impls::claude_code::{ClaudeCodeAgent, ClaudeCodeJsonAgent};
use llm_toolkit::agent::{Agent, AgentError, AgentOutput, Payload};
use llm_toolkit::orchestrator::{BlueprintWorkflow, ParallelOrchestrator};
use orcs_application::UtilityAgentService;
use orcs_core::OrcsError;
use orcs_core::agent::build_enhanced_path;
use orcs_core::repository::TaskRepository;
use orcs_core::task::{Task, TaskContext, TaskStatus};
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

        // Use fallback title immediately for fast UI display
        let fallback_title = message_content
            .chars()
            .take(100)
            .collect::<String>()
            .trim()
            .to_string();

        // Create initial task record with Pending status and fallback title
        let mut task = Task {
            id: task_id.clone(),
            session_id,
            title: fallback_title.clone(), // Temporary title
            description: message_content.clone(),
            status: TaskStatus::Pending,
            created_at: now.clone(),
            updated_at: now.clone(),
            completed_at: None,
            steps_executed: 0,
            steps_skipped: 0,
            context_keys: 0,
            error: None,
            result: None,
            execution_details: None,
            strategy: None,
            journal_log: None,
        };

        // 🚀 STEP 1: Save immediately with Pending status (for instant UI display)
        if let Some(repo) = &self.task_repository {
            if let Err(e) = repo.save(&task).await {
                tracing::warn!("Failed to save initial task record: {}", e);
            }
        }

        // Send task created event
        if let Some(sender) = &self.event_sender {
            let event =
                tracing_layer::OrchestratorEventBuilder::info_from_task("Task created", &task)
                    .build();
            match sender.send(event) {
                Ok(_) => eprintln!("[TaskExecutor] Event sent successfully"),
                Err(e) => eprintln!("[TaskExecutor] Failed to send event: {:?}", e),
            }
            tokio::task::yield_now().await;
        }

        // 📝 STEP 2: Generate better title using LLM (can take time, but doesn't block UI)
        let generated_title = if let Some(utility) = &self.utility_service {
            utility
                .generate_task_title(&message_content)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to generate task title: {}, using fallback", e);
                    fallback_title.clone()
                })
        } else {
            fallback_title.clone()
        };

        // Update title (keep Pending status until Planning phase starts)
        task.title = generated_title;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        if let Some(repo) = &self.task_repository {
            if let Err(e) = repo.save(&task).await {
                tracing::warn!("Failed to update task title: {}", e);
            }
        }

        task.status = TaskStatus::Running;
        task.updated_at = chrono::Utc::now().to_rfc3339();
        if let Some(repo) = &self.task_repository {
            if let Err(e) = repo.save(&task).await {
                tracing::warn!("Failed to update task to Running: {}", e);
            }
        }

        if let Some(sender) = &self.event_sender {
            let event = tracing_layer::OrchestratorEventBuilder::info_from_task(
                "Task execution started",
                &task,
            )
            .build();
            match sender.send(event) {
                Ok(_) => eprintln!("[TaskExecutor] Event sent successfully"),
                Err(e) => eprintln!("[TaskExecutor] Failed to send event: {:?}", e),
            }
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
            tracing::info!(
                "[TaskExecutor] Using default ParallelOrchestrator (no workspace context)"
            );
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

            // Extract strategy and journal log from orchestrator
            task.strategy = orchestrator
                .strategy_map()
                .and_then(|s| serde_json::to_string_pretty(s).ok());
            task.journal_log = orchestrator
                .execution_journal()
                .and_then(|j| serde_json::to_string_pretty(j).ok());

            // Save final task record
            if let Some(repo) = &self.task_repository {
                if let Err(e) = repo.save(&task).await {
                    tracing::warn!("Failed to save completed task record: {}", e);
                }
            }

            // Send task completed event
            if let Some(sender) = &self.event_sender {
                let event = tracing_layer::OrchestratorEventBuilder::info_from_task(
                    "Task execution completed",
                    &task,
                )
                .build();
                match sender.send(event) {
                    Ok(_) => eprintln!("[TaskExecutor] Event sent successfully"),
                    Err(e) => eprintln!("[TaskExecutor] Failed to send event: {:?}", e),
                }
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

            // Extract strategy and journal log from orchestrator (even on failure)
            task.strategy = orchestrator
                .strategy_map()
                .and_then(|s| serde_json::to_string_pretty(s).ok());
            task.journal_log = orchestrator
                .execution_journal()
                .and_then(|j| serde_json::to_string_pretty(j).ok());

            // Save failed task record
            if let Some(repo) = &self.task_repository {
                if let Err(e) = repo.save(&task).await {
                    tracing::warn!("Failed to save failed task record: {}", e);
                }
            }

            // Send task failed event
            if let Some(sender) = &self.event_sender {
                let event = tracing_layer::OrchestratorEventBuilder::error_from_task(
                    "Task execution failed",
                    &task,
                )
                .build();
                match sender.send(event) {
                    Ok(_) => eprintln!("[TaskExecutor] Event sent successfully"),
                    Err(e) => eprintln!("[TaskExecutor] Failed to send event: {:?}", e),
                }
            }

            Err(OrcsError::Execution(format!(
                "Task execution failed: {}",
                error_msg
            )))
        }
    }
}
