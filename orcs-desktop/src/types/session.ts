/**
 * セッション（会話履歴）
 */
export interface Session {
  id: string;
  title: string;
  createdAt: Date;
  lastActive: Date;
  messageCount: number;
}

/**
 * セッションのソート順
 */
export function sortSessionsByLastActive(sessions: Session[]): Session[] {
  return [...sessions].sort((a, b) => b.lastActive.getTime() - a.lastActive.getTime());
}

/**
 * セッションタイトルを生成（最初のメッセージから）
 */
export function generateSessionTitle(firstMessage: string): string {
  const truncated = firstMessage.slice(0, 50);
  return truncated.length < firstMessage.length ? `${truncated}...` : truncated;
}
