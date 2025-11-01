/**
 * 接続ステータス
 */
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

/**
 * アプリケーションステータス（Idle/Awaiting/Thinking等）
 */
export type AppStatus = 'Idle' | 'Awaiting' | 'Thinking';

/**
 * システムステータス情報
 */
export interface StatusInfo {
  connection: ConnectionStatus;
  activeTasks: number;
  currentAgent: string;
  mode: AppStatus;
  lastUpdate: Date;
}

/**
 * デフォルトのステータス情報
 */
export const getDefaultStatus = (): StatusInfo => ({
  connection: 'connected',
  activeTasks: 0,
  currentAgent: 'idle',
  mode: 'Idle',
  lastUpdate: new Date(),
});
