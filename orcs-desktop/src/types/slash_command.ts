/**
 * Type of slash command execution
 */
export type CommandType = 'prompt' | 'shell' | 'task';

/**
 * Custom slash command definition
 */
export interface SlashCommand {
  /** Command name (used as /name in chat) */
  name: string;
  /** Icon to display in UI */
  icon: string;
  /** Human-readable description */
  description: string;
  /** Type of command (prompt, shell, or task) */
  type: CommandType;
  /** Command content (prompt template, shell command, or task description) */
  content: string;
  /** Working directory for shell commands (supports variables like {workspace_path}) */
  workingDir?: string;
  /** Optional description of expected arguments */
  argsDescription?: string;
  /** Task execution strategy blueprint (JSON serialized) for task type commands */
  taskBlueprint?: string;
}

export interface ExpandedSlashCommand {
  content: string;
  workingDir?: string;
}

/**
 * Extended command definition including custom commands
 */
export interface ExtendedCommandDefinition {
  name: string;
  icon: string;
  description: string;
  usage: string;
  examples?: string[];
  isCustom: boolean;
  customCommand?: SlashCommand;
}
