import { createContext, useCallback, useContext, useMemo, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import type { Workspace, UploadedFile } from '../types/workspace';
import { useAppStateStore } from '../stores/appStateStore';
import { useWorkspaceStore } from '../stores/workspaceStore';

export interface WorkspaceContextValue {
  workspace: Workspace | null;
  allWorkspaces: Workspace[];
  files: UploadedFile[];
  switchWorkspace: (sessionId: string, workspaceId: string) => Promise<void>;
  toggleFavorite: (workspaceId: string) => Promise<void>;
  toggleFileArchive: (file: UploadedFile) => Promise<void>;
}

const WorkspaceContext = createContext<WorkspaceContextValue | undefined>(undefined);

interface WorkspaceProviderProps {
  children: ReactNode;
}

export function WorkspaceProvider({ children }: WorkspaceProviderProps) {
  const { appState } = useAppStateStore();
  const workspaces = useWorkspaceStore((state) => state.workspaces);

  // Computed values from stores (Phase 4)
  const workspace = useMemo(() => {
    const workspaceId = appState?.lastSelectedWorkspaceId;
    if (!workspaceId) return null;
    return workspaces.get(workspaceId) || null;
  }, [appState?.lastSelectedWorkspaceId, workspaces]);

  const allWorkspaces = useMemo(() => {
    return Array.from(workspaces.values()).sort((a, b) => {
      // Sort by: favorite first, then by last_accessed desc
      if (a.isFavorite && !b.isFavorite) return -1;
      if (!a.isFavorite && b.isFavorite) return 1;
      return b.lastAccessed - a.lastAccessed;
    });
  }, [workspaces]);

  const files = useMemo(() => {
    return workspace?.resources.uploadedFiles || [];
  }, [workspace]);

  const switchWorkspace = useCallback(async (sessionId: string, workspaceId: string) => {
    try {
      await invoke('switch_workspace', { sessionId, workspaceId });
      // Event-driven update via workspace:update event
    } catch (err) {
      console.error('Failed to switch workspace:', err);
      throw err;
    }
  }, []);

  const toggleFavorite = useCallback(async (workspaceId: string) => {
    try {
      await invoke('toggle_favorite_workspace', { workspaceId });
      // Event-driven update via workspace:update event
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
      throw err;
    }
  }, []);

  const toggleFileArchive = useCallback(
    async (file: UploadedFile) => {
      if (!workspace) return;

      try {
        await invoke('toggle_workspace_file_archive', {
          workspaceId: workspace.id,
          fileId: file.id,
        });

        // Event-driven update via workspace:update event
        notifications.show({
          title: file.isArchived ? 'File Unarchived' : 'File Archived',
          message: `${file.name} has been ${file.isArchived ? 'unarchived' : 'archived'}`,
          color: 'blue',
        });
      } catch (err) {
        notifications.show({
          title: 'Error',
          message: `Failed to ${file.isArchived ? 'unarchive' : 'archive'} file: ${err}`,
          color: 'red',
        });
      }
    },
    [workspace],
  );

  const value = useMemo<WorkspaceContextValue>(
    () => ({
      workspace,
      allWorkspaces,
      files,
      switchWorkspace,
      toggleFavorite,
      toggleFileArchive,
    }),
    [workspace, allWorkspaces, files, switchWorkspace, toggleFavorite, toggleFileArchive],
  );

  return <WorkspaceContext.Provider value={value}>{children}</WorkspaceContext.Provider>;
}

export function useWorkspaceContext(): WorkspaceContextValue {
  const context = useContext(WorkspaceContext);
  if (!context) {
    throw new Error('useWorkspaceContext must be used within a WorkspaceProvider');
  }
  return context;
}

