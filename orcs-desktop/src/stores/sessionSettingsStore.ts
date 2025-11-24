/**
 * Session Settings Store
 * Backend-First SSOT pattern for session-scoped settings
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { PersonaConfig } from '../types/agent';
import type { MessageType } from '../types/message';
import { changeTalkStyle } from '../services/talkStyleService';
import { changeConversationMode } from '../services/conversationModeService';
import { changeExecutionStrategy } from '../services/executionStrategyService';

export type MessageCallback = (type: MessageType, author: string, text: string) => void;

export interface SessionSettingsStore {
  // State
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  personas: PersonaConfig[];
  activeParticipantIds: string[];
  isLoaded: boolean;

  // Actions
  loadSettings: (sessionId: string | null) => Promise<void>;
  updateTalkStyle: (style: string | null, addMessage: MessageCallback) => Promise<void>;
  updateConversationMode: (mode: string, addMessage: MessageCallback) => Promise<void>;
  updateExecutionStrategy: (strategy: string, addMessage: MessageCallback) => Promise<void>;
  toggleParticipant: (personaId: string, isActive: boolean, addMessage: MessageCallback) => Promise<void>;
  refreshPersonas: () => Promise<void>;

  // Getters
  getPersonaById: (personaId: string) => PersonaConfig | undefined;
  getActivePersonas: () => PersonaConfig[];
}

export const useSessionSettingsStore = create<SessionSettingsStore>((set, get) => ({
  // Initial state
  conversationMode: 'normal',
  talkStyle: null,
  executionStrategy: 'sequential',
  personas: [],
  activeParticipantIds: [],
  isLoaded: false,

  // Actions
  loadSettings: async (sessionId: string | null) => {
    if (!sessionId) {
      console.log('[SessionSettingsStore] No session ID, skipping load');
      return;
    }

    console.log('[SessionSettingsStore] Loading settings for session:', sessionId);

    try {
      // Load all settings from backend
      const [mode, style, personas, activeIds] = await Promise.all([
        invoke<string>('get_conversation_mode'),
        invoke<string | null>('get_talk_style'),
        invoke<PersonaConfig[]>('get_personas'),
        invoke<string[]>('get_active_participants'),
      ]);

      set({
        conversationMode: mode,
        talkStyle: style,
        personas,
        activeParticipantIds: activeIds,
        isLoaded: true,
      });

      console.log('[SessionSettingsStore] Settings loaded successfully');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to load settings:', error);
      throw error;
    }
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

  toggleParticipant: async (personaId: string, isActive: boolean, _addMessage: MessageCallback) => {
    // To be implemented
    console.log('[SessionSettingsStore] toggleParticipant called:', personaId, isActive);
  },

  refreshPersonas: async () => {
    try {
      const personas = await invoke<PersonaConfig[]>('get_personas');
      const activeIds = await invoke<string[]>('get_active_participants');

      set({
        personas,
        activeParticipantIds: activeIds,
      });

      console.log('[SessionSettingsStore] Personas refreshed');
    } catch (error) {
      console.error('[SessionSettingsStore] Failed to refresh personas:', error);
      throw error;
    }
  },

  // Getters
  getPersonaById: (personaId: string) => {
    const state = get();
    return state.personas.find(p => p.id === personaId);
  },

  getActivePersonas: () => {
    const state = get();
    return state.personas.filter(p => state.activeParticipantIds.includes(p.id));
  },
}));
