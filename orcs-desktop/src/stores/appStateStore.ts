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

  // Tab management
  openTab: (sessionId: string, workspaceId: string) => Promise<string>;
  closeTab: (tabId: string) => Promise<void>;
  setActiveTab: (tabId: string) => Promise<void>;
  reorderTabs: (fromIndex: number, toIndex: number) => Promise<void>;
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
      set({ appState: { ...current, last_selected_workspace_id: workspaceId } });
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
      set({ appState: { ...current, last_selected_workspace_id: null } });
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
      set({ appState: { ...current, active_session_id: sessionId } });
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
      set({ appState: { ...current, active_session_id: null } });
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

  openTab: async (sessionId: string, workspaceId: string) => {
    console.log('[AppStateStore] Opening tab:', { sessionId, workspaceId });

    try {
      // Backend call returns tab_id and emits app-state:update event
      const tabId = await invoke<string>('open_tab', { sessionId, workspaceId });
      console.log('[AppStateStore] Tab opened:', tabId);
      return tabId;
    } catch (error) {
      console.error('[AppStateStore] Failed to open tab:', error);
      throw error;
    }
  },

  closeTab: async (tabId: string) => {
    console.log('[AppStateStore] Closing tab:', tabId);

    try {
      // Backend call emits app-state:update event
      await invoke('close_tab', { tabId });
      console.log('[AppStateStore] Tab closed:', tabId);
    } catch (error) {
      console.error('[AppStateStore] Failed to close tab:', error);
      throw error;
    }
  },

  setActiveTab: async (tabId: string) => {
    console.log('[AppStateStore] Setting active tab:', tabId);

    // Optimistic update for better UX (this is a frequent operation)
    const current = get().appState;
    if (current) {
      set({
        appState: {
          ...current,
          active_tab_id: tabId,
          open_tabs: current.open_tabs.map((tab) =>
            tab.id === tabId
              ? { ...tab, last_accessed_at: Math.floor(Date.now() / 1000) }
              : tab
          ),
        },
      });
    }

    try {
      // Backend call (memory-only update, no disk write)
      await invoke('set_active_tab', { tabId });
    } catch (error) {
      console.error('[AppStateStore] Failed to set active tab:', error);
      // Revert optimistic update
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },

  reorderTabs: async (fromIndex: number, toIndex: number) => {
    console.log('[AppStateStore] Reordering tabs:', { fromIndex, toIndex });

    // Optimistic update for better UX
    const current = get().appState;
    if (current) {
      const newTabs = [...current.open_tabs];
      const [movedTab] = newTabs.splice(fromIndex, 1);
      newTabs.splice(toIndex, 0, movedTab);

      // Update order field
      const reorderedTabs = newTabs.map((tab, index) => ({
        ...tab,
        order: index,
      }));

      set({
        appState: {
          ...current,
          open_tabs: reorderedTabs,
        },
      });
    }

    try {
      // Backend call (memory-only update, no disk write)
      await invoke('reorder_tabs', { fromIndex, toIndex });
    } catch (error) {
      console.error('[AppStateStore] Failed to reorder tabs:', error);
      // Revert optimistic update
      if (current) {
        set({ appState: current });
      }
      throw error;
    }
  },
}));
