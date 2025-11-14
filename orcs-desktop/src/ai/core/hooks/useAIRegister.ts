/**
 * useAIRegister Hook
 *
 * UIコンポーネントにAI機能を接続するための核心的なフック
 *
 * このフックは、入力フィールド等のUIコンポーネントに対して、
 * AI生成・修正機能をヘッドレスな形で提供します。
 */

import { useState, useCallback, useRef } from 'react';
import { useAIContext } from '../context/AIContext';
import type { AIRegisterOptions, AIRegisterResult, AIHistoryEntry } from '../types/ai';

/**
 * UIコンポーネントにAI機能を登録する
 *
 * @example
 * ```tsx
 * function MyInput() {
 *   const [value, setValue] = useState('');
 *
 *   const ai = useAIRegister({
 *     context: { scope: 'MyInput', type: 'string' },
 *     getValue: () => value,
 *     setValue: (newValue) => setValue(newValue),
 *   });
 *
 *   return (
 *     <div>
 *       <input value={value} onChange={(e) => setValue(e.target.value)} />
 *       <button {...ai.triggerProps}>✨</button>
 *       {ai.menuProps.isOpen && (
 *         <div>
 *           <button onClick={() => ai.actions.generate()}>💫 Generate</button>
 *           <button onClick={() => ai.actions.refine()}>🖌️ Refine</button>
 *         </div>
 *       )}
 *     </div>
 *   );
 * }
 * ```
 */
export function useAIRegister(options: AIRegisterOptions): AIRegisterResult {
  const { provider, enabled: globalEnabled } = useAIContext();

  // 有効/無効判定
  const enabled = (options.enabled ?? true) && globalEnabled && provider !== null;

  // 状態管理
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [history, setHistory] = useState<AIHistoryEntry[]>([]);
  const [error, setError] = useState<Error | null>(null);

  // 現在の値を履歴に保存するためのref
  const lastValueRef = useRef<string>('');

  /**
   * 履歴に新しいエントリを追加
   */
  const addToHistory = useCallback(
    (value: string, operation: 'generate' | 'refine', direction?: string) => {
      const entry: AIHistoryEntry = {
        value,
        timestamp: new Date(),
        operation,
        direction,
      };

      setHistory((prev) => [...prev, entry]);
      lastValueRef.current = value;
    },
    []
  );

  /**
   * 💫 生成アクション
   */
  const generate = useCallback(
    async (direction?: string) => {
      if (!enabled || !provider) {
        console.warn('[AI] Generate called but AI is not enabled or provider is null');
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        console.log('[AI] Generating content...', {
          scope: options.context.scope,
          direction,
        });

        const result = await provider.generate(options.context, direction);

        console.log('[AI] Generation successful', {
          length: result.length,
        });

        // 値を設定
        options.setValue(result);

        // 履歴に追加
        addToHistory(result, 'generate', direction);

        // メニューを閉じる
        setIsMenuOpen(false);
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        console.error('[AI] Generate failed:', error);
        setError(error);

        // エラーハンドラが指定されていれば呼び出す
        options.onError?.(error);
      } finally {
        setIsLoading(false);
      }
    },
    [enabled, provider, options, addToHistory]
  );

  /**
   * 🖌️ 修正アクション
   */
  const refine = useCallback(
    async (direction?: string) => {
      if (!enabled || !provider) {
        console.warn('[AI] Refine called but AI is not enabled or provider is null');
        return;
      }

      const currentValue = options.getValue();

      // 値が空の場合はgenerateにフォールバック
      if (!currentValue.trim()) {
        console.log('[AI] Refine called with empty value, falling back to generate');
        return generate(direction);
      }

      setIsLoading(true);
      setError(null);

      try {
        console.log('[AI] Refining content...', {
          scope: options.context.scope,
          direction,
          currentLength: currentValue.length,
        });

        const result = await provider.refine(currentValue, options.context, direction);

        console.log('[AI] Refinement successful', {
          length: result.length,
        });

        // 値を設定
        options.setValue(result);

        // 履歴に追加
        addToHistory(result, 'refine', direction);

        // メニューを閉じる
        setIsMenuOpen(false);
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        console.error('[AI] Refine failed:', error);
        setError(error);

        // エラーハンドラが指定されていれば呼び出す
        options.onError?.(error);
      } finally {
        setIsLoading(false);
      }
    },
    [enabled, provider, options, generate, addToHistory]
  );

  /**
   * ← Undoアクション
   */
  const undo = useCallback(() => {
    if (history.length === 0) {
      console.warn('[AI] Undo called but no history available');
      return;
    }

    // 最新の履歴を削除
    const newHistory = history.slice(0, -1);
    setHistory(newHistory);

    // 1つ前の値に戻す
    if (newHistory.length > 0) {
      const previousEntry = newHistory[newHistory.length - 1];
      options.setValue(previousEntry.value);
      lastValueRef.current = previousEntry.value;
      console.log('[AI] Undo to previous value');
    } else {
      // 履歴が空になった場合は空文字に戻す
      options.setValue('');
      lastValueRef.current = '';
      console.log('[AI] Undo to empty (no more history)');
    }
  }, [history, options]);

  /**
   * 🗒️ 履歴表示アクション
   */
  const showHistory = useCallback(() => {
    console.log('[AI] Show history:', history);
    // TODO: 履歴モーダルを表示する実装
    // 現時点ではコンソールログのみ
  }, [history]);

  /**
   * 💬 チャットを開くアクション
   */
  const showChat = useCallback(() => {
    console.log('[AI] Show chat for scope:', options.context.scope);
    // TODO: チャットパネルを表示する実装
    // 現時点ではコンソールログのみ
  }, [options.context]);

  // 戻り値
  return {
    triggerProps: {
      onClick: () => {
        if (enabled) {
          setIsMenuOpen((prev) => !prev);
        } else {
          console.warn('[AI] Trigger clicked but AI is not enabled');
        }
      },
      isActive: isMenuOpen,
    },

    menuProps: {
      isOpen: isMenuOpen,
      onClose: () => setIsMenuOpen(false),
    },

    actions: {
      generate,
      refine,
      undo,
      showHistory,
      showChat,
    },

    state: {
      isLoading,
      history,
      canUndo: history.length > 0,
      error,
    },
  };
}
