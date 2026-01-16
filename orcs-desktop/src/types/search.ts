/**
 * Search options to control what is searched.
 *
 * Default: searches current workspace's sessions and files.
 * - `-p`: also search project files (root_path)
 * - `-a`: search all workspaces' sessions and files
 * - `-f` (or `-ap`): search everything (all workspaces + project files)
 */
export interface SearchOptions {
  /** Search all workspaces instead of just current workspace */
  all_workspaces: boolean;
  /** Include project files (workspace.root_path) in search */
  include_project: boolean;
}

export interface SearchFilters {
  file_types?: string[];
  exclude_paths?: string[];
  max_results?: number;
  context_before?: number;
  context_after?: number;
}

export interface SearchResultItem {
  path: string;
  line_number?: number;
  content: string;
  context_before?: string[];
  context_after?: string[];
}

export interface SearchResult {
  query: string;
  options: SearchOptions;
  items: SearchResultItem[];
  summary?: string;
  total_matches: number;
}
