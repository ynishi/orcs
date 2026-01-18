import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Session } from '../types/session';
import { useAppStateStore } from '../stores/appStateStore';

export interface SessionContextValue {
  sessions: Session[];
  // currentSessionId removed - use useAppStateStore().appState?.activeSessionId
  loading: boolean;
  error: string | null;
  createSession: (workspaceId?: string) => Promise<string>;
  switchSession: (sessionId: string) => Promise<Session>;
  deleteSession: (sessionId: string) => Promise<void>;
  renameSession: (sessionId: string, newTitle: string) => Promise<void>;
  saveCurrentSession: () => Promise<void>;
  refreshSessions: () => Promise<void>;
  updateSession: (sessionId: string) => Promise<void>;
}

const SessionContext = createContext<SessionContextValue | undefined>(undefined);

interface SessionProviderProps {
  children: ReactNode;
}

export function SessionProvider({ children }: SessionProviderProps) {
  const [sessions, setSessions] = useState<Session[]>([]);
  // currentSessionId removed - use useAppStateStore().appState?.activeSessionId
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const sessionList = await invoke<Session[]>('list_sessions');
      setSessions(sessionList);

      // activeSessionId is managed by appStateStore (SSOT)
      // get_active_session invoke removed - appStateStore already has it
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
      // If workspace_id not provided, get from appStateStore
      const appState = useAppStateStore.getState().appState;
      const finalWorkspaceId = workspaceId || appState?.defaultWorkspaceId;

      if (!finalWorkspaceId) {
        throw new Error('No default workspace available');
      }

      const newSession = await invoke<Session>('create_session', { workspaceId: finalWorkspaceId });
      // setCurrentSessionId removed - Rust emits app-state:update event
      // appStateStore automatically updates activeSessionId

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
        // setCurrentSessionId removed - Rust emits app-state:update event
        // appStateStore automatically updates activeSessionId
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
        // Get current session ID from appStateStore
        const appState = useAppStateStore.getState().appState;
        const currentSessionId = appState?.activeSessionId ?? null;

        await invoke('delete_session', { sessionId });

        const remainingSessions = sessions.filter((s) => s.id !== sessionId);
        setSessions(remainingSessions);

        if (currentSessionId === sessionId) {
          const currentIndex = sessions.findIndex((s) => s.id === sessionId);

          if (remainingSessions.length > 0) {
            const nextIndex = Math.min(Math.max(currentIndex, 0), remainingSessions.length - 1);
            const nextSession = remainingSessions[nextIndex];
            await invoke('switch_session', { sessionId: nextSession.id });
            // setCurrentSessionId removed - Rust emits app-state:update event
          } else {
            // No sessions left - create new one
            // activeSessionId will be updated by create_session event
            await createSession();
          }
        }
      } catch (err) {
        console.error('Failed to delete session:', err);
        throw new Error(`Failed to delete session: ${err}`);
      }
    },
    [sessions, createSession],
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

  const updateSession = useCallback(async (sessionId: string) => {
    try {
      const updatedSession = await invoke<Session | null>('get_session', { sessionId });
      if (updatedSession) {
        setSessions((prev) =>
          prev.map((s) => (s.id === sessionId ? updatedSession : s))
        );
      }
    } catch (err) {
      console.error('Failed to update session:', err);
    }
  }, []);

  const value = useMemo<SessionContextValue>(
    () => ({
      sessions,
      // currentSessionId removed - use useAppStateStore().appState?.activeSessionId
      loading,
      error,
      createSession,
      switchSession: switchSessionHandler,
      deleteSession,
      renameSession,
      saveCurrentSession,
      refreshSessions,
      updateSession,
    }),
    [
      sessions,
      loading,
      error,
      createSession,
      switchSessionHandler,
      deleteSession,
      renameSession,
      saveCurrentSession,
      refreshSessions,
      updateSession,
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
