/**
 * タスクのステータス
 */
export type TaskStatus = 'Pending' | 'Running' | 'Completed' | 'Failed';

/**
 * タスク実行履歴
 */
export interface Task {
  id: string;
  session_id: string;
  title: string;
  description: string;
  status: TaskStatus;
  created_at: string;
  updated_at: string;
  completed_at?: string;
  steps_executed: number;
  steps_skipped: number;
  context_keys: number;
  error?: string;
  result?: string;
  execution_details?: ExecutionDetails;
  strategy?: string;
  journal_log?: string;
}

/**
 * Step情報
 */
export interface StepInfo {
  id: string;
  description: string;
  status: StepStatus;
  agent: string;
  output?: any;
  error?: string;
}

/**
 * Stepのステータス
 */
export type StepStatus = 'Pending' | 'Running' | 'Completed' | 'Skipped' | 'Failed';

/**
 * 実行詳細
 */
export interface ExecutionDetails {
  steps: StepInfo[];
  context: Record<string, any>;
}

/**
 * タスクステータスに応じたアイコンを取得
 */
export function getTaskIcon(status: TaskStatus): string {
  switch (status) {
    case 'Pending':
      return '⬜';
    case 'Running':
      return '🔄';
    case 'Completed':
      return '✅';
    case 'Failed':
      return '❌';
    default:
      return '⬜';
  }
}

/**
 * タスクステータスに応じたカラーを取得
 */
export function getTaskColor(status: TaskStatus): string {
  switch (status) {
    case 'Pending':
      return 'gray';
    case 'Running':
      return 'blue';
    case 'Completed':
      return 'green';
    case 'Failed':
      return 'red';
    default:
      return 'gray';
  }
}
