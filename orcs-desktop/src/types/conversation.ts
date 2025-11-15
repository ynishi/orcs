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
    icon: '💬',
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
