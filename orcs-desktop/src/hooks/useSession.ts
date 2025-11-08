import { SessionProvider, useSessionContext, type SessionContextValue } from '../context/SessionContext';

export type UseSessionResult = SessionContextValue;

export { SessionProvider };

export function useSession(): UseSessionResult {
  return useSessionContext();
}
