/**
 * Type of slash command execution
 */
export type CommandType = 'prompt' | 'shell' | 'task' | 'action';

/**
 * Configuration for Action type commands
 */
export interface ActionConfig {
  /** Backend to use for execution (e.g., "gemini_api", "claude_api", "open_ai_api") */
  backend?: string;
  /** Model name override */
  modelName?: string;
  /** Persona ID to use for execution (Phase 2) */
  personaId?: string;
  /** Gemini thinking level (LOW/MEDIUM/HIGH) */
  geminiThinkingLevel?: string;
  /** Enable Gemini Google Search */
  geminiGoogleSearch?: boolean;
}

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
  /** Type of command (prompt, shell, task, or action) */
  type: CommandType;
  /**
   * Command content:
   * - Prompt: Prompt template with variables ({workspace}, {args}, etc.)
   * - Shell: Shell command to execute
   * - Task: Task description
   * - Action: Prompt template with all variables ({session_all}, {session_recent}, {workspace}, {workspace_path}, {files}, {git_branch}, {git_status}, {args})
   */
  content: string;
  /** Working directory for shell commands (supports variables like {workspace_path}) */
  workingDir?: string;
  /** Optional description of expected arguments */
  argsDescription?: string;
  /** Task execution strategy blueprint (JSON serialized) for task type commands */
  taskBlueprint?: string;
  /** Configuration for Action type commands (backend, model, etc.) */
  actionConfig?: ActionConfig;
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
