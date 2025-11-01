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
  workspace_id?: string; // Optional workspace ID for filtering
  active_participant_ids: string[]; // Active participants
  execution_strategy: string; // "broadcast" or "sequential"
  system_messages: ConversationMessage[]; // System messages (join/leave events, etc.)
  participants: Record<string, string>; // Persona ID -> name mapping
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
  if (!session || !session.persona_histories) {
    return 0;
  }
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
  if (!session || !session.persona_histories || !session.current_persona_id) {
    return [];
  }
  return session.persona_histories[session.current_persona_id] || [];
}

/**
 * メッセージと発話者情報のペア
 */
export interface MessageWithAuthor {
  message: ConversationMessage;
  authorId: string; // Persona ID or "You" or "System"
}

/**
 * 全Personaの会話履歴とsystem_messagesを時系列順に統合（発話者情報付き）
 */
export function getAllMessagesWithAuthors(session: Session): MessageWithAuthor[] {
  if (!session) {
    return [];
  }

  const messagesWithAuthors: MessageWithAuthor[] = [];

  // persona_historiesから取得
  if (session.persona_histories) {
    for (const [personaId, messages] of Object.entries(session.persona_histories)) {
      for (const msg of messages) {
        messagesWithAuthors.push({ message: msg, authorId: personaId });
      }
    }
  }

  // system_messagesから取得
  if (session.system_messages) {
    for (const msg of session.system_messages) {
      messagesWithAuthors.push({ message: msg, authorId: 'System' });
    }
  }

  // 時系列順にソート
  return messagesWithAuthors.sort((a, b) =>
    new Date(a.message.timestamp).getTime() - new Date(b.message.timestamp).getTime()
  );
}

/**
 * 全Personaの会話履歴とsystem_messagesを時系列順に統合
 */
export function getAllMessages(session: Session): ConversationMessage[] {
  return getAllMessagesWithAuthors(session).map(item => item.message);
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
 * ConversationMessageとauthorIdをUI用のMessageに変換
 */
export function convertToUIMessageWithAuthor(
  msg: ConversationMessage,
  authorId: string,
  participants: Record<string, string>,
  userNickname: string = 'You'
): Message {
  // Check if this is an error message (special authorId "Error")
  if (authorId === 'Error') {
    return {
      id: `${msg.timestamp}-${Math.random()}`,
      type: 'error',
      author: '',
      text: msg.content,
      timestamp: new Date(msg.timestamp),
    };
  }

  const messageType: MessageType = msg.role === 'User' ? 'user' : msg.role === 'Assistant' ? 'ai' : 'system';

  // authorIdが"You"ならユーザー、"System"ならシステム、それ以外はペルソナIDから名前を解決
  let author: string;
  if (msg.role === 'User') {
    author = authorId === 'You' ? userNickname : (participants[authorId] || authorId);
  } else if (msg.role === 'System') {
    author = 'SYSTEM';
  } else {
    // Assistant: participantsマップから名前を解決、見つからない場合はauthorIdをそのまま使用
    author = participants[authorId] || authorId;
  }

  return {
    id: `${msg.timestamp}-${Math.random()}`,
    type: messageType,
    author,
    text: msg.content,
    timestamp: new Date(msg.timestamp),
  };
}

/**
 * ConversationMessageをUI用のMessageに変換（後方互換性のため残す）
 */
export function convertToUIMessage(msg: ConversationMessage, userNickname: string = 'You'): Message {
  const messageType: MessageType = msg.role === 'User' ? 'user' : msg.role === 'Assistant' ? 'ai' : 'system';

  return {
    id: `${msg.timestamp}-${Math.random()}`,
    type: messageType,
    author: msg.role === 'User' ? userNickname : msg.role === 'Assistant' ? 'AI' : 'System',
    text: msg.content,
    timestamp: new Date(msg.timestamp),
  };
}

/**
 * セッションの会話履歴をUI用Messageの配列に変換
 */
export function convertSessionToMessages(session: Session, userNickname: string = 'You'): Message[] {
  return getAllMessagesWithAuthors(session).map(item =>
    convertToUIMessageWithAuthor(item.message, item.authorId, session.participants || {}, userNickname)
  );
}
