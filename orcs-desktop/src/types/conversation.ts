/**
 * Type definitions auto-generated from Rust code
 */
import type {
  ConversationModeType,
  TalkStyleType,
  ExecutionModelType as ExecutionStrategyType,
  PresetSourceType as PresetSource,
} from './generated/schema';
export type { ConversationModeType, TalkStyleType, ExecutionStrategyType, PresetSource };

export interface ConversationModeOption {
  value: ConversationModeType;
  label: string;
  icon: string;
  description: string;
}

export const CONVERSATION_MODES: ConversationModeOption[] = [
  {
    value: 'detailed',
    label: 'Detailed',
    icon: '📖',
    description: 'Detailed mode (comprehensive explanations)',
  },
  {
    value: 'normal',
    label: 'Normal',
    icon: '🗨️',
    description: 'Normal conversation mode',
  },
  {
    value: 'concise',
    label: 'Concise',
    icon: '📝',
    description: 'Concise mode (300 chars)',
  },
  {
    value: 'brief',
    label: 'Brief',
    icon: '✏️',
    description: 'Brief mode (150 chars)',
  },
  {
    value: 'discussion',
    label: 'Discussion',
    icon: '💭',
    description: 'Discussion mode',
  },
];

export interface TalkStyleOption {
  value: TalkStyleType;
  label: string;
  icon: string;
  description: string;
}

export const DEFAULT_STYLE_ICON: string = '💬';
export const DEFAULT_STYLE_LABEL: string = 'Normal';

export const TALK_STYLES: TalkStyleOption[] = [
  {
    value: 'Brainstorm',
    label: 'Brainstorm',
    icon: '💡',
    description: 'Brainstorming session',
  },
  {
    value: 'Casual',
    label: 'Casual',
    icon: '☕',
    description: 'Casual conversation',
  },
  {
    value: 'DecisionMaking',
    label: 'Decision Making',
    icon: '🎯',
    description: 'Decision making',
  },
  {
    value: 'Debate',
    label: 'Debate',
    icon: '⚖️',
    description: 'Debate style',
  },
  {
    value: 'ProblemSolving',
    label: 'Problem Solving',
    icon: '🔧',
    description: 'Problem solving',
  },
  {
    value: 'Review',
    label: 'Review',
    icon: '🔍',
    description: 'Review session',
  },
  {
    value: 'Planning',
    label: 'Planning',
    icon: '📋',
    description: 'Planning session',
  },
  {
    value: 'Research',
    label: 'Research',
    icon: '🔬',
    description: 'Fact-focused deep investigation',
  },
];

export function getConversationModeOption(mode: ConversationModeType): ConversationModeOption | undefined {
  return CONVERSATION_MODES.find(m => m.value === mode);
}

export function getTalkStyleOption(style: TalkStyleType): TalkStyleOption | undefined {
  return TALK_STYLES.find(s => s.value === style);
}

export interface ExecutionStrategyOption {
  value: ExecutionStrategyType;
  label: string;
  icon: string;
  description: string;
}

export const EXECUTION_STRATEGIES: ExecutionStrategyOption[] = [
  {
    value: 'broadcast',
    label: 'Broadcast',
    icon: '📢',
    description: 'Send to all participants',
  },
  {
    value: 'sequential',
    label: 'Sequential',
    icon: '➡️',
    description: 'Send one by one',
  },
  {
    value: 'mentioned',
    label: 'Mentioned',
    icon: '👤',
    description: 'Send to @mentioned only',
  },
];

export function getExecutionStrategyOption(strategy: ExecutionStrategyType): ExecutionStrategyOption | undefined {
  return EXECUTION_STRATEGIES.find(s => s.value === strategy);
}

/**
 * Dialogue Preset definitions
 */
export interface DialoguePreset {
  id: string;
  name: string;
  icon?: string;
  description?: string;
  executionStrategy: ExecutionStrategyType; // was execution_strategy
  conversationMode: ConversationModeType; // was conversation_mode
  talkStyle?: TalkStyleType; // was talk_style
  createdAt: string; // was created_at
  source: PresetSource;
  defaultPersonaIds?: string[]; // Persona IDs to auto-add on apply
}

/**
 * Check if current settings match a preset
 */
export function matchesPreset(
  preset: DialoguePreset,
  executionStrategy: string,
  conversationMode: string,
  talkStyle: string | null
): boolean {
  return (
    preset.executionStrategy === executionStrategy &&
    preset.conversationMode === conversationMode &&
    (preset.talkStyle || null) === talkStyle
  );
}
