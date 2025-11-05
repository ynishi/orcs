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
 * ペルソナ情報（バックエンドから取得）
 */
export interface PersonaInfo {
  id: string;
  name: string;
  role: string;
  background: string;
}

/**
 * ペルソナ設定(バックエンドのPersonaConfigに対応)
 */
export type PersonaBackend = 'claude_cli' | 'claude_api' | 'gemini_cli' | 'gemini_api' | 'open_ai_api' | 'codex_cli';

export interface PersonaConfig {
  id: string;
  name: string;
  role: string;
  background: string;
  communication_style: string;
  default_participant: boolean;
  source: 'System' | 'User';
  backend: PersonaBackend;
  model_name?: string;
  icon?: string;
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
