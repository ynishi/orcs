import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Workspace } from '../types/workspace';

interface WorkspaceStore {
  // State
  workspaces: Map<string, Workspace>; // id -> Workspace
  isLoaded: boolean;

  // Actions
  initialize: () => Promise<void>;

  // Helpers
  getWorkspace: (id: string) => Workspace | undefined;
  getAllWorkspaces: () => Workspace[];
  getCurrentWorkspace: (workspaceId: string | null) => Workspace | undefined;
}

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
  workspaces: new Map(),
  isLoaded: false,

  initialize: async () => {
    console.log('[WorkspaceStore] Initializing...');

    try {
      // Get initial snapshot
      const workspaceList = await invoke<Workspace[]>('get_workspaces_snapshot');
      const workspaceMap = new Map(workspaceList.map((ws) => [ws.id, ws]));

      console.log('[WorkspaceStore] Initial snapshot:', workspaceList.length, 'workspaces');
      set({ workspaces: workspaceMap, isLoaded: true });

      // Listen to snapshot events (for future use)
      await listen<Workspace[]>('workspace:snapshot', (event) => {
        console.log('[WorkspaceStore] Received snapshot:', event.payload.length);
        const map = new Map(event.payload.map((ws) => [ws.id, ws]));
        set({ workspaces: map, isLoaded: true });
      });

      // Listen to update events
      await listen<Workspace>('workspace:update', (event) => {
        console.log('[WorkspaceStore] Received update:', event.payload.id);
        console.log('[WorkspaceStore] 📁 Uploaded files count:', event.payload.resources?.uploadedFiles?.length || 0);
        console.log('[WorkspaceStore] 📁 Uploaded files:', event.payload.resources?.uploadedFiles);
        set((state) => {
          const newMap = new Map(state.workspaces);
          newMap.set(event.payload.id, event.payload);
          return { workspaces: newMap };
        });
      });

      // Listen to delete events
      await listen<{ id: string }>('workspace:delete', (event) => {
        console.log('[WorkspaceStore] Received delete:', event.payload.id);
        set((state) => {
          const newMap = new Map(state.workspaces);
          newMap.delete(event.payload.id);
          return { workspaces: newMap };
        });
      });

      console.log('[WorkspaceStore] Initialized successfully');
    } catch (error) {
      console.error('[WorkspaceStore] Failed to initialize:', error);
      throw error;
    }
  },

  getWorkspace: (id: string) => {
    return get().workspaces.get(id);
  },

  getAllWorkspaces: () => {
    return Array.from(get().workspaces.values()).sort((a, b) => {
      // Sort by: favorite first, then by last_accessed desc
      if (a.isFavorite && !b.isFavorite) return -1;
      if (!a.isFavorite && b.isFavorite) return 1;
      return b.lastAccessed - a.lastAccessed;
    });
  },

  getCurrentWorkspace: (workspaceId: string | null) => {
    if (!workspaceId) return undefined;
    return get().workspaces.get(workspaceId);
  },
}));
