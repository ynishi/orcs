import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Workspace, UploadedFile } from '../types/workspace';

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
}

/**
 * Raw workspace data structure as returned from Rust/Tauri with snake_case fields.
 * We need to convert this to our camelCase TypeScript types.
 */
interface RawWorkspace {
  id: string;
  name: string;
  root_path: string;
  resources: {
    uploaded_files: RawUploadedFile[];
    generated_docs: Array<{
      id: string;
      title: string;
      path: string;
      doc_type: string;
      session_id: string;
      generated_at: number;
    }>;
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
}

/**
 * Converts a raw workspace object from Rust (snake_case) to TypeScript (camelCase).
 */
function convertWorkspace(raw: RawWorkspace): Workspace {
  return {
    id: raw.id,
    name: raw.name,
    rootPath: raw.root_path,
    resources: {
      uploadedFiles: raw.resources.uploaded_files.map(file => ({
        id: file.id,
        name: file.name,
        path: file.path,
        mimeType: file.mime_type,
        size: file.size,
        uploadedAt: file.uploaded_at,
      })),
      generatedDocs: raw.resources.generated_docs.map(doc => ({
        id: doc.id,
        title: doc.title,
        path: doc.path,
        docType: doc.doc_type,
        sessionId: doc.session_id,
        generatedAt: doc.generated_at,
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
  };
}

/**
 * Hook return type for useWorkspace.
 */
export interface UseWorkspaceResult {
  /** The current workspace, or null if not loaded */
  workspace: Workspace | null;
  /** List of uploaded files in the workspace */
  files: UploadedFile[];
  /** Whether the workspace is currently being loaded */
  isLoading: boolean;
  /** Error message if workspace loading failed */
  error: string | null;
  /** Manually refresh the workspace data */
  refresh: () => Promise<void>;
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
  const [files, setFiles] = useState<UploadedFile[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetches the current workspace and its files from the backend.
   */
  const fetchWorkspace = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Call the Tauri command to get the current workspace
      const rawWorkspace = await invoke<RawWorkspace>('get_current_workspace');

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

  // Fetch workspace on mount
  useEffect(() => {
    fetchWorkspace();
  }, [fetchWorkspace]);

  return {
    workspace,
    files,
    isLoading,
    error,
    refresh: fetchWorkspace,
  };
}
