/**
 * セッション（会話履歴）
 * Matches Rust's SessionData structure exactly
 */
export interface Session {
  id: string;
  title: string;
  created_at: string; // ISO 8601 timestamp
  updated_at: string; // ISO 8601 timestamp
  current_persona_id: string;
  persona_histories: Record<string, ConversationMessage[]>;
  app_mode: AppMode;
}

/**
 * 会話履歴の1メッセージ
 * Matches Rust's ConversationMessage
 */
export interface ConversationMessage {
  role: 'User' | 'Assistant' | 'System';
  content: string;
  timestamp: string; // ISO 8601 timestamp
}

/**
 * アプリケーションモード
 * Matches Rust's AppMode enum with #[serde(tag = "type", content = "data")]
 */
export type AppMode =
  | { type: 'Idle' }
  | { type: 'AwaitingConfirmation'; data: { plan: Plan } };

/**
 * プラン
 */
export interface Plan {
  steps: string[];
}

// ============================================================================
// UI用のヘルパー関数
// ============================================================================

/**
 * セッションの総メッセージ数を取得
 */
export function getMessageCount(session: Session): number {
  return Object.values(session.persona_histories)
    .flat()
    .length;
}

/**
 * セッション作成日時をDateオブジェクトで取得
 */
export function getCreatedAt(session: Session): Date {
  return new Date(session.created_at);
}

/**
 * セッション最終更新日時をDateオブジェクトで取得
 */
export function getLastActive(session: Session): Date {
  return new Date(session.updated_at);
}

/**
 * セッションを最終更新日時でソート（新しい順）
 */
export function sortSessionsByLastActive(sessions: Session[]): Session[] {
  return [...sessions].sort((a, b) =>
    getLastActive(b).getTime() - getLastActive(a).getTime()
  );
}

/**
 * 現在のPersona IDの会話履歴を取得
 */
export function getCurrentPersonaMessages(session: Session): ConversationMessage[] {
  return session.persona_histories[session.current_persona_id] || [];
}

/**
 * 全Personaの会話履歴を時系列順に統合
 */
export function getAllMessages(session: Session): ConversationMessage[] {
  const allMessages = Object.values(session.persona_histories).flat();
  return allMessages.sort((a, b) =>
    new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );
}

/**
 * セッションタイトルを生成（最初のメッセージから）
 */
export function generateSessionTitle(firstMessage: string): string {
  const truncated = firstMessage.slice(0, 50);
  return truncated.length < firstMessage.length ? `${truncated}...` : truncated;
}

/**
 * AppModeがIdleかどうか判定
 */
export function isIdleMode(mode: AppMode): boolean {
  return mode.type === 'Idle';
}

/**
 * AppModeからPlanを取得（AwaitingConfirmationの場合）
 */
export function getPlan(mode: AppMode): Plan | null {
  if (mode.type === 'AwaitingConfirmation') {
    return mode.data.plan;
  }
  return null;
}

// ============================================================================
// UI Message変換
// ============================================================================

import type { Message, MessageType } from './message';

/**
 * ConversationMessageをUI用のMessageに変換
 */
export function convertToUIMessage(msg: ConversationMessage): Message {
  const messageType: MessageType = msg.role === 'User' ? 'user' : msg.role === 'Assistant' ? 'ai' : 'system';

  return {
    id: `${msg.timestamp}-${Math.random()}`,
    type: messageType,
    author: msg.role === 'User' ? 'You' : msg.role === 'Assistant' ? 'AI' : 'System',
    text: msg.content,
    timestamp: new Date(msg.timestamp),
  };
}

/**
 * セッションの会話履歴をUI用Messageの配列に変換
 */
export function convertSessionToMessages(session: Session): Message[] {
  return getAllMessages(session).map(convertToUIMessage);
}
