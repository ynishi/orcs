import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Workspace, UploadedFile } from '../types/workspace';

// Global fetch coordination to prevent duplicate fetches across multiple hook instances
let isFetchingAllWorkspaces = false;
let pendingFetchPromise: Promise<RawWorkspace[]> | null = null;

/**
 * Raw uploaded file structure as returned from Rust/Tauri with snake_case fields.
 */
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
}

/**
 * Raw workspace data structure as returned from Rust/Tauri with snake_case fields.
 * We need to convert this to our camelCase TypeScript types.
 */
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

/**
 * Converts a raw workspace object from Rust (snake_case) to TypeScript (camelCase).
 */
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

/**
 * Hook return type for useWorkspace.
 */
export interface UseWorkspaceResult {
  /** The current workspace, or null if not loaded */
  workspace: Workspace | null;
  /** List of all registered workspaces */
  allWorkspaces: Workspace[];
  /** List of uploaded files in the workspace */
  files: UploadedFile[];
  /** Whether the workspace is currently being loaded */
  isLoading: boolean;
  /** Error message if workspace loading failed */
  error: string | null;
  /** Manually refresh the workspace data */
  refresh: () => Promise<void>;
  /** Switch to a different workspace */
  switchWorkspace: (sessionId: string, workspaceId: string) => Promise<void>;
  /** Toggle the favorite status of a workspace */
  toggleFavorite: (workspaceId: string) => Promise<void>;
  /** Refresh the list of all workspaces */
  refreshWorkspaces: () => Promise<void>;
}

/**
 * Custom hook for managing workspace state and file listings.
 *
 * This hook:
 * - Automatically fetches the current workspace on mount
 * - Loads the list of uploaded files for the workspace
 * - Provides loading and error states
 * - Exposes a refresh function to manually reload workspace data
 *
 * @returns {UseWorkspaceResult} Workspace state and control functions
 *
 * @example
 * ```tsx
 * function WorkspaceComponent() {
 *   const { workspace, files, isLoading, error, refresh } = useWorkspace();
 *
 *   if (isLoading) return <div>Loading workspace...</div>;
 *   if (error) return <div>Error: {error}</div>;
 *   if (!workspace) return <div>No workspace found</div>;
 *
 *   return (
 *     <div>
 *       <h1>{workspace.name}</h1>
 *       <p>Files: {files.length}</p>
 *       <button onClick={refresh}>Refresh</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useWorkspace(): UseWorkspaceResult {
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [allWorkspaces, setAllWorkspaces] = useState<Workspace[]>([]);
  const [files, setFiles] = useState<UploadedFile[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetches the current workspace and its files from the backend.
   */
  const fetchWorkspace = useCallback(async () => {
    console.log('[useWorkspace] Fetching current workspace...');
    setIsLoading(true);
    setError(null);

    try {
      // Call the Tauri command to get the current workspace
      const rawWorkspace = await invoke<RawWorkspace>('get_current_workspace');
      console.log('[useWorkspace] Current workspace:', rawWorkspace.id, rawWorkspace.name);

      // Convert from snake_case to camelCase
      const convertedWorkspace = convertWorkspace(rawWorkspace);
      setWorkspace(convertedWorkspace);

      // Fetch the list of uploaded files for this workspace
      try {
        const rawFiles = await invoke<RawUploadedFile[]>('list_workspace_files', {
          workspaceId: convertedWorkspace.id,
        });

        // Convert files from snake_case to camelCase
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
        }));

        setFiles(convertedFiles);
      } catch (fileError) {
        // If file listing fails, log the error but don't fail the entire operation
        console.error('Failed to list workspace files:', fileError);
        setFiles([]);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to fetch workspace:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Fetches the list of all workspaces.
   * Uses global coordination to prevent duplicate fetches when multiple components mount simultaneously.
   */
  const fetchAllWorkspaces = useCallback(async () => {
    try {
      console.log('[useWorkspace] Fetching all workspaces...');

      // If already fetching, wait for the existing promise
      if (isFetchingAllWorkspaces && pendingFetchPromise) {
        console.log('[useWorkspace] Fetch already in progress, waiting...');
        const rawWorkspaces = await pendingFetchPromise;
        const converted = rawWorkspaces.map(convertWorkspace);
        setAllWorkspaces(converted);
        return;
      }

      // Start new fetch
      isFetchingAllWorkspaces = true;
      pendingFetchPromise = invoke<RawWorkspace[]>('list_workspaces');

      const rawWorkspaces = await pendingFetchPromise;
      console.log('[useWorkspace] Received', rawWorkspaces.length, 'raw workspaces');
      const converted = rawWorkspaces.map(convertWorkspace);
      console.log('[useWorkspace] Setting allWorkspaces state with', converted.length, 'workspaces');
      setAllWorkspaces(converted);

      // Clear the fetch state after a short delay to allow other hooks to use the result
      setTimeout(() => {
        isFetchingAllWorkspaces = false;
        pendingFetchPromise = null;
      }, 100);
    } catch (err) {
      console.error('[useWorkspace] Failed to fetch all workspaces:', err);
      isFetchingAllWorkspaces = false;
      pendingFetchPromise = null;
    }
  }, []);

  /**
   * Switches to a different workspace.
   */
  const switchWorkspace = useCallback(async (sessionId: string, workspaceId: string) => {
    try {
      await invoke('switch_workspace', { sessionId, workspaceId });
      // The workspace-switched event will trigger a refresh
    } catch (err) {
      console.error('Failed to switch workspace:', err);
      throw err;
    }
  }, []);

  /**
   * Toggles the favorite status of a workspace.
   */
  const toggleFavorite = useCallback(async (workspaceId: string) => {
    try {
      await invoke('toggle_favorite_workspace', { workspaceId });
      // Refresh workspace lists after toggling
      await fetchAllWorkspaces();
      if (workspace?.id === workspaceId) {
        await fetchWorkspace();
      }
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
      throw err;
    }
  }, [workspace, fetchWorkspace, fetchAllWorkspaces]);

  // Fetch workspace on mount
  useEffect(() => {
    fetchWorkspace();
    fetchAllWorkspaces();
  }, [fetchWorkspace, fetchAllWorkspaces]);

  return {
    workspace,
    allWorkspaces,
    files,
    isLoading,
    error,
    refresh: fetchWorkspace,
    switchWorkspace,
    toggleFavorite,
    refreshWorkspaces: fetchAllWorkspaces,
  };
}
