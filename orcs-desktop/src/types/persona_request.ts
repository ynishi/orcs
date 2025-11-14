/**
 * Unified request model for creating a persona.
 * Matches Rust's CreatePersonaRequest exactly.
 * ID is NOT accepted - always auto-generated as UUID on backend.
 */
export interface CreatePersonaRequest {
  /** Display name (required) */
  name: string;

  /** Role or title (required) */
  role: string;

  /** Background description (required, min 10 chars recommended) */
  background: string;

  /** Communication style (required, min 10 chars recommended) */
  communication_style: string;

  /** Whether to include in new sessions by default */
  default_participant?: boolean;

  /** LLM backend to use */
  backend: 'claude_cli' | 'claude_api' | 'gemini_cli' | 'gemini_api' | 'open_ai_api' | 'codex_cli';

  /** Optional specific model name */
  model_name?: string;

  /** Optional visual icon/emoji */
  icon?: string;

  /** Optional base color for UI theming */
  base_color?: string;
}
