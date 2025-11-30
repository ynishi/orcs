import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface DebugSettings {
  enableLlmDebug: boolean;
  logLevel: string;
}

export interface DebugStore {
  debugSettings: DebugSettings | null;
  isLoaded: boolean;

  // Actions
  initialize: () => Promise<void>;
  updateDebugSettings: (enableLlmDebug: boolean, logLevel: string) => Promise<void>;
  toggleDebugMode: () => Promise<void>;
}

export const useDebugStore = create<DebugStore>((set, get) => ({
  debugSettings: null,
  isLoaded: false,

  initialize: async () => {
    console.log('[DebugStore] Initializing...');

    try {
      const settings = await invoke<DebugSettings>('get_debug_settings');
      console.log('[DebugStore] Loaded settings:', settings);
      set({ debugSettings: settings, isLoaded: true });
    } catch (error) {
      console.error('[DebugStore] Failed to load debug settings:', error);
      // Set default values on error
      set({
        debugSettings: { enableLlmDebug: false, logLevel: 'info' },
        isLoaded: true,
      });
    }
  },

  updateDebugSettings: async (enableLlmDebug: boolean, logLevel: string) => {
    console.log('[DebugStore] Updating debug settings:', { enableLlmDebug, logLevel });

    try {
      await invoke('update_debug_settings', {
        enableLlmDebug,
        logLevel,
      });

      // Update local state
      set({
        debugSettings: { enableLlmDebug, logLevel },
      });

      console.log('[DebugStore] Debug settings updated successfully');
    } catch (error) {
      console.error('[DebugStore] Failed to update debug settings:', error);
      throw error;
    }
  },

  toggleDebugMode: async () => {
    const current = get().debugSettings;
    if (!current) {
      console.warn('[DebugStore] Cannot toggle - settings not loaded');
      return;
    }

    const newValue = !current.enableLlmDebug;
    const newLogLevel = newValue ? 'trace' : 'info';

    await get().updateDebugSettings(newValue, newLogLevel);
  },
}));
