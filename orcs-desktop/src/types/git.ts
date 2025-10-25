/**
 * Git repository information
 */
export interface GitInfo {
  /** Whether the current directory is in a Git repository */
  is_repo: boolean;
  /** Current branch name (if in a repo) */
  branch: string | null;
  /** Repository name (if in a repo) */
  repo_name: string | null;
}
