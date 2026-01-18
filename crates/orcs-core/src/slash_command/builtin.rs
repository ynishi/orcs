//! Builtin slash commands provided by the system.
//!
//! These commands are always available and cannot be modified by users.
//! They are loaded once at startup and cached for the lifetime of the application.

use serde::Serialize;
use std::sync::OnceLock;

/// A builtin slash command provided by the system.
#[derive(Debug, Clone, Serialize)]
pub struct BuiltinSlashCommand {
    /// Command name (without the leading /)
    pub name: &'static str,
    /// Usage format (e.g., "/help [command]")
    pub usage: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// Optional description of expected arguments
    pub args: Option<&'static str>,
}

impl BuiltinSlashCommand {
    /// Creates a new builtin slash command.
    pub const fn new(
        name: &'static str,
        usage: &'static str,
        description: &'static str,
        args: Option<&'static str>,
    ) -> Self {
        Self {
            name,
            usage,
            description,
            args,
        }
    }
}

/// Static storage for builtin commands (initialized once).
static BUILTIN_COMMANDS: OnceLock<Vec<BuiltinSlashCommand>> = OnceLock::new();

/// Returns a reference to all builtin slash commands.
///
/// The commands are initialized on first access and cached for subsequent calls.
pub fn builtin_commands() -> &'static [BuiltinSlashCommand] {
    BUILTIN_COMMANDS.get_or_init(|| {
        vec![
            BuiltinSlashCommand::new(
                "help",
                "/help [command]",
                "Show available commands and their usage",
                Some("Optional command name to show detailed help"),
            ),
            BuiltinSlashCommand::new(
                "status",
                "/status",
                "Display current system status and active tasks",
                None,
            ),
            BuiltinSlashCommand::new(
                "task",
                "/task <description>",
                "Create an orchestrated task from the provided description",
                Some("Describe the work you want executed"),
            ),
            BuiltinSlashCommand::new(
                "expert",
                "/expert <expertise>",
                "Create an adhoc expert persona for immediate collaboration",
                Some("Expertise area or domain knowledge"),
            ),
            BuiltinSlashCommand::new(
                "blueprint",
                "/blueprint <task description>",
                "Convert a task or topic into the BlueprintWorkflow format",
                Some("Task or discussion context to convert"),
            ),
            BuiltinSlashCommand::new(
                "workspace",
                "/workspace [name]",
                "Switch to a different workspace or list all available workspaces",
                Some("Workspace name (optional)"),
            ),
            BuiltinSlashCommand::new(
                "files",
                "/files",
                "List files saved to workspace storage (not project source files)",
                None,
            ),
            BuiltinSlashCommand::new(
                "search",
                "/search <query> [-p|-a|-f]",
                "Search workspace sessions and files for the provided query",
                Some("(default) current workspace, -p +project files, -a all workspaces, -f full (all + project)"),
            ),
            BuiltinSlashCommand::new(
                "mode",
                "/mode [normal|concise|brief|discussion]",
                "Change conversation mode to control agent verbosity",
                Some("normal / concise / brief / discussion"),
            ),
            BuiltinSlashCommand::new(
                "talk",
                "/talk [brainstorm|casual|decision_making|debate|problem_solving|review|planning|none]",
                "Set dialogue style for multi-agent collaboration",
                Some("brainstorm / casual / decision_making / debate / problem_solving / review / planning / none"),
            ),
            BuiltinSlashCommand::new(
                "create-persona",
                "/create-persona <json>",
                "Create a new persona from JSON definition (UUID auto-generated)",
                Some(r#"JSON with required fields: name, role, background (min 10 chars), communication_style (min 10 chars), backend (claude_cli/claude_api/gemini_cli/gemini_api/open_ai_api/codex_cli). Optional: model_name, default_participant (bool), icon, base_color. NOTE: ID is always auto-generated as UUID (not accepted in request)"#),
            ),
            BuiltinSlashCommand::new(
                "create-slash-command",
                "/create-slash-command <json>",
                "Create a new slash command (not yet implemented)",
                Some("JSON slash command definition"),
            ),
            BuiltinSlashCommand::new(
                "create-workspace",
                "/create-workspace <json>",
                "Create a new workspace (not yet implemented)",
                Some("JSON workspace definition"),
            ),
        ]
    })
}

/// Find a builtin command by name.
pub fn find_builtin_command(name: &str) -> Option<&'static BuiltinSlashCommand> {
    builtin_commands().iter().find(|cmd| cmd.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_commands_initialized() {
        let commands = builtin_commands();
        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "help"));
        assert!(commands.iter().any(|c| c.name == "search"));
    }

    #[test]
    fn test_find_builtin_command() {
        assert!(find_builtin_command("help").is_some());
        assert!(find_builtin_command("nonexistent").is_none());
    }
}
