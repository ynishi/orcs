import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Session } from '../types/session';

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      console.log('[useSessions] Loading sessions from backend...');
      // セッション一覧を取得
      const sessionList = await invoke<Session[]>('list_sessions');
      console.log('[useSessions] Loaded sessions:', sessionList.map(s => ({ id: s.id, workspace_id: s.workspace_id })));
      setSessions(sessionList);

      // 現在アクティブなセッションを取得
      const activeSession = await invoke<Session | null>('get_active_session');
      if (activeSession) {
        console.log('[useSessions] Active session:', activeSession.id, 'workspace_id:', activeSession.workspace_id);
        setCurrentSessionId(activeSession.id);
      }
    } catch (err) {
      console.error('Failed to load sessions:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  // 初回ロード: セッション一覧と現在のセッションを取得
  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  // Listen for workspace-switched events to refresh session list
  useEffect(() => {
    const unlisten = listen<string>('workspace-switched', async () => {
      console.log('[useSessions] workspace-switched event received, refreshing sessions');
      await loadSessions();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [loadSessions]);

  const createSession = async (): Promise<string> => {
    try {
      const newSession = await invoke<Session>('create_session');
      setCurrentSessionId(newSession.id);
      // Reload sessions to ensure workspace_id and other fields are properly set
      await loadSessions();
      return newSession.id;
    } catch (err) {
      console.error('Failed to create session:', err);
      throw new Error(`Failed to create session: ${err}`);
    }
  };

  const switchSession = async (sessionId: string): Promise<Session> => {
    try {
      const session = await invoke<Session>('switch_session', { sessionId });
      setCurrentSessionId(sessionId);
      await loadSessions(); // リロード
      return session;
    } catch (err) {
      console.error('Failed to switch session:', err);
      throw new Error(`Failed to switch session: ${err}`);
    }
  };

  const deleteSession = async (sessionId: string): Promise<void> => {
    try {
      await invoke('delete_session', { sessionId });

      // 削除されたセッションが現在のセッションだった場合
      if (currentSessionId === sessionId) {
        // 削除前のセッションリストから次のセッションを選択
        const currentIndex = sessions.findIndex(s => s.id === sessionId);
        const remainingSessions = sessions.filter(s => s.id !== sessionId);

        if (remainingSessions.length > 0) {
          // 次のセッションを選択（現在のインデックスか、それより前）
          const nextIndex = Math.min(currentIndex, remainingSessions.length - 1);
          const nextSession = remainingSessions[nextIndex];
          console.log(`[useSessions] Switching to next session: ${nextSession.id}`);
          await invoke('switch_session', { sessionId: nextSession.id });
          setCurrentSessionId(nextSession.id);
        } else {
          // 最後の1つだった場合は新しいセッションを作成
          console.log('[useSessions] No remaining sessions, creating new one');
          await createSession();
        }
      }

      // セッションリストをリロード
      await loadSessions();
    } catch (err) {
      console.error('Failed to delete session:', err);
      throw new Error(`Failed to delete session: ${err}`);
    }
  };

  const renameSession = async (sessionId: string, newTitle: string): Promise<void> => {
    try {
      await invoke('rename_session', { sessionId, newTitle });
      // Update local state
      setSessions(prev =>
        prev.map(s => s.id === sessionId ? { ...s, title: newTitle } : s)
      );
    } catch (err) {
      console.error('Failed to rename session:', err);
      throw new Error(`Failed to rename session: ${err}`);
    }
  };

  const saveCurrentSession = async () => {
    try {
      await invoke('save_current_session');
    } catch (err) {
      console.error('Failed to save session:', err);
    }
  };

  return {
    sessions,
    currentSessionId,
    loading,
    error,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
    saveCurrentSession,
    refreshSessions: loadSessions,
  };
}
