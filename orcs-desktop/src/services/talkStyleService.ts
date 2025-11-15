/**
 * Talk Style Service
 * Application layer logic for talk style changes
 */

import { InvokeArgs } from '@tauri-apps/api/core';
import { MessageType } from '../types/message';
import { TALK_STYLES, DEFAULT_STYLE_LABEL } from '../types/conversation';
import { handleAndPersistSystemMessage, conversationMessage } from '../utils/systemMessage';

export interface TalkStyleServiceDependencies {
  invoke: <T>(cmd: string, args?: InvokeArgs) => Promise<T>;
  addMessage: (type: MessageType, author: string, text: string) => void;
}

/**
 * Change talk style with backend persistence and system message notification
 */
export async function changeTalkStyle(
  style: string | null,
  deps: TalkStyleServiceDependencies
): Promise<void> {
  const { invoke, addMessage } = deps;

  try {
    // Update backend
    await invoke('set_talk_style', { style });

    // Show system message
    const styleLabel = style
      ? (TALK_STYLES.find(s => s.value === style)?.label || style)
      : DEFAULT_STYLE_LABEL;
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });

    await handleAndPersistSystemMessage(
      conversationMessage(
        `Talk style changed to: ${styleLabel} [${timestamp}]`,
        'info'
      ),
      addMessage,
      invoke
    );
  } catch (error) {
    console.error('Failed to set talk style:', error);
    await handleAndPersistSystemMessage(
      conversationMessage(
        `Failed to set talk style: ${error}`,
        'error'
      ),
      addMessage,
      invoke
    );
    throw error; // Re-throw to allow caller to handle if needed
  }
}
