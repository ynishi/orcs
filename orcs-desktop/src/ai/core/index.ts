/**
 * AI Core Exports
 *
 * AI機能のコアロジック（ヘッドレス部分）をエクスポート
 */

// Types
export type {
  AIContextInfo,
  AIOperation,
  AIHistoryEntry,
  IAIProvider,
  AIRegisterOptions,
  AIRegisterResult,
} from './types/ai';

// Context
export { AIProvider, useAIContext } from './context/AIContext';
export type { AIContextValue, AIProviderProps } from './context/AIContext';

// Hooks
export { useAIRegister } from './hooks/useAIRegister';
