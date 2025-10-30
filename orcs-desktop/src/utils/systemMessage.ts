import { notifications } from '@mantine/notifications';

/**
 * System message categories based on persistence and context requirements
 */
export enum MessageCategory {
  /**
   * Transient UI feedback (Toast only, not persisted)
   * Examples: File operations, Session/Workspace CRUD
   */
  TRANSIENT_FEEDBACK = 'transient_feedback',

  /**
   * Conversation context events (Chat only, persisted)
   * Examples: Persona join/leave, Strategy changes, Command execution
   */
  CONVERSATION_EVENT = 'conversation_event',
}

/**
 * Message severity levels
 */
export type MessageSeverity = 'info' | 'success' | 'warning' | 'error';

/**
 * System message structure
 */
export interface SystemMessage {
  /** Message category determines display method */
  category: MessageCategory;
  /** Title for toast notifications */
  title?: string;
  /** Message content */
  message: string;
  /** Severity level (determines color/icon) */
  severity: MessageSeverity;
  /** Optional custom icon (emoji or icon name) */
  icon?: string;
}

/**
 * Callback type for adding messages to chat
 */
export type OnMessageCallback = (
  type: 'system' | 'error',
  author: string,
  text: string
) => void;

/**
 * Severity to color mapping for toast notifications
 */
const SEVERITY_COLORS: Record<MessageSeverity, string> = {
  info: 'blue',
  success: 'green',
  warning: 'yellow',
  error: 'red',
};

/**
 * Default icons for each severity
 */
const DEFAULT_ICONS: Record<MessageSeverity, string> = {
  info: 'ℹ️',
  success: '✅',
  warning: '⚠️',
  error: '❌',
};

/**
 * Handles system messages according to their category
 *
 * @param msg - System message to handle
 * @param onMessage - Optional callback for chat messages (required for CONVERSATION_EVENT)
 */
export function handleSystemMessage(
  msg: SystemMessage,
  onMessage?: OnMessageCallback
): void {
  switch (msg.category) {
    case MessageCategory.TRANSIENT_FEEDBACK:
      // Show toast notification only
      notifications.show({
        title: msg.title || severityToTitle(msg.severity),
        message: msg.message,
        color: SEVERITY_COLORS[msg.severity],
        icon: msg.icon || DEFAULT_ICONS[msg.severity],
      });
      break;

    case MessageCategory.CONVERSATION_EVENT:
      // Add to chat conversation (persisted)
      if (onMessage) {
        const type = msg.severity === 'error' ? 'error' : 'system';
        const prefix = msg.icon ? `${msg.icon} ` : '';
        onMessage(type, 'SYSTEM', `${prefix}${msg.message}`);
      } else {
        console.warn(
          '[SystemMessage] CONVERSATION_EVENT requires onMessage callback',
          msg
        );
      }
      break;

    default:
      console.error('[SystemMessage] Unknown message category:', msg);
  }
}

/**
 * Convert severity to default title
 */
function severityToTitle(severity: MessageSeverity): string {
  switch (severity) {
    case 'info':
      return 'Information';
    case 'success':
      return 'Success';
    case 'warning':
      return 'Warning';
    case 'error':
      return 'Error';
  }
}

/**
 * Helper: Create transient feedback message
 */
export function transientMessage(
  message: string,
  severity: MessageSeverity = 'success',
  options?: { title?: string; icon?: string }
): SystemMessage {
  return {
    category: MessageCategory.TRANSIENT_FEEDBACK,
    message,
    severity,
    title: options?.title,
    icon: options?.icon,
  };
}

/**
 * Helper: Create conversation event message
 */
export function conversationMessage(
  message: string,
  severity: MessageSeverity = 'info',
  icon?: string
): SystemMessage {
  return {
    category: MessageCategory.CONVERSATION_EVENT,
    message,
    severity,
    icon,
  };
}
