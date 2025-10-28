/**
 * TypeScript type definitions for workspace-related domain models.
 * These types correspond to the Rust domain models in orcs-core/workspace.
 */

/**
 * Represents a file uploaded to the workspace.
 */
export interface UploadedFile {
  /** Unique identifier for the uploaded file */
  id: string;
  /** Original filename */
  name: string;
  /** Path to the stored file */
  path: string;
  /** MIME type of the file */
  mimeType: string;
  /** File size in bytes */
  size: number;
  /** Timestamp when the file was uploaded (Unix timestamp in seconds) */
  uploadedAt: number;
  /** Session ID if this file was saved from a chat message */
  sessionId?: string;
  /** Message timestamp if this file was saved from a chat message (ISO 8601) */
  messageTimestamp?: string;
  /** Author of the file (user ID, persona ID, or "system") */
  author?: string;
}

/**
 * Represents a temporary file created during operations.
 */
export interface TempFile {
  /** Unique identifier for the temp file */
  id: string;
  /** Path to the temporary file */
  path: string;
  /** Purpose or description of the temp file */
  purpose: string;
  /** Timestamp when the file was created (Unix timestamp in seconds) */
  createdAt: number;
  /** Whether the file should be deleted after session ends */
  autoDelete: boolean;
}

/**
 * Collection of all resources managed within a workspace.
 */
export interface WorkspaceResources {
  /** Files uploaded by the user or system */
  uploadedFiles: UploadedFile[];
  /** Temporary files created during session operations */
  tempFiles: TempFile[];
}

/**
 * Project-specific context and metadata.
 */
export interface ProjectContext {
  /** Programming languages detected in the project */
  languages: string[];
  /** Build system or framework (e.g., "cargo", "npm", "maven") */
  buildSystem?: string;
  /** Project description or purpose */
  description?: string;
  /** Git repository URL if available */
  repositoryUrl?: string;
  /** Additional metadata as key-value pairs */
  metadata: Record<string, string>;
}

/**
 * Represents a project-level workspace containing all resources and context
 * associated with a specific project.
 */
export interface Workspace {
  /** Unique identifier for the workspace */
  id: string;
  /** Name of the workspace (typically derived from project name) */
  name: string;
  /** Root directory path of the project */
  rootPath: string;
  /** Directory where workspace data is stored (e.g., ~/.orcs/workspaces/{id}) */
  workspaceDir: string;
  /** Collection of all workspace resources */
  resources: WorkspaceResources;
  /** Project-specific context and metadata */
  projectContext: ProjectContext;
  /** Last accessed timestamp (Unix timestamp in seconds) */
  lastAccessed: number;
  /** Whether this workspace is marked as favorite */
  isFavorite: boolean;
}

/**
 * Session-specific workspace view that references the parent workspace.
 */
export interface SessionWorkspace {
  /** ID of the parent workspace */
  workspaceId: string;
  /** ID of the current session */
  sessionId: string;
  /** Temporary files specific to this session */
  sessionTempFiles: TempFile[];
}
