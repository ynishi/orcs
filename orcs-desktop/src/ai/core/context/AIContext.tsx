/**
 * AI Context
 *
 * アプリケーション全体でAI機能を共有するためのContext
 */

import { createContext, useContext, useState, useMemo, type ReactNode } from 'react';
import type { IAIProvider } from '../types/ai';

/**
 * AIContext の値
 */
export interface AIContextValue {
  /** 現在のAIプロバイダー */
  provider: IAIProvider | null;

  /** プロバイダーを設定 */
  setProvider: (provider: IAIProvider) => void;

  /** 方向性のプリセット */
  directions: string[];

  /** カスタム方向性を追加 */
  addDirection: (direction: string) => void;

  /** AI機能の有効/無効 */
  enabled: boolean;

  /** AI機能の有効/無効を切り替え */
  setEnabled: (enabled: boolean) => void;
}

const AIContext = createContext<AIContextValue | undefined>(undefined);

/**
 * AIProvider Props
 */
export interface AIProviderProps {
  children: ReactNode;

  /** 初期プロバイダー（オプション） */
  provider?: IAIProvider;

  /** 初期方向性リスト（オプション） */
  initialDirections?: string[];

  /** 初期有効状態（デフォルト: true） */
  initialEnabled?: boolean;
}

/**
 * AI Context Provider
 *
 * アプリケーションのルートでこのProviderをラップしてください。
 *
 * @example
 * ```tsx
 * import { AIProvider } from '@/ai/core/context/AIContext';
 * import { GeminiProvider } from '@/ai/providers/GeminiProvider';
 *
 * function App() {
 *   return (
 *     <AIProvider provider={new GeminiProvider()}>
 *       <YourApp />
 *     </AIProvider>
 *   );
 * }
 * ```
 */
export function AIProvider({
  children,
  provider: initialProvider,
  initialDirections = [
    'フォーマルに',
    '簡潔に',
    '専門的に',
    '友好的に',
    'カジュアルに',
    '詳しく',
  ],
  initialEnabled = true,
}: AIProviderProps) {
  const [provider, setProvider] = useState<IAIProvider | null>(initialProvider || null);
  const [directions, setDirections] = useState<string[]>(initialDirections);
  const [enabled, setEnabled] = useState<boolean>(initialEnabled);

  const addDirection = (direction: string) => {
    if (!directions.includes(direction)) {
      setDirections((prev) => [...prev, direction]);
    }
  };

  const value = useMemo<AIContextValue>(
    () => ({
      provider,
      setProvider,
      directions,
      addDirection,
      enabled,
      setEnabled,
    }),
    [provider, directions, enabled]
  );

  return <AIContext.Provider value={value}>{children}</AIContext.Provider>;
}

/**
 * AI Context を使用するフック
 *
 * @throws AIProviderでラップされていない場合はエラー
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { provider, directions } = useAIContext();
 *
 *   if (!provider) {
 *     return <div>AI provider not configured</div>;
 *   }
 *
 *   return <div>AI enabled with {directions.length} directions</div>;
 * }
 * ```
 */
export function useAIContext(): AIContextValue {
  const context = useContext(AIContext);

  if (!context) {
    throw new Error('useAIContext must be used within AIProvider');
  }

  return context;
}
