/**
 * Session Settings Store
 * Backend-First SSOT pattern for session-scoped settings
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { MessageType } from '../types/message';
import { changeTalkStyle } from '../services/talkStyleService';
import { changeConversationMode } from '../services/conversationModeService';
import { changeExecutionStrategy } from '../services/executionStrategyService';
import type { ContextMode } from '../types/session';

export type MessageCallback = (type: MessageType, author: string, text: string) => void;

export interface SessionSettingsStore {
  // State
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  contextMode: ContextMode;
  activeParticipantIds: string[];
  isLoaded: boolean;

  // Actions
  loadSettings: (sessionId: string | null) => Promise<void>;
  updateTalkStyle: (style: string | null, addMessage: MessageCallback) => Promise<void>;
  updateConversationMode: (mode: string, addMessage: MessageCallback) => Promise<void>;
  updateExecutionStrategy: (strategy: string, addMessage: MessageCallback) => Promise<void>;
  updateContextMode: (mode: ContextMode, addMessage: MessageCallback) => Promise<void>;
  toggleParticipant: (personaId: string, isActive: boolean, personaName: string, addMessage: MessageCallback) => Promise<void>;
  refreshActiveParticipants: () => Promise<void>;
}

export const useSessionSettingsStore = create<SessionSettingsStore>((set, get) => ({
  // Initial state
  conversationMode: 'normal',
  talkStyle: null,
  executionStrategy: 'sequential',
  contextMode: 'rich',
  activeParticipantIds: [],
  isLoaded: false,

  // Actions
  loadSettings: async (sessionId: string | null) => {
    if (!sessionId) {
      console.log('[SessionSettingsStore] No session ID, skipping load');
      return;
    }

    console.log('[SessionSettingsStore] Loading settings for session:', sessionId);

    // Load settings individually with fallback to defaults
    let mode = 'normal';
    let style: string | null = null;
    let ctxMode: ContextMode = 'rich';
    let activeIds: string[] = [];

    try {
      mode = await invoke<string>('get_conversation_mode');
    } catch (error) {
      console.warn('[SessionSettingsStore] Failed to load conversation mode, using default:', error);
    }

    try {
      style = await invoke<string | null>('get_talk_style');
    } catch (error) {
      console.warn('[SessionSettingsStore] Failed to load talk style, using default:', error);
    }

    try {
      const ctxModeStr = await invoke<string>('get_context_mode');
      ctxMode = ctxModeStr === 'clean' ? 'clean' : 'rich';
    } catch (error) {
      console.warn('[SessionSettingsStore] Failed to load context mode, using default:', error);
    }

    try {
      activeIds = await invoke<string[]>('get_active_participants');
    } catch (error) {
      console.warn('[SessionSettingsStore] Failed to load active participants, using empty list:', error);
    }

    set({
      conversationMode: mode,
      talkStyle: style,
      contextMode: ctxMode,
      activeParticipantIds: activeIds,
      isLoaded: true,
    });

    console.log('[SessionSettingsStore] Settings loaded (with fallbacks if needed)');
  },

  updateTalkStyle: async (style: string | null, addMessage: MessageCallback) => {
    console.log('[SessionSettingsStore] updateTalkStyle called:', style);

    // Optimistic update
    const currentStyle = get().talkStyle;
    set({ talkStyle: style });

    try {
      // Delegate to service layer
      await changeTalkStyle(style, { invoke, addMessage });
      console.log('[SessionSettingsStore] TalkStyle updated successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to update TalkStyle:', error);
      // Revert optimistic update
      set({ talkStyle: currentStyle });
      throw error;
    }
  },

  updateConversationMode: async (mode: string, addMessage: MessageCallback) => {
    console.log('[SessionSettingsStore] updateConversationMode called:', mode);

    // Optimistic update
    const currentMode = get().conversationMode;
    set({ conversationMode: mode });

    try {
      // Delegate to service layer
      await changeConversationMode(mode, { invoke, addMessage });
      console.log('[SessionSettingsStore] ConversationMode updated successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to update ConversationMode:', error);
      // Revert optimistic update
      set({ conversationMode: currentMode });
      throw error;
    }
  },

  updateExecutionStrategy: async (strategy: string, addMessage: MessageCallback) => {
    console.log('[SessionSettingsStore] updateExecutionStrategy called:', strategy);

    // Optimistic update
    const currentStrategy = get().executionStrategy;
    set({ executionStrategy: strategy });

    try {
      // Delegate to service layer
      await changeExecutionStrategy(strategy, { invoke, addMessage });
      console.log('[SessionSettingsStore] ExecutionStrategy updated successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to update ExecutionStrategy:', error);
      // Revert optimistic update
      set({ executionStrategy: currentStrategy });
      throw error;
    }
  },

  updateContextMode: async (mode: ContextMode, addMessage: MessageCallback) => {
    console.log('[SessionSettingsStore] updateContextMode called:', mode);

    // Optimistic update
    const currentMode = get().contextMode;
    set({ contextMode: mode });

    try {
      await invoke('set_context_mode', { mode });
      const label = mode === 'rich' ? 'Rich Context' : 'Clean Context';
      addMessage('system', 'System', `Context mode changed to ${label}`);
      console.log('[SessionSettingsStore] ContextMode updated successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to update ContextMode:', error);
      // Revert optimistic update
      set({ contextMode: currentMode });
      throw error;
    }
  },

  toggleParticipant: async (personaId: string, isActive: boolean, personaName: string, addMessage: MessageCallback) => {
    console.log('[SessionSettingsStore] toggleParticipant called:', personaId, isActive);

    try {
      if (isActive) {
        await invoke('add_participant', { personaId });
        addMessage('system', 'System', `${personaName} が会話に参加しました`);
      } else {
        await invoke('remove_participant', { personaId });
        addMessage('system', 'System', `${personaName} が会話から退出しました`);
      }

      // Refresh to update active participant list
      await get().refreshActiveParticipants();
      console.log('[SessionSettingsStore] Participant toggled successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to toggle participant:', error);
      addMessage('error', 'System', `Failed to update participant: ${error}`);
      throw error;
    }
  },

  refreshActiveParticipants: async () => {
    try {
      const activeIds = await invoke<string[]>('get_active_participants');

      set({
        activeParticipantIds: activeIds,
      });

      console.log('[SessionSettingsStore] Active participants refreshed');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to refresh active participants:', error);
      throw error;
    }
  },
}));
