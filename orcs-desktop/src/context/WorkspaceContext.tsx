import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notifications } from '@mantine/notifications';
import type { Workspace, UploadedFile } from '../types/workspace';

export interface WorkspaceContextValue {
  workspace: Workspace | null;
  allWorkspaces: Workspace[];
  files: UploadedFile[];
  isLoading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  switchWorkspace: (sessionId: string, workspaceId: string) => Promise<void>;
  toggleFavorite: (workspaceId: string) => Promise<void>;
  toggleFileArchive: (file: UploadedFile) => Promise<void>;
  refreshWorkspaces: () => Promise<void>;
}

const WorkspaceContext = createContext<WorkspaceContextValue | undefined>(undefined);

interface WorkspaceProviderProps {
  children: ReactNode;
}

interface RawUploadedFile {
  id: string;
  name: string;
  path: string;
  mime_type: string;
  size: number;
  uploaded_at: number;
  session_id?: string;
  message_timestamp?: string;
  author?: string;
  is_archived: boolean;
}

interface RawWorkspace {
  id: string;
  name: string;
  root_path: string;
  workspace_dir: string;
  resources: {
    uploaded_files: RawUploadedFile[];
    temp_files: Array<{
      id: string;
      path: string;
      purpose: string;
      created_at: number;
      auto_delete: boolean;
    }>;
  };
  project_context: {
    languages: string[];
    build_system?: string;
    description?: string;
    repository_url?: string;
    metadata: Record<string, string>;
  };
  last_accessed: number;
  is_favorite: boolean;
}

function convertWorkspace(raw: RawWorkspace): Workspace {
  return {
    id: raw.id,
    name: raw.name,
    rootPath: raw.root_path,
    workspaceDir: raw.workspace_dir,
    resources: {
      uploadedFiles: raw.resources.uploaded_files.map(file => ({
        id: file.id,
        name: file.name,
        path: file.path,
        mimeType: file.mime_type,
        size: file.size,
        uploadedAt: file.uploaded_at,
        sessionId: file.session_id,
        messageTimestamp: file.message_timestamp,
        author: file.author,
        isArchived: file.is_archived,
      })),
      tempFiles: raw.resources.temp_files.map(file => ({
        id: file.id,
        path: file.path,
        purpose: file.purpose,
        createdAt: file.created_at,
        autoDelete: file.auto_delete,
      })),
    },
    projectContext: {
      languages: raw.project_context.languages,
      buildSystem: raw.project_context.build_system,
      description: raw.project_context.description,
      repositoryUrl: raw.project_context.repository_url,
      metadata: raw.project_context.metadata,
    },
    lastAccessed: raw.last_accessed,
    isFavorite: raw.is_favorite,
  };
}

export function WorkspaceProvider({ children }: WorkspaceProviderProps) {
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [allWorkspaces, setAllWorkspaces] = useState<Workspace[]>([]);
  const [files, setFiles] = useState<UploadedFile[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const workspaceIdRef = useRef<string | null>(null);
  const isFetchingAllRef = useRef(false);
  const pendingFetchRef = useRef<Promise<RawWorkspace[]> | null>(null);

  const fetchWorkspace = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const rawWorkspace = await invoke<RawWorkspace>('get_current_workspace');
      const convertedWorkspace = convertWorkspace(rawWorkspace);
      workspaceIdRef.current = convertedWorkspace.id;
      setWorkspace(convertedWorkspace);

      try {
        const rawFiles = await invoke<RawUploadedFile[]>('list_workspace_files', {
          workspaceId: convertedWorkspace.id,
        });

        const convertedFiles: UploadedFile[] = rawFiles.map((file: RawUploadedFile) => ({
          id: file.id,
          name: file.name,
          path: file.path,
          mimeType: file.mime_type,
          size: file.size,
          uploadedAt: file.uploaded_at,
          sessionId: file.session_id,
          messageTimestamp: file.message_timestamp,
          author: file.author,
          isArchived: file.is_archived,
        }));

        // Sort by upload time (most recent first)
        const sortedFiles = convertedFiles.sort((a, b) => b.uploadedAt - a.uploadedAt);

        setFiles(sortedFiles);
      } catch (fileError) {
        console.error('Failed to list workspace files:', fileError);
        setFiles([]);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      setWorkspace(null);
      setFiles([]);
      console.error('Failed to fetch workspace:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const fetchAllWorkspaces = useCallback(async () => {
    try {
      if (isFetchingAllRef.current && pendingFetchRef.current) {
        const rawWorkspaces = await pendingFetchRef.current;
        const converted = rawWorkspaces.map(convertWorkspace);
        setAllWorkspaces(converted);
        return;
      }

      isFetchingAllRef.current = true;
      pendingFetchRef.current = invoke<RawWorkspace[]>('list_workspaces');

      const rawWorkspaces = await pendingFetchRef.current;
      const converted = rawWorkspaces.map(convertWorkspace);
      setAllWorkspaces(converted);
    } catch (err) {
      console.error('[useWorkspace] Failed to fetch all workspaces:', err);
    } finally {
      isFetchingAllRef.current = false;
      pendingFetchRef.current = null;
    }
  }, []);

  const switchWorkspace = useCallback(async (sessionId: string, workspaceId: string) => {
    try {
      await invoke('switch_workspace', { sessionId, workspaceId });
      // The workspace-switched event triggers refresh via listener.
    } catch (err) {
      console.error('Failed to switch workspace:', err);
      throw err;
    }
  }, []);

  const toggleFavorite = useCallback(
    async (workspaceId: string) => {
      try {
        await invoke('toggle_favorite_workspace', { workspaceId });
        await fetchAllWorkspaces();
        if (workspaceIdRef.current === workspaceId) {
          await fetchWorkspace();
        }
      } catch (err) {
        console.error('Failed to toggle favorite:', err);
        throw err;
      }
    },
    [fetchAllWorkspaces, fetchWorkspace],
  );

  const toggleFileArchive = useCallback(
    async (file: UploadedFile) => {
      if (!workspace) return;

      try {
        await invoke('toggle_workspace_file_archive', {
          workspaceId: workspace.id,
          fileId: file.id,
        });

        // Refresh files
        await fetchWorkspace();

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
    [workspace, fetchWorkspace],
  );

  useEffect(() => {
    fetchWorkspace();
    fetchAllWorkspaces();
  }, [fetchWorkspace, fetchAllWorkspaces]);

  useEffect(() => {
    let unlistenFiles: (() => void) | undefined;
    let canceled = false;

    (async () => {
      const { listen } = await import('@tauri-apps/api/event');
      if (canceled) return;

      unlistenFiles = await listen<string>('workspace-files-changed', async (event) => {
        if (event.payload && workspaceIdRef.current && event.payload !== workspaceIdRef.current) {
          return;
        }
        await fetchWorkspace();
      });
    })();

    return () => {
      canceled = true;
      if (unlistenFiles) {
        unlistenFiles();
      }
    };
  }, [fetchWorkspace]);

  const value = useMemo<WorkspaceContextValue>(
    () => ({
      workspace,
      allWorkspaces,
      files,
      isLoading,
      error,
      refresh: fetchWorkspace,
      switchWorkspace,
      toggleFavorite,
      toggleFileArchive,
      refreshWorkspaces: fetchAllWorkspaces,
    }),
    [workspace, allWorkspaces, files, isLoading, error, fetchWorkspace, switchWorkspace, toggleFavorite, toggleFileArchive, fetchAllWorkspaces],
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

