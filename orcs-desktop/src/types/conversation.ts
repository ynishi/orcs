/**
 * Conversation Mode definitions
 */
export type ConversationModeType = 'normal' | 'concise' | 'brief' | 'discussion';

export interface ConversationModeOption {
  value: ConversationModeType;
  label: string;
  icon: string;
  description: string;
}

export const CONVERSATION_MODES: ConversationModeOption[] = [
  {
    value: 'normal',
    label: '通常',
    icon: '🗨️',
    description: 'Normal conversation mode',
  },
  {
    value: 'concise',
    label: '簡潔',
    icon: '📝',
    description: 'Concise mode (300 chars)',
  },
  {
    value: 'brief',
    label: '極簡潔',
    icon: '✏️',
    description: 'Brief mode (150 chars)',
  },
  {
    value: 'discussion',
    label: '議論',
    icon: '💭',
    description: 'Discussion mode',
  },
];

/**
 * Talk Style definitions
 */
export type TalkStyleType = 'brainstorm' | 'casual' | 'decision_making' | 'debate' | 'problem_solving' | 'review' | 'planning';

export interface TalkStyleOption {
  value: TalkStyleType;
  label: string;
  icon: string;
  description: string;
}

export const DEFAULT_STYLE_ICON: string = '💬';
export const DEFAULT_STYLE_LABEL: string = '通常';

export const TALK_STYLES: TalkStyleOption[] = [
  {
    value: 'brainstorm',
    label: 'ブレインストーミング',
    icon: '💡',
    description: 'Brainstorming session',
  },
  {
    value: 'casual',
    label: 'カジュアル',
    icon: '☕',
    description: 'Casual conversation',
  },
  {
    value: 'decision_making',
    label: '意思決定',
    icon: '🎯',
    description: 'Decision making',
  },
  {
    value: 'debate',
    label: '議論',
    icon: '⚖️',
    description: 'Debate style',
  },
  {
    value: 'problem_solving',
    label: '問題解決',
    icon: '🔧',
    description: 'Problem solving',
  },
  {
    value: 'review',
    label: 'レビュー',
    icon: '🔍',
    description: 'Review session',
  },
  {
    value: 'planning',
    label: '計画',
    icon: '📋',
    description: 'Planning session',
  },
];

export function getConversationModeOption(mode: ConversationModeType): ConversationModeOption | undefined {
  return CONVERSATION_MODES.find(m => m.value === mode);
}

export function getTalkStyleOption(style: TalkStyleType): TalkStyleOption | undefined {
  return TALK_STYLES.find(s => s.value === style);
}

/**
 * Execution Strategy definitions
 */
export type ExecutionStrategyType = 'broadcast' | 'sequential' | 'mentioned';

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
export type PresetSource = 'system' | 'user';

export interface DialoguePreset {
  id: string;
  name: string;
  icon?: string;
  description?: string;
  execution_strategy: ExecutionStrategyType;
  conversation_mode: ConversationModeType;
  talk_style?: TalkStyleType;
  created_at: string;
  source: PresetSource;
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
    preset.execution_strategy === executionStrategy &&
    preset.conversation_mode === conversationMode &&
    (preset.talk_style || null) === talkStyle
  );
}
