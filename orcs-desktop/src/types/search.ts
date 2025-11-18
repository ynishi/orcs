export type SearchScope = 'workspace' | 'local' | 'global';

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
  scope: SearchScope;
  items: SearchResultItem[];
  total_matches: number;
}
