export type SessionEvent =
  | {
      type: 'user_input';
      content: string;
      attachments?: string[];
    }
  | {
      type: 'system_event';
      content: string;
      messageType?: string;
      severity?: 'info' | 'warning' | 'critical';
    }
  | {
      type: 'moderator_action';
      action: ModeratorActionEvent;
    };

export type ModeratorActionEvent =
  | {
      kind: 'set_conversation_mode';
      mode: string;
    }
  | {
      kind: 'append_system_message';
      content: string;
      messageType?: string;
      severity?: 'info' | 'warning' | 'critical';
    };
