/**
 * Type of slash command execution
 */
export type CommandType = 'prompt' | 'shell' | 'task' | 'action' | 'pipeline';

/**
 * A single step in a pipeline
 */
export interface PipelineStep {
  /** Name of the command to execute */
  commandName: string;
  /** Optional arguments for this step */
  args?: string;
}

/**
 * Configuration for Pipeline type commands
 */
export interface PipelineConfig {
  /** Ordered list of steps to execute */
  steps: PipelineStep[];
  /** Stop execution on first error (default: true) */
  failOnError: boolean;
  /** Pass previous step output as input to next step (default: true) */
  chainOutput: boolean;
}

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
  /** Configuration for Pipeline type commands */
  pipelineConfig?: PipelineConfig;
  /**
   * Whether to include this command in system prompts for personas.
   * Default: true for Prompt/Shell/Action, false for Task
   */
  includeInSystemPrompt?: boolean;
  /** Whether this command is marked as favorite */
  isFavorite?: boolean;
  /** Sort order within favorites (lower = higher priority) */
  sortOrder?: number;
}

export interface ExpandedSlashCommand {
  content: string;
  workingDir?: string;
}

/**
 * Persona info returned with action command result
 */
export interface ActionPersonaInfo {
  name: string;
  icon?: string;
  backend: string;
}

/**
 * Result of executing an action command
 */
export interface ActionCommandResult {
  result: string;
  personaInfo?: ActionPersonaInfo;
}

/**
 * Result of a single pipeline step execution
 */
export interface PipelineStepResult {
  /** Step index (0-based) */
  stepIndex: number;
  /** Command name that was executed */
  commandName: string;
  /** Whether the step succeeded */
  success: boolean;
  /** Output content from the step */
  output?: string;
  /** Error message if failed */
  error?: string;
}

/**
 * Result of executing a pipeline command
 */
export interface PipelineResult {
  /** Overall success status */
  success: boolean;
  /** Results from each step */
  steps: PipelineStepResult[];
  /** Final combined output */
  finalOutput?: string;
  /** Error message if pipeline failed */
  error?: string;
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
