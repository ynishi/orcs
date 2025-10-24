import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Session } from '../types/session';

export function useSessions() {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 初回ロード: セッション一覧と現在のセッションを取得
  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    try {
      setLoading(true);
      setError(null);

      // セッション一覧を取得
      const sessionList = await invoke<Session[]>('list_sessions');
      setSessions(sessionList);

      // 現在アクティブなセッションを取得
      const activeSession = await invoke<Session | null>('get_active_session');
      if (activeSession) {
        setCurrentSessionId(activeSession.id);
      }
    } catch (err) {
      console.error('Failed to load sessions:', err);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const createSession = async (): Promise<string> => {
    try {
      const newSession = await invoke<Session>('create_session');
      // Update local state directly instead of reloading
      setSessions(prev => [...prev, newSession]);
      setCurrentSessionId(newSession.id);
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
      await loadSessions(); // リロード

      // 削除されたセッションが現在のセッションだった場合
      if (currentSessionId === sessionId) {
        // 新しいセッションを作成
        await createSession();
      }
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
