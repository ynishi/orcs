// Re-export generated types from schema
export type {
  ConversationMessage,
  MessageMetadata,
  MessageRole,
  Plan,
  ConversationMode,
  AutoChatConfig,
  StopCondition,
  SessionType,
} from '../bindings/generated';

import type { Message, MessageType } from './message';
import type { ConversationMessage, MessageMetadata, Plan, AutoChatConfig } from '../bindings/generated';

/**
 * Application mode for session state management.
 *
 * Note: This is manually defined because schema-bridge doesn't support
 * Rust's `#[serde(tag = "type", content = "data")]` tagged enums yet.
 */
export type AppMode =
  | { type: 'Idle' }
  | { type: 'AwaitingConfirmation'; data: { plan: Plan } };

/**
 * Full Session interface with persona_histories and system_messages.
 * Extends SessionType from generated types.
 *
 * Note: SessionType (from Rust) excludes persona_histories due to schema-bridge limitations.
 * This interface adds the missing fields for frontend use.
 */
export interface Session {
  id: string;
  title: string;
  createdAt: string; // ISO 8601 timestamp (was created_at)
  updatedAt: string; // ISO 8601 timestamp (was updated_at)
  currentPersonaId: string; // was current_persona_id
  personaHistories: Record<string, ConversationMessage[]>; // was persona_histories
  appMode: AppMode; // was app_mode
  workspaceId: string; // was workspace_id
  activeParticipantIds: string[]; // was active_participant_ids
  executionStrategy: string; // "broadcast" or "sequential" (was execution_strategy)
  systemMessages: ConversationMessage[]; // was system_messages
  participants: Record<string, string>;
  participantIcons: Record<string, string>; // was participant_icons
  participantColors: Record<string, string>; // was participant_colors
  participantBackends?: Record<string, string>; // was participant_backends
  participantModels?: Record<string, string | null>; // was participant_models
  isFavorite?: boolean; // was is_favorite
  isArchived?: boolean; // was is_archived
  sortOrder?: number; // was sort_order
  autoChatConfig?: AutoChatConfig; // was auto_chat_config
  isMuted?: boolean; // was is_muted
}

/**
 * Conversation message metadata (alias for backward compatibility)
 * @deprecated Use MessageMetadata from generated types
 */
export type ConversationMessageMetadata = MessageMetadata;

// ============================================================================
// UI用のヘルパー関数
// ============================================================================

/**
 * セッションの総メッセージ数を取得
 */
export function getMessageCount(session: Session): number {
  if (!session || !session.personaHistories) {
    return 0;
  }
  return Object.values(session.personaHistories)
    .flat()
    .length;
}

/**
 * セッション作成日時をDateオブジェクトで取得
 */
export function getCreatedAt(session: Session): Date {
  return new Date(session.createdAt);
}

/**
 * セッション最終更新日時をDateオブジェクトで取得
 */
export function getLastActive(session: Session): Date {
  return new Date(session.updatedAt);
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
  if (!session || !session.personaHistories || !session.currentPersonaId) {
    return [];
  }
  return session.personaHistories[session.currentPersonaId] || [];
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

  // personaHistoriesから取得
  if (session.personaHistories) {
    for (const [personaId, messages] of Object.entries(session.personaHistories)) {
      for (const msg of messages) {
        messagesWithAuthors.push({ message: msg, authorId: personaId });
      }
    }
  }

  // systemMessagesから取得
  if (session.systemMessages) {
    for (const msg of session.systemMessages) {
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

const KNOWN_MESSAGE_TYPES: MessageType[] = [
  'user',
  'ai',
  'system',
  'error',
  'command',
  'task',
  'thinking',
  'shell_output',
];

const isMessageType = (value?: string): value is MessageType =>
  typeof value === 'string' && (KNOWN_MESSAGE_TYPES as string[]).includes(value);

function resolveMessageType(msg: ConversationMessage): MessageType {
  const metadataType = msg.metadata?.systemMessageType ?? undefined;
  if (isMessageType(metadataType)) {
    return metadataType;
  }

  if (msg.role === 'User') {
    return 'user';
  }

  if (msg.role === 'Assistant') {
    return 'ai';
  }

  if (msg.metadata?.errorSeverity === 'critical') {
    return 'error';
  }

  return 'system';
}

/**
 * ConversationMessageとauthorIdをUI用のMessageに変換
 */
export function convertToUIMessageWithAuthor(
  msg: ConversationMessage,
  authorId: string,
  participants: Record<string, string>,
  participantIcons: Record<string, string> = {},
  participantColors: Record<string, string> = {},
  participantBackends: Record<string, string> = {},
  participantModels: Record<string, string | null> = {},
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

  const messageType = resolveMessageType(msg);

  // Resolve author name based on message role
  let author: string;
  let icon: string | undefined;
  let baseColor: string | undefined;
  let backend: string | undefined;
  let modelName: string | null | undefined;
  if (msg.role === 'User') {
    // User messages always use the configured nickname
    author = userNickname;
    // User has no icon, color, backend, or model
  } else if (msg.role === 'System') {
    author = 'SYSTEM';
    // System has no icon, color, backend, or model
  } else {
    // Assistant: participantsマップから名前を解決、見つからない場合はauthorIdをそのまま使用
    author = participants[authorId] || authorId;
    // Get icon, color, backend, and model from participant maps if available
    icon = participantIcons[authorId];
    baseColor = participantColors[authorId];
    backend = participantBackends[authorId];
    modelName = participantModels[authorId];
  }

  // Convert file paths to AttachedFile objects (without preview data initially)
  const attachments = msg.attachments && msg.attachments.length > 0
    ? msg.attachments.map(filePath => {
        const fileName = filePath.split('/').pop() || 'unknown';
        return {
          name: fileName,
          path: filePath,
          mimeType: 'application/octet-stream', // Will be updated later
          size: 0, // Will be updated later
        };
      })
    : undefined;

  return {
    id: `${msg.timestamp}-${Math.random()}`,
    type: messageType,
    author,
    text: msg.content,
    timestamp: new Date(msg.timestamp),
    icon,
    baseColor,
    backend,
    modelName,
    attachments,
  };
}

/**
 * ConversationMessageをUI用のMessageに変換（後方互換性のため残す）
 */
export function convertToUIMessage(msg: ConversationMessage, userNickname: string = 'You'): Message {
  const messageType = resolveMessageType(msg);

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
    convertToUIMessageWithAuthor(
      item.message,
      item.authorId,
      session.participants || {},
      session.participantIcons || {},
      session.participantColors || {},
      session.participantBackends || {},
      session.participantModels || {},
      userNickname
    )
  );
}
