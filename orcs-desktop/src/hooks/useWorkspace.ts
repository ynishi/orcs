import { WorkspaceProvider, useWorkspaceContext, type WorkspaceContextValue } from '../context/WorkspaceContext';

export type UseWorkspaceResult = WorkspaceContextValue;

export { WorkspaceProvider };

export function useWorkspace(): UseWorkspaceResult {
  return useWorkspaceContext();
}
