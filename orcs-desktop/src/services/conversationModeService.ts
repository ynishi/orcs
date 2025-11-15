/**
 * Conversation Mode Service
 * Application layer logic for conversation mode changes
 */

import { InvokeArgs } from '@tauri-apps/api/core';
import { MessageType } from '../types/message';
import { CONVERSATION_MODES } from '../types/conversation';
import { handleAndPersistSystemMessage, conversationMessage } from '../utils/systemMessage';

export interface ConversationModeServiceDependencies {
  invoke: <T>(cmd: string, args?: InvokeArgs) => Promise<T>;
  addMessage: (type: MessageType, author: string, text: string) => void;
}

/**
 * Change conversation mode with backend persistence and system message notification
 */
export async function changeConversationMode(
  mode: string,
  deps: ConversationModeServiceDependencies
): Promise<void> {
  const { invoke, addMessage } = deps;

  try {
    // Update backend
    await invoke('set_conversation_mode', { mode });

    // Show system message
    const modeLabel = CONVERSATION_MODES.find(m => m.value === mode)?.label || mode;
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });

    await handleAndPersistSystemMessage(
      conversationMessage(
        `Conversation mode changed to: ${modeLabel} [${timestamp}]`,
        'info'
      ),
      addMessage,
      invoke
    );
  } catch (error) {
    console.error('Failed to set conversation mode:', error);
    await handleAndPersistSystemMessage(
      conversationMessage(
        `Failed to set conversation mode: ${error}`,
        'error'
      ),
      addMessage,
      invoke
    );
    throw error;
  }
}
