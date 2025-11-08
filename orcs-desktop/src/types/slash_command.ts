/**
 * Type of slash command execution
 */
export type CommandType = 'prompt' | 'shell';

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
  /** Type of command (prompt or shell) */
  type: CommandType;
  /** Command content (prompt template or shell command) */
  content: string;
  /** Working directory for shell commands (supports variables like {workspace_path}) */
  workingDir?: string;
  /** Optional description of expected arguments */
  argsDescription?: string;
}

export interface ExpandedSlashCommand {
  content: string;
  workingDir?: string;
  has_prompt_template?: boolean;
  immediate_messages?: Array<{
    content: string;
    message_type: string;
    severity: string;
    persist_to_session?: boolean;
  }>;
  prompt_to_send?: string;
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
