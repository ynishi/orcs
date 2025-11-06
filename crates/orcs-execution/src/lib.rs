use async_trait::async_trait;
use llm_toolkit::agent::impls::ClaudeCodeAgent;
use llm_toolkit::agent::{Agent, AgentError, AgentOutput, Payload};
use llm_toolkit::orchestrator::{BlueprintWorkflow, ParallelOrchestrator};
use orcs_core::OrcsError;
use orcs_core::task::TaskContext;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

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
}

impl TaskExecutor {
    /// Creates a new `TaskExecutor` instance with ClaudeCodeAgent.
    pub fn new() -> Self {
        Self {
            agent: Arc::new(ClaudeCodeAgent::new()),
        }
    }

    /// Creates a new `TaskExecutor` instance with a custom agent.
    pub fn with_agent(agent: Arc<dyn Agent<Output = String> + Send + Sync>) -> Self {
        Self { agent }
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
    /// * `message_content` - The message content to execute as a task
    ///
    /// # Returns
    ///
    /// * `Ok(String)` with the execution result summary
    /// * `Err(OrcsError)` if an error occurs during execution
    pub async fn execute_from_message(
        &self,
        message_content: String,
    ) -> Result<String, OrcsError> {
        tracing::info!("TaskExecutor: Executing task from message with ParallelOrchestrator");
        tracing::debug!("Task content: {}", message_content.chars().take(200).collect::<String>());

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

        if result.success {
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
            if let Some(execute_result) = result.context.get("execute") {
                tracing::debug!("Execution result: {:?}", execute_result);
                Ok(format!("{}\n\nResult: {}", summary, execute_result))
            } else {
                Ok(summary)
            }
        } else {
            let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
            Err(OrcsError::Execution(format!("Task execution failed: {}", error_msg)))
        }
    }
}
