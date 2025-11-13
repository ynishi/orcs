import { notifications } from '@mantine/notifications';
import { MessageType } from '../types/message';

/**
 * # System Message Display Guidelines
 *
 * ORCS provides three ways to display messages to the user. Choose based on:
 * 1. **Importance**: Should this be part of conversation history?
 * 2. **Context**: Does this provide conversational context for AI agents?
 * 3. **Persistence**: Should this survive session switches?
 *
 * ## Method 1: `handleSystemMessage()` with CONVERSATION_EVENT
 *
 * **When to use:**
 * - Important events that should be part of conversation history
 * - Events that provide context for AI agents
 * - User-initiated actions with permanent effects
 *
 * **Examples:**
 * - ✅ Expert persona created/saved
 * - ✅ Participant joined/left
 * - ✅ Execution strategy changed
 * - ✅ Command execution results
 * - ✅ Error messages (failures that affect conversation)
 *
 * **Persistence:** ⚠️ MESSAGE IS ADDED TO UI BUT NOT AUTOMATICALLY PERSISTED
 *
 * **To persist to backend:**
 * ```typescript
 * // Add to UI
 * handleSystemMessage(conversationMessage('...'), addMessage);
 *
 * // THEN explicitly persist
 * await invoke('append_system_messages', {
 *   messages: [{ content: '...', messageType: 'info', severity: 'info' }]
 * });
 * ```
 *
 * ## Method 2: `invoke('append_system_messages')`
 *
 * **When to use:**
 * - When you need GUARANTEED persistence to session history
 * - Important events that must survive app restarts
 * - Events that should be visible after session switching
 *
 * **Examples:**
 * - ✅ Expert persona creation (with details)
 * - ✅ Task execution results
 * - ✅ Critical errors or warnings
 *
 * **Persistence:** ✅ SAVED TO SESSION FILE, SURVIVES RESTARTS
 *
 * **Note:** This ONLY persists to backend. You must also call handleSystemMessage()
 * if you want immediate UI feedback.
 *
 * ## Method 3: `notifications.show()` (Toast)
 *
 * **When to use:**
 * - Temporary feedback that doesn't need conversation context
 * - Progress indicators
 * - Confirmations for UI-only operations
 *
 * **Examples:**
 * - ✅ "Creating expert..." (progress)
 * - ✅ "Session switched" (transient feedback)
 * - ✅ "File uploaded" (confirmation)
 * - ✅ "Copied to clipboard"
 * - ❌ Expert persona created (use CONVERSATION_EVENT instead)
 *
 * **Persistence:** ❌ DISAPPEARS ON NEXT RENDER, NOT PERSISTED
 *
 * ## Decision Tree
 *
 * ```
 * Should this be conversation context?
 * ├─ YES → Will AI agents need this info?
 * │   ├─ YES → append_system_messages + handleSystemMessage
 * │   └─ NO  → handleSystemMessage only
 * └─ NO  → Is this temporary progress/feedback?
 *     ├─ YES → notifications.show()
 *     └─ NO  → Re-evaluate if it should be conversation context
 * ```
 *
 * ## Common Patterns
 *
 * **Pattern 1: Important persisted event**
 * ```typescript
 * // Progress toast (temporary)
 * notifications.show({ id: 'op', message: 'Processing...', autoClose: false });
 *
 * // Persist result to session
 * await invoke('append_system_messages', {
 *   messages: [{ content: '✅ Operation completed', ... }]
 * });
 * notifications.hide('op');
 * ```
 *
 * **Pattern 2: Transient feedback only**
 * ```typescript
 * notifications.show({ title: 'Saved', message: 'Changes saved', color: 'green' });
 * ```
 *
 * **Pattern 3: Immediate chat message (not persisted)**
 * ```typescript
 * handleSystemMessage(conversationMessage('Strategy changed'), addMessage);
 * await saveCurrentSession(); // Saves via normal session save
 * ```
 */

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
   * Conversation context events (Chat only, persisted via session save)
   * Examples: Persona join/leave, Strategy changes, Command execution
   *
   * ⚠️ Note: This adds message to UI but does NOT automatically persist to backend.
   * For guaranteed persistence, use invoke('append_system_messages') after calling this.
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
  /** Message type (for CONVERSATION_EVENT, determines chat display style) */
  messageType?: MessageType;
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
  type: MessageType,
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
 * ⚠️ INTERNAL USE ONLY - DO NOT EXPORT OR USE DIRECTLY
 * Use handleAndPersistSystemMessage() instead for conversation events that need persistence.
 * This function is internal to systemMessage module and should not be used outside.
 *
 * @param msg - System message to handle
 * @param onMessage - Optional callback for chat messages (required for CONVERSATION_EVENT)
 */
function handleSystemMessage(
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
        // Use explicit messageType if provided, otherwise infer from severity
        const type = msg.messageType || (msg.severity === 'error' ? 'error' : 'system');
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
 * Helper: Create transient feedback message (Toast only, not persisted)
 *
 * Use this for:
 * - Temporary progress updates
 * - UI operation confirmations
 * - Non-conversational feedback
 *
 * @example
 * handleSystemMessage(transientMessage('File saved successfully'), addMessage);
 * // Shows toast, does NOT add to chat, does NOT persist
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
 * Helper: Create conversation event message (Chat display, needs explicit persistence)
 *
 * ⚠️ IMPORTANT: This adds message to UI chat but does NOT persist to backend automatically.
 *
 * For persistence, use BOTH:
 * ```typescript
 * // 1. Show in chat immediately
 * handleSystemMessage(conversationMessage('Event happened'), addMessage);
 *
 * // 2. Persist to session file
 * await invoke('append_system_messages', {
 *   messages: [{ content: 'Event happened', messageType: 'info', severity: 'info' }]
 * });
 * ```
 *
 * Use this for:
 * - Important conversation context
 * - Events AI agents should know about
 * - User-initiated actions
 *
 * @example
 * // Non-persisted (lost on session reload)
 * handleSystemMessage(conversationMessage('Strategy changed to broadcast'), addMessage);
 *
 * @example
 * // Persisted (survives session reload)
 * await invoke('append_system_messages', {
 *   messages: [{ content: '🔶 Expert created', messageType: 'info', severity: 'info' }]
 * });
 */
export function conversationMessage(
  message: string,
  severity: MessageSeverity = 'info',
  icon?: string,
  messageType?: MessageType
): SystemMessage {
  return {
    category: MessageCategory.CONVERSATION_EVENT,
    message,
    severity,
    icon,
    messageType,
  };
}

/**
 * Helper: Create command message (displayed with "COMMAND" label)
 */
export function commandMessage(message: string): SystemMessage {
  return {
    category: MessageCategory.CONVERSATION_EVENT,
    message,
    severity: 'info',
    messageType: 'command',
  };
}

/**
 * Helper: Create shell output message (displayed with monospace font)
 */
export function shellOutputMessage(message: string): SystemMessage {
  return {
    category: MessageCategory.CONVERSATION_EVENT,
    message,
    severity: 'info',
    messageType: 'shell_output',
  };
}

/**
 * Helper: Create task message (displayed with task styling)
 */
export function taskMessage(message: string): SystemMessage {
  return {
    category: MessageCategory.CONVERSATION_EVENT,
    message,
    severity: 'info',
    messageType: 'task',
  };
}

/**
 * Handles system message with both immediate UI display AND backend persistence
 *
 * This is the recommended method for important conversation events that should:
 * 1. Display immediately in chat UI
 * 2. Persist to session file (survives reloads and session switches)
 *
 * @param msg - System message to display and persist
 * @param onMessage - Callback for adding message to chat UI
 * @param invokeFunc - Tauri invoke function for backend persistence
 *
 * @example
 * ```typescript
 * await handleAndPersistSystemMessage(
 *   conversationMessage('Expert created: Film Production Specialist', 'info'),
 *   addMessage,
 *   invoke
 * );
 * ```
 */
export async function handleAndPersistSystemMessage(
  msg: SystemMessage,
  onMessage: OnMessageCallback,
  invokeFunc: typeof import('@tauri-apps/api/core').invoke
): Promise<void> {
  // 1. Display immediately in UI
  handleSystemMessage(msg, onMessage);

  // 2. Persist to backend
  if (msg.category === MessageCategory.CONVERSATION_EVENT) {
    const messageType = msg.messageType || (msg.severity === 'error' ? 'error' : 'info');
    const prefix = msg.icon ? `${msg.icon} ` : '';
    const content = `${prefix}${msg.message}`;

    await invokeFunc('append_system_messages', {
      messages: [
        {
          content,
          messageType,
          severity: msg.severity,
        },
      ],
    });
  }
}
