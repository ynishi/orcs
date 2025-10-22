/**
 * エージェントのステータス
 */
export type AgentStatus = 'idle' | 'running' | 'error' | 'offline';

/**
 * エージェント
 */
export interface Agent {
  id: string;
  name: string;
  status: AgentStatus;
  description: string;
  lastActive?: Date;
  isActive: boolean; // 議論に参加しているかどうか
}

/**
 * エージェントステータスに応じたアイコンを取得
 */
export function getAgentIcon(status: AgentStatus): string {
  switch (status) {
    case 'idle':
      return '⚪';
    case 'running':
      return '🟢';
    case 'error':
      return '🔴';
    case 'offline':
      return '⚫';
    default:
      return '⚪';
  }
}

/**
 * エージェントステータスに応じたカラーを取得
 */
export function getAgentColor(status: AgentStatus): string {
  switch (status) {
    case 'idle':
      return 'gray';
    case 'running':
      return 'green';
    case 'error':
      return 'red';
    case 'offline':
      return 'dark';
    default:
      return 'gray';
  }
}
