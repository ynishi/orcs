//! Slash command domain models.

use serde::{Deserialize, Serialize};

/// Type of slash command execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CommandType {
    /// Expands into a prompt template and sends to AI
    Prompt,
    /// Executes a shell command and shows output
    Shell,
    /// Executes a long-running orchestration task workflow
    Task,
    /// Executes a predefined action directly and returns result (e.g., generate_summary)
    Action,
    /// Executes multiple commands in sequence, passing output between steps
    Pipeline,
}

/// A single step in a pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineStep {
    /// Name of the slash command to execute
    #[serde(rename = "commandName")]
    pub command_name: String,

    /// Optional arguments to pass to the command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<String>,
}

/// Configuration for Pipeline type commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineConfig {
    /// Steps to execute in sequence
    pub steps: Vec<PipelineStep>,

    /// Whether to stop execution on first error (default: true)
    #[serde(default = "default_fail_on_error")]
    #[serde(rename = "failOnError")]
    pub fail_on_error: bool,

    /// Whether to pass previous step's output as input to next step (default: true)
    #[serde(default = "default_chain_output")]
    #[serde(rename = "chainOutput")]
    pub chain_output: bool,
}

fn default_fail_on_error() -> bool {
    true
}

fn default_chain_output() -> bool {
    true
}

/// Configuration for Action type commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionConfig {
    /// Backend to use for execution (e.g., "gemini_api", "claude_api", "open_ai_api")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,

    /// Model name override
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "modelName")]
    pub model_name: Option<String>,

    /// Persona ID to use for execution (Phase 2)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "personaId")]
    pub persona_id: Option<String>,

    /// Gemini thinking level (LOW/MEDIUM/HIGH)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "geminiThinkingLevel")]
    pub gemini_thinking_level: Option<String>,

    /// Enable Gemini Google Search
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "geminiGoogleSearch")]
    pub gemini_google_search: Option<bool>,
}

/// A custom slash command definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    /// Command name (used as /name in chat)
    pub name: String,
    /// Icon to display in UI
    pub icon: String,
    /// Human-readable description
    pub description: String,
    /// Type of command (prompt, shell, or task)
    #[serde(rename = "type")]
    pub command_type: CommandType,
    /// Command content (prompt template, shell command, or task description)
    pub content: String,
    /// Working directory for shell commands (supports variables like {workspace_path})
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "workingDir")]
    pub working_dir: Option<String>,
    /// Optional description of expected arguments (displayed in UI)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "argsDescription")]
    pub args_description: Option<String>,
    /// Task execution strategy blueprint (JSON serialized StrategyMap) for Task type commands
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "taskBlueprint")]
    pub task_blueprint: Option<String>,

    /// Configuration for Action type commands (backend, model, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "actionConfig")]
    pub action_config: Option<ActionConfig>,

    /// Configuration for Pipeline type commands (steps to execute)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pipelineConfig")]
    pub pipeline_config: Option<PipelineConfig>,

    /// Whether to include this command in system prompts for personas.
    /// Default: true for Prompt/Shell/Action, false for Task
    #[serde(default = "default_include_in_system_prompt")]
    #[serde(rename = "includeInSystemPrompt")]
    pub include_in_system_prompt: bool,

    /// Whether this command is marked as favorite.
    #[serde(default)]
    #[serde(rename = "isFavorite")]
    pub is_favorite: bool,

    /// Sort order within favorites (lower = higher priority).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<u32>,
}

/// Default value for include_in_system_prompt (true).
/// Note: Task commands should explicitly set this to false on creation.
fn default_include_in_system_prompt() -> bool {
    true
}

impl SlashCommand {
    /// Creates a new prompt-type slash command.
    pub fn new_prompt(name: String, icon: String, description: String, content: String) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Prompt,
            content,
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: true,
            is_favorite: false,
            sort_order: None,
        }
    }

    /// Creates a new prompt-type slash command with an argument description.
    pub fn new_prompt_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_prompt(name, icon, description, content);
        cmd.args_description = args_description;
        cmd
    }

    /// Creates a new shell-type slash command.
    pub fn new_shell(
        name: String,
        icon: String,
        description: String,
        content: String,
        working_dir: Option<String>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Shell,
            content,
            working_dir,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: true,
            is_favorite: false,
            sort_order: None,
        }
    }

    /// Creates a new shell-type slash command with an argument description.
    pub fn new_shell_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        working_dir: Option<String>,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_shell(name, icon, description, content, working_dir);
        cmd.args_description = args_description;
        cmd
    }

    /// Creates a new task-type slash command.
    /// Note: Task commands default to include_in_system_prompt = false
    pub fn new_task(
        name: String,
        icon: String,
        description: String,
        content: String,
        task_blueprint: Option<String>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Task,
            content,
            working_dir: None,
            args_description: None,
            task_blueprint,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: false, // Task commands excluded by default
            is_favorite: false,
            sort_order: None,
        }
    }

    /// Creates a new task-type slash command with an argument description.
    pub fn new_task_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        task_blueprint: Option<String>,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_task(name, icon, description, content, task_blueprint);
        cmd.args_description = args_description;
        cmd
    }

    /// Creates a new action-type slash command.
    /// Action commands use content as a prompt template with variables like {session_all}, {session_recent}.
    pub fn new_action(
        name: String,
        icon: String,
        description: String,
        content: String,
        action_config: Option<ActionConfig>,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Action,
            content,
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config,
            pipeline_config: None,
            include_in_system_prompt: true,
            is_favorite: false,
            sort_order: None,
        }
    }

    /// Creates a new action-type slash command with an argument description.
    pub fn new_action_with_args(
        name: String,
        icon: String,
        description: String,
        content: String,
        action_config: Option<ActionConfig>,
        args_description: Option<String>,
    ) -> Self {
        let mut cmd = Self::new_action(name, icon, description, content, action_config);
        cmd.args_description = args_description;
        cmd
    }

    /// Creates a new pipeline-type slash command.
    /// Pipeline commands execute multiple slash commands in sequence.
    pub fn new_pipeline(
        name: String,
        icon: String,
        description: String,
        pipeline_config: PipelineConfig,
    ) -> Self {
        Self {
            name,
            icon,
            description,
            command_type: CommandType::Pipeline,
            content: String::new(), // Pipeline doesn't use content directly
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: Some(pipeline_config),
            include_in_system_prompt: false, // Pipeline commands excluded by default
            is_favorite: false,
            sort_order: None,
        }
    }
}
