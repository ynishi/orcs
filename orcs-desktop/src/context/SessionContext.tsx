import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Session } from '../types/session';

export interface SessionContextValue {
  sessions: Session[];
  currentSessionId: string | null;
  loading: boolean;
  error: string | null;
  createSession: (workspaceId?: string) => Promise<string>;
  switchSession: (sessionId: string) => Promise<Session>;
  deleteSession: (sessionId: string) => Promise<void>;
  renameSession: (sessionId: string, newTitle: string) => Promise<void>;
  saveCurrentSession: () => Promise<void>;
  refreshSessions: () => Promise<void>;
}

const SessionContext = createContext<SessionContextValue | undefined>(undefined);

interface SessionProviderProps {
  children: ReactNode;
}

export function SessionProvider({ children }: SessionProviderProps) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const sessionList = await invoke<Session[]>('list_sessions');
      setSessions(sessionList);

      const activeSession = await invoke<Session | null>('get_active_session');
      if (activeSession) {
        setCurrentSessionId(activeSession.id);
      } else {
        setCurrentSessionId(null);
      }
    } catch (err) {
      console.error('Failed to load sessions:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  const createSession = useCallback(async (workspaceId?: string): Promise<string> => {
    try {
      // If workspace_id not provided, get default workspace_id
      const finalWorkspaceId = workspaceId || await invoke<string>('get_default_workspace_id');

      const newSession = await invoke<Session>('create_session', { workspaceId: finalWorkspaceId });
      setCurrentSessionId(newSession.id);
      // Add new session to local sessions array instead of reloading all sessions
      setSessions((prev) => [...prev, newSession]);
      return newSession.id;
    } catch (err) {
      console.error('Failed to create session:', err);
      throw new Error(`Failed to create session: ${err}`);
    }
  }, []);

  const switchSessionHandler = useCallback(
    async (sessionId: string): Promise<Session> => {
      try {
        const session = await invoke<Session>('switch_session', { sessionId });
        setCurrentSessionId(sessionId);
        // Note: loadSessions() removed here to avoid unnecessary re-fetching
        // Tab switching should use cached session data
        // Updated session data (e.g., updated_at) will be reflected on next refresh
        return session;
      } catch (err) {
        console.error('Failed to switch session:', err);
        throw new Error(`Failed to switch session: ${err}`);
      }
    },
    [],
  );

  const deleteSession = useCallback(
    async (sessionId: string) => {
      try {
        await invoke('delete_session', { sessionId });

        const remainingSessions = sessions.filter((s) => s.id !== sessionId);
        setSessions(remainingSessions);

        if (currentSessionId === sessionId) {
          const currentIndex = sessions.findIndex((s) => s.id === sessionId);

          if (remainingSessions.length > 0) {
            const nextIndex = Math.min(Math.max(currentIndex, 0), remainingSessions.length - 1);
            const nextSession = remainingSessions[nextIndex];
            await invoke('switch_session', { sessionId: nextSession.id });
            setCurrentSessionId(nextSession.id);
          } else {
            setCurrentSessionId(null);
            await createSession();
          }
        }
      } catch (err) {
        console.error('Failed to delete session:', err);
        throw new Error(`Failed to delete session: ${err}`);
      }
    },
    [currentSessionId, sessions, createSession],
  );

  const renameSession = useCallback(async (sessionId: string, newTitle: string) => {
    try {
      await invoke('rename_session', { sessionId, newTitle });
      setSessions((prev) =>
        prev.map((s) => (s.id === sessionId ? { ...s, title: newTitle } : s)),
      );
    } catch (err) {
      console.error('Failed to rename session:', err);
      throw new Error(`Failed to rename session: ${err}`);
    }
  }, []);

  const saveCurrentSession = useCallback(async () => {
    try {
      await invoke('save_current_session');
    } catch (err) {
      console.error('Failed to save session:', err);
    }
  }, []);

  const refreshSessions = useCallback(async () => {
    await loadSessions();
  }, [loadSessions]);

  const value = useMemo<SessionContextValue>(
    () => ({
      sessions,
      currentSessionId,
      loading,
      error,
      createSession,
      switchSession: switchSessionHandler,
      deleteSession,
      renameSession,
      saveCurrentSession,
      refreshSessions,
    }),
    [
      sessions,
      currentSessionId,
      loading,
      error,
      createSession,
      switchSessionHandler,
      deleteSession,
      renameSession,
      saveCurrentSession,
      refreshSessions,
    ],
  );

  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSessionContext(): SessionContextValue {
  const context = useContext(SessionContext);
  if (!context) {
    throw new Error('useSessionContext must be used within a SessionProvider');
  }
  return context;
}
