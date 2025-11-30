/**
 * メッセージタイプの定義
 */
export type MessageType = 'user' | 'ai' | 'system' | 'error' | 'command' | 'task' | 'thinking' | 'shell_output';

/**
 * Streaming dialogue turn from Rust backend
 *
 * Note: Rust uses #[serde(rename_all = "camelCase")] so fields are camelCase
 */
export type StreamingDialogueTurn = {
  sessionId: string; // was session_id
  timestamp: string;
} & StreamingDialogueTurnKind;

export type StreamingDialogueTurnKind =
  | {
      type: 'Chunk';
      author: string;
      content: string;
    }
  | {
      type: 'Final';
    }
  | {
      type: 'Error';
      message: string;
    }
  | {
      type: 'AutoChatProgress';
      currentIteration: number; // was current_iteration
      maxIterations: number; // was max_iterations
    }
  | {
      type: 'AutoChatComplete';
      totalIterations: number; // was total_iterations
    };

/**
 * 添付ファイル情報
 */
export interface AttachedFile {
  name: string;          // ファイル名
  path: string;          // ワークスペース内のパス
  mimeType: string;      // MIMEタイプ
  size: number;          // ファイルサイズ（バイト）
  data?: string;         // Base64エンコードされたデータ（画像の場合）
}

/**
 * メッセージインターフェース
 */
export interface Message {
  id: string;
  type: MessageType;
  author: string;
  text: string;
  timestamp: Date;
  metadata?: MessageMetadata;
  icon?: string; // Optional icon/emoji for the author
  baseColor?: string; // Optional base color for message background tinting
  backend?: string; // Optional backend type (e.g., "claude_api", "gemini_cli")
  modelName?: string | null; // Optional model name (e.g., "claude-sonnet-4-5-20250929")
  attachments?: AttachedFile[]; // 添付ファイル情報
}

/**
 * LLM デバッグ情報
 */
export interface LlmDebugInfo {
  prompt: string;
  rawResponse: string;
  model?: string;
}

/**
 * メッセージメタデータ
 */
export interface MessageMetadata {
  command?: string;
  taskId?: string;
  agentId?: string;
  status?: 'pending' | 'running' | 'completed' | 'failed';
  llmDebugInfo?: LlmDebugInfo;
}

/**
 * メッセージスタイル設定
 */
export interface MessageStyle {
  backgroundColor: string;
  textColor: string;
  borderColor?: string;
  iconColor?: string;
  align: 'left' | 'center' | 'right';
  showAvatar: boolean;
  showBadge: boolean;
}

/**
 * メッセージタイプに応じたスタイルを取得
 */
export const getMessageStyle = (type: MessageType): MessageStyle => {
  switch (type) {
    case 'user':
      return {
        backgroundColor: '#e7f5ff',
        textColor: '#1971c2',
        align: 'right',
        showAvatar: true,
        showBadge: false,
      };
    case 'ai':
      return {
        backgroundColor: '#f8f9fa',
        textColor: '#212529',
        align: 'left',
        showAvatar: true,
        showBadge: false,
      };
    case 'system':
      return {
        backgroundColor: '#fff9db',
        textColor: '#e67700',
        borderColor: '#fab005',
        iconColor: '#fab005',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    case 'error':
      return {
        backgroundColor: '#ffe3e3',
        textColor: '#c92a2a',
        borderColor: '#fa5252',
        iconColor: '#fa5252',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    case 'command':
      return {
        backgroundColor: '#e9ecef',
        textColor: '#495057',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    case 'task':
      return {
        backgroundColor: '#d3f9d8',
        textColor: '#2b8a3e',
        borderColor: '#51cf66',
        iconColor: '#51cf66',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    case 'thinking':
      return {
        backgroundColor: '#e5dbff',
        textColor: '#7048e8',
        borderColor: '#9775fa',
        iconColor: '#9775fa',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    case 'shell_output':
      return {
        backgroundColor: '#2e2e2e',
        textColor: '#00ff00',
        borderColor: '#4a4a4a',
        iconColor: '#00ff00',
        align: 'left',
        showAvatar: false,
        showBadge: true,
      };
    default:
      return {
        backgroundColor: '#f8f9fa',
        textColor: '#212529',
        align: 'left',
        showAvatar: false,
        showBadge: false,
      };
  }
};
