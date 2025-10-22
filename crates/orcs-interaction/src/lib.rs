pub mod persona_agent;
pub mod personas;

use orcs_types::{AppMode, Plan};
use llm_toolkit::agent::impls::ClaudeCodeAgent;
use llm_toolkit::agent::{Agent, Payload};
use crate::persona_agent::PersonaAgent;
use crate::personas::MAI_PERSONA;

/// Result of handling user input in a stateful conversation.
///
/// This enum represents the different outcomes that can occur when processing
/// user input based on the current application mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionResult {
    /// No state change occurred.
    NoOp,
    /// The application mode should be updated to the specified mode.
    ModeChanged(AppMode),
    /// A plan was confirmed and tasks should be dispatched.
    TasksToDispatch {
        /// The list of tasks to be dispatched.
        tasks: Vec<String>,
    },
    /// A new message to be displayed to the user.
    NewMessage(String),
}

/// Manages user interaction and input parsing for the Orcs system.
///
/// The `InteractionManager` is now responsible for stateful conversation management,
/// handling mode transitions and plan confirmation.
pub struct InteractionManager {
    agent: PersonaAgent<ClaudeCodeAgent>,
}

impl InteractionManager {
    /// Creates a new instance of `InteractionManager`.
    pub fn new() -> Self {
        let base_agent = ClaudeCodeAgent::new();
        let persona = MAI_PERSONA.clone();
        let agent = PersonaAgent::new(base_agent, persona);
        Self { agent }
    }

    /// Handles user input based on the current application mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The current application mode
    /// * `input` - A string slice containing the raw user input
    ///
    /// # Returns
    ///
    /// Returns an `InteractionResult` indicating the action to be taken:
    /// - `NoOp`: No state change occurred
    /// - `ModeChanged`: The application mode should transition to a new state
    /// - `TasksToDispatch`: A plan was confirmed and tasks should be dispatched
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use orcs_interaction::{InteractionManager, InteractionResult};
    /// use orcs_types::{AppMode, Plan};
    ///
    /// # async fn example() {
    /// let manager = InteractionManager::new();
    ///
    /// // In Idle mode, /plan command creates a plan
    /// let result = manager.handle_input(&AppMode::Idle, "/plan").await;
    /// assert!(matches!(result, InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { .. })));
    ///
    /// // In AwaitingConfirmation mode, "yes" dispatches tasks
    /// let plan = Plan { steps: vec!["Step 1".to_string()] };
    /// let result = manager.handle_input(&AppMode::AwaitingConfirmation { plan }, "yes").await;
    /// assert!(matches!(result, InteractionResult::TasksToDispatch { .. }));
    /// # }
    /// ```
    pub async fn handle_input(&self, mode: &AppMode, input: &str) -> InteractionResult {
        match mode {
            AppMode::Idle => self.handle_idle_mode(input).await,
            AppMode::AwaitingConfirmation { plan } => self.handle_awaiting_confirmation(input, plan),
        }
    }

    /// Handles user input when the application is in Idle mode.
    async fn handle_idle_mode(&self, input: &str) -> InteractionResult {
        let trimmed = input.trim();

        if trimmed == "/plan" {
            // Create a dummy plan with sample steps
            let plan = Plan {
                steps: vec![
                    "Step 1: Refactor module A".to_string(),
                    "Step 2: Add tests for B".to_string(),
                ],
            };
            InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { plan })
        } else {
            let payload = Payload::text(input);
            let result = self.agent.execute(payload).await;
            match result {
                Ok(response) => InteractionResult::NewMessage(format!("{}: {}", self.agent.persona.name, response)),
                Err(e) => InteractionResult::NewMessage(format!("Error: {}", e)),
            }
        }
    }

    /// Handles user input when the application is awaiting plan confirmation.
    fn handle_awaiting_confirmation(&self, input: &str, plan: &Plan) -> InteractionResult {
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "yes" | "y" => {
                InteractionResult::TasksToDispatch {
                    tasks: plan.steps.clone(),
                }
            }
            "no" | "n" => {
                InteractionResult::ModeChanged(AppMode::Idle)
            }
            _ => InteractionResult::NoOp,
        }
    }
}

impl Default for InteractionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_idle_mode_plan_command() {
        let manager = InteractionManager::new();
        let mode = AppMode::Idle;

        // Test /plan command in Idle mode
        let result = manager.handle_input(&mode, "/plan").await;

        // Should transition to AwaitingConfirmation with a plan
        match result {
            InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { plan }) => {
                assert_eq!(plan.steps.len(), 2);
                assert_eq!(plan.steps[0], "Step 1: Refactor module A");
                assert_eq!(plan.steps[1], "Step 2: Add tests for B");
            }
            _ => panic!("Expected ModeChanged with AwaitingConfirmation"),
        }
    }

    #[tokio::test]
    async fn test_idle_mode_non_plan_input() {
        let manager = InteractionManager::new();
        let mode = AppMode::Idle;

        // Test regular input in Idle mode
        let result = manager.handle_input(&mode, "Hello, world!").await;
        match result {
            InteractionResult::NewMessage(msg) => {
                assert!(msg.starts_with("Mai: "), "Message should start with 'Mai: ', got: {}", msg);
            }
            _ => panic!("Expected NewMessage"),
        }

        // Test other commands in Idle mode
        let result = manager.handle_input(&mode, "/help").await;
        match result {
            InteractionResult::NewMessage(msg) => {
                assert!(msg.starts_with("Mai: "), "Message should start with 'Mai: ', got: {}", msg);
            }
            _ => panic!("Expected NewMessage"),
        }

        // Test empty input in Idle mode
        let result = manager.handle_input(&mode, "").await;
        match result {
            InteractionResult::NewMessage(msg) => {
                assert!(msg.starts_with("Mai: "), "Message should start with 'Mai: ', got: {}", msg);
            }
            _ => panic!("Expected NewMessage"),
        }
    }

    #[tokio::test]
    async fn test_awaiting_confirmation_yes() {
        let manager = InteractionManager::new();
        let plan = Plan {
            steps: vec![
                "Task 1: Update code".to_string(),
                "Task 2: Run tests".to_string(),
                "Task 3: Deploy".to_string(),
            ],
        };
        let mode = AppMode::AwaitingConfirmation { plan: plan.clone() };

        // Test "yes" response
        let result = manager.handle_input(&mode, "yes").await;
        match result {
            InteractionResult::TasksToDispatch { tasks } => {
                assert_eq!(tasks.len(), 3);
                assert_eq!(tasks[0], "Task 1: Update code");
                assert_eq!(tasks[1], "Task 2: Run tests");
                assert_eq!(tasks[2], "Task 3: Deploy");
            }
            _ => panic!("Expected TasksToDispatch"),
        }

        // Test "y" response (short form)
        let result = manager.handle_input(&mode, "y").await;
        match result {
            InteractionResult::TasksToDispatch { tasks } => {
                assert_eq!(tasks.len(), 3);
            }
            _ => panic!("Expected TasksToDispatch"),
        }

        // Test case insensitivity
        let result = manager.handle_input(&mode, "YES").await;
        assert!(matches!(result, InteractionResult::TasksToDispatch { .. }));

        let result = manager.handle_input(&mode, "Y").await;
        assert!(matches!(result, InteractionResult::TasksToDispatch { .. }));
    }

    #[tokio::test]
    async fn test_awaiting_confirmation_no() {
        let manager = InteractionManager::new();
        let plan = Plan {
            steps: vec!["Task 1".to_string()],
        };
        let mode = AppMode::AwaitingConfirmation { plan };

        // Test "no" response
        let result = manager.handle_input(&mode, "no").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));

        // Test "n" response (short form)
        let result = manager.handle_input(&mode, "n").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));

        // Test case insensitivity
        let result = manager.handle_input(&mode, "NO").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));

        let result = manager.handle_input(&mode, "N").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));
    }

    #[tokio::test]
    async fn test_awaiting_confirmation_invalid_input() {
        let manager = InteractionManager::new();
        let plan = Plan {
            steps: vec!["Task 1".to_string()],
        };
        let mode = AppMode::AwaitingConfirmation { plan };

        // Test invalid responses that should return NoOp
        let result = manager.handle_input(&mode, "maybe").await;
        assert_eq!(result, InteractionResult::NoOp);

        let result = manager.handle_input(&mode, "").await;
        assert_eq!(result, InteractionResult::NoOp);

        let result = manager.handle_input(&mode, "help").await;
        assert_eq!(result, InteractionResult::NoOp);

        let result = manager.handle_input(&mode, "yesno").await;
        assert_eq!(result, InteractionResult::NoOp);
    }

    #[tokio::test]
    async fn test_mode_transitions() {
        let manager = InteractionManager::new();

        // Start in Idle mode
        let mut mode = AppMode::Idle;

        // Trigger plan creation
        let result = manager.handle_input(&mode, "/plan").await;
        match result {
            InteractionResult::ModeChanged(new_mode) => {
                mode = new_mode;
            }
            _ => panic!("Expected mode change"),
        }

        // Verify we're in AwaitingConfirmation
        assert!(matches!(mode, AppMode::AwaitingConfirmation { .. }));

        // Reject the plan
        let result = manager.handle_input(&mode, "no").await;
        match result {
            InteractionResult::ModeChanged(new_mode) => {
                mode = new_mode;
            }
            _ => panic!("Expected mode change back to Idle"),
        }

        // Verify we're back in Idle
        assert_eq!(mode, AppMode::Idle);
    }

    #[tokio::test]
    async fn test_whitespace_handling() {
        let manager = InteractionManager::new();

        // Test /plan with whitespace
        let result = manager.handle_input(&AppMode::Idle, "  /plan  ").await;
        assert!(matches!(
            result,
            InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { .. })
        ));

        let plan = Plan {
            steps: vec!["Task".to_string()],
        };
        let mode = AppMode::AwaitingConfirmation { plan };

        // Test yes/no with whitespace
        let result = manager.handle_input(&mode, "  yes  ").await;
        assert!(matches!(result, InteractionResult::TasksToDispatch { .. }));

        let result = manager.handle_input(&mode, "  no  ").await;
        assert_eq!(result, InteractionResult::ModeChanged(AppMode::Idle));
    }

    #[tokio::test]
    async fn test_default_trait() {
        let manager = InteractionManager::default();
        let result = manager.handle_input(&AppMode::Idle, "/plan").await;
        assert!(matches!(
            result,
            InteractionResult::ModeChanged(AppMode::AwaitingConfirmation { .. })
        ));
    }
}
