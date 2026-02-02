//! Slash command creation and update request models.

use serde::{Deserialize, Serialize};

use super::{ActionConfig, CommandType, PipelineConfig, SlashCommand};

/// Request to create a new slash command.
///
/// This is the unified request model used by both:
/// - SlashCommand `/create-slash-command` (from JSON)
/// - UI Form (from SlashCommandEditorModal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSlashCommandRequest {
    /// Command name (required, used as /name in chat)
    pub name: String,

    /// Icon to display in UI (optional, defaults to ⚡)
    #[serde(default = "default_icon")]
    pub icon: String,

    /// Human-readable description (required)
    pub description: String,

    /// Type of command: prompt, shell, or task (required)
    #[serde(rename = "type")]
    pub command_type: CommandType,

    /// Command content (required)
    pub content: String,

    /// Working directory for shell commands (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "workingDir")]
    pub working_dir: Option<String>,

    /// Optional description of expected arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "argsDescription")]
    pub args_description: Option<String>,

    /// Task execution strategy blueprint (optional, for task type)
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
    /// If not provided, defaults based on command_type:
    /// - Prompt/Shell/Action: true
    /// - Task: false
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "includeInSystemPrompt")]
    pub include_in_system_prompt: Option<bool>,

    /// Whether this command is marked as favorite.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isFavorite")]
    pub is_favorite: Option<bool>,

    /// Sort order within favorites (lower = higher priority).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<u32>,
}

fn default_icon() -> String {
    "⚡".to_string()
}

/// Returns the default value for include_in_system_prompt based on command type.
fn default_include_for_type(command_type: &CommandType) -> bool {
    match command_type {
        CommandType::Task => false,
        _ => true, // Prompt, Shell, Action default to true
    }
}

impl CreateSlashCommandRequest {
    /// Validate the request and return errors if any.
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.trim().is_empty() {
            return Err("Name is required and cannot be empty".to_string());
        }

        // Command name should not contain spaces
        if self.name.contains(' ') {
            return Err("Command name cannot contain spaces".to_string());
        }

        // Command name should not start with slash
        if self.name.starts_with('/') {
            return Err(
                "Command name should not start with '/' (it will be added automatically)"
                    .to_string(),
            );
        }

        // Validate description
        if self.description.trim().is_empty() {
            return Err("Description is required and cannot be empty".to_string());
        }

        // Validate content (required for all types)
        if self.content.trim().is_empty() {
            return Err("Content is required and cannot be empty".to_string());
        }

        Ok(())
    }

    /// Convert this request into a SlashCommand.
    pub fn into_slash_command(self) -> SlashCommand {
        let include_in_system_prompt = self
            .include_in_system_prompt
            .unwrap_or_else(|| default_include_for_type(&self.command_type));

        SlashCommand {
            name: self.name,
            icon: self.icon,
            description: self.description,
            command_type: self.command_type,
            content: self.content,
            working_dir: self.working_dir,
            args_description: self.args_description,
            task_blueprint: self.task_blueprint,
            action_config: self.action_config,
            pipeline_config: self.pipeline_config,
            include_in_system_prompt,
            is_favorite: self.is_favorite.unwrap_or(false),
            sort_order: self.sort_order,
        }
    }

    /// Create a request from an existing SlashCommand (for editing).
    pub fn from_slash_command(cmd: &SlashCommand) -> Self {
        Self {
            name: cmd.name.clone(),
            icon: cmd.icon.clone(),
            description: cmd.description.clone(),
            command_type: cmd.command_type.clone(),
            content: cmd.content.clone(),
            working_dir: cmd.working_dir.clone(),
            args_description: cmd.args_description.clone(),
            task_blueprint: cmd.task_blueprint.clone(),
            action_config: cmd.action_config.clone(),
            pipeline_config: cmd.pipeline_config.clone(),
            include_in_system_prompt: Some(cmd.include_in_system_prompt),
            is_favorite: Some(cmd.is_favorite),
            sort_order: cmd.sort_order,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_success() {
        let req = CreateSlashCommandRequest {
            name: "test".to_string(),
            icon: "🔧".to_string(),
            description: "Test command".to_string(),
            command_type: CommandType::Prompt,
            content: "Test prompt".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let req = CreateSlashCommandRequest {
            name: "".to_string(),
            icon: "⚡".to_string(),
            description: "Test".to_string(),
            command_type: CommandType::Prompt,
            content: "Test".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_name_with_spaces() {
        let req = CreateSlashCommandRequest {
            name: "test command".to_string(),
            icon: "⚡".to_string(),
            description: "Test".to_string(),
            command_type: CommandType::Prompt,
            content: "Test".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_name_with_slash() {
        let req = CreateSlashCommandRequest {
            name: "/test".to_string(),
            icon: "⚡".to_string(),
            description: "Test".to_string(),
            command_type: CommandType::Prompt,
            content: "Test".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_action_requires_content() {
        let req = CreateSlashCommandRequest {
            name: "summary".to_string(),
            icon: "📝".to_string(),
            description: "Generate summary".to_string(),
            command_type: CommandType::Action,
            content: "".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_validate_action_success() {
        let req = CreateSlashCommandRequest {
            name: "summary".to_string(),
            icon: "📝".to_string(),
            description: "Generate summary".to_string(),
            command_type: CommandType::Action,
            content: "{session_all}\n\nSummarize the above conversation.".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_into_slash_command() {
        let req = CreateSlashCommandRequest {
            name: "test".to_string(),
            icon: "🔧".to_string(),
            description: "Test command".to_string(),
            command_type: CommandType::Shell,
            content: "echo hello".to_string(),
            working_dir: Some("/tmp".to_string()),
            args_description: Some("args".to_string()),
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        let cmd = req.into_slash_command();
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.icon, "🔧");
        assert_eq!(cmd.command_type, CommandType::Shell);
        assert!(cmd.include_in_system_prompt); // Shell defaults to true
        assert!(!cmd.is_favorite); // Defaults to false
        assert!(cmd.sort_order.is_none());
    }

    #[test]
    fn test_into_slash_command_task_default() {
        let req = CreateSlashCommandRequest {
            name: "build".to_string(),
            icon: "🔨".to_string(),
            description: "Build project".to_string(),
            command_type: CommandType::Task,
            content: "Build the project".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: None,
            sort_order: None,
        };

        let cmd = req.into_slash_command();
        assert!(!cmd.include_in_system_prompt); // Task defaults to false
    }

    #[test]
    fn test_into_slash_command_explicit_override() {
        let req = CreateSlashCommandRequest {
            name: "build".to_string(),
            icon: "🔨".to_string(),
            description: "Build project".to_string(),
            command_type: CommandType::Task,
            content: "Build the project".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            include_in_system_prompt: Some(true), // Explicitly override to true
            is_favorite: None,
            sort_order: None,
        };

        let cmd = req.into_slash_command();
        assert!(cmd.include_in_system_prompt); // Explicit override
    }

    #[test]
    fn test_into_slash_command_with_favorite() {
        let req = CreateSlashCommandRequest {
            name: "fav-cmd".to_string(),
            icon: "⭐".to_string(),
            description: "Favorite command".to_string(),
            command_type: CommandType::Prompt,
            content: "Test".to_string(),
            working_dir: None,
            args_description: None,
            task_blueprint: None,
            action_config: None,
            pipeline_config: None,
            include_in_system_prompt: None,
            is_favorite: Some(true),
            sort_order: Some(1),
        };

        let cmd = req.into_slash_command();
        assert!(cmd.is_favorite);
        assert_eq!(cmd.sort_order, Some(1));
    }
}
