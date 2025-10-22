/**
 * 接続ステータス
 */
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

/**
 * システムステータス情報
 */
export interface StatusInfo {
  connection: ConnectionStatus;
  activeTasks: number;
  currentAgent: string;
  mode: string;
  lastUpdate: Date;
}

/**
 * デフォルトのステータス情報
 */
export const getDefaultStatus = (): StatusInfo => ({
  connection: 'connected',
  activeTasks: 0,
  currentAgent: 'idle',
  mode: 'Chat',
  lastUpdate: new Date(),
});
