import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppState } from '../types/generated/schema';

export interface AppStateStore {
  appState: AppState | null;
  isLoaded: boolean;

  // Actions
  initialize: () => Promise<void>;
  setLastSelectedWorkspace: (workspaceId: string) => Promise<void>;
  clearLastSelectedWorkspace: () => Promise<void>;
  setActiveSession: (sessionId: string) => Promise<void>;
  clearActiveSession: () => Promise<void>;
}

export const useAppStateStore = create<AppStateStore>((set, get) => ({
  appState: null,
  isLoaded: false,

  initialize: async () => {
    console.log('[AppStateStore] Initializing...');

    try {
      // Get initial snapshot
      const snapshot = await invoke<AppState>('get_app_state_snapshot');
      console.log('[AppStateStore] Initial snapshot:', snapshot);
      set({ appState: snapshot, isLoaded: true });

      // Listen to snapshot events (for app startup)
      await listen<AppState>('app-state:snapshot', (event) => {
        console.log('[AppStateStore] Received snapshot:', event.payload);
        set({ appState: event.payload, isLoaded: true });
      });

      // Listen to update events (for runtime changes)
      await listen<AppState>('app-state:update', (event) => {
        console.log('[AppStateStore] Received update:', event.payload);
        set({ appState: event.payload });
      });

      console.log('[AppStateStore] Initialized successfully');
    } catch (error) {
      console.error('[AppStateStore] Failed to initialize:', error);
      throw error;
    }
  },

  setLastSelectedWorkspace: async (workspaceId: string) => {
    console.log('[AppStateStore] Setting last selected workspace:', workspaceId);

    // Optimistic update
    const current = get().appState;
    if (current) {
      set({ appState: { ...current, lastSelectedWorkspaceId: workspaceId } });
    }

    try {
      // Backend call (event will update with final state)
      await invoke('set_last_selected_workspace', { workspaceId });
    } catch (error) {
      console.error('[AppStateStore] Failed to set last selected workspace:', error);
      // Revert optimistic update
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },

  clearLastSelectedWorkspace: async () => {
    console.log('[AppStateStore] Clearing last selected workspace');

    const current = get().appState;
    if (current) {
      set({ appState: { ...current, lastSelectedWorkspaceId: null } });
    }

    try {
      await invoke('clear_last_selected_workspace');
    } catch (error) {
      console.error('[AppStateStore] Failed to clear last selected workspace:', error);
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },

  setActiveSession: async (sessionId: string) => {
    console.log('[AppStateStore] Setting active session:', sessionId);

    const current = get().appState;
    if (current) {
      set({ appState: { ...current, activeSessionId: sessionId } });
    }

    try {
      await invoke('set_active_session_in_app_state', { sessionId });
    } catch (error) {
      console.error('[AppStateStore] Failed to set active session:', error);
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },

  clearActiveSession: async () => {
    console.log('[AppStateStore] Clearing active session');

    const current = get().appState;
    if (current) {
      set({ appState: { ...current, activeSessionId: null } });
    }

    try {
      await invoke('clear_active_session_in_app_state');
    } catch (error) {
      console.error('[AppStateStore] Failed to clear active session:', error);
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },
}));
