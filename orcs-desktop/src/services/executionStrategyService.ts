/**
 * Execution Strategy Service
 * Application layer logic for execution strategy changes
 */

import { InvokeArgs } from '@tauri-apps/api/core';
import { MessageType } from '../types/message';
import { EXECUTION_STRATEGIES } from '../types/conversation';
import { handleAndPersistSystemMessage, conversationMessage } from '../utils/systemMessage';

export interface ExecutionStrategyServiceDependencies {
  invoke: <T>(cmd: string, args?: InvokeArgs) => Promise<T>;
  addMessage: (type: MessageType, author: string, text: string) => void;
}

/**
 * Change execution strategy with backend persistence and system message notification
 */
export async function changeExecutionStrategy(
  strategy: string,
  deps: ExecutionStrategyServiceDependencies
): Promise<void> {
  const { invoke, addMessage } = deps;

  try {
    // Update backend
    await invoke('set_execution_strategy', { strategy });

    // Show system message
    const strategyLabel = EXECUTION_STRATEGIES.find(s => s.value === strategy)?.label || strategy;
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });

    await handleAndPersistSystemMessage(
      conversationMessage(
        `Execution strategy changed to: ${strategyLabel} [${timestamp}]`,
        'info'
      ),
      addMessage,
      invoke
    );
  } catch (error) {
    console.error('Failed to set execution strategy:', error);
    await handleAndPersistSystemMessage(
      conversationMessage(
        `Failed to set execution strategy: ${error}`,
        'error'
      ),
      addMessage,
      invoke
    );
    throw error;
  }
}
