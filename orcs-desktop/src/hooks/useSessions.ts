import { SessionProvider, useSessionContext, type SessionContextValue } from '../context/SessionContext';

export type UseSessionsResult = SessionContextValue;

export { SessionProvider };

export function useSessions(): UseSessionsResult {
  return useSessionContext();
}
