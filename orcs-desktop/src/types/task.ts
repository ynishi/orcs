// Re-export generated types
export type { TaskType, TaskStatus } from '../bindings/generated';
import type { TaskType, TaskStatus } from '../bindings/generated';

/**
 * タスク実行進捗情報（リアルタイム更新用）
 */
export interface TaskProgress {
  taskId: string; // was task_id
  currentWave?: number; // was current_wave
  currentStep?: string; // was current_step
  currentAgent?: string; // was current_agent
  lastMessage?: string; // was last_message
  lastUpdated: number; // was last_updated
}

/**
 * タスク実行履歴
 * Extends TaskType from generated schema with additional frontend-specific fields
 */
export interface Task extends TaskType {
  // TaskType already has: id, sessionId, title, description, status, createdAt, updatedAt,
  // completedAt, stepsExecuted, stepsSkipped, contextKeys, error, result

  // Additional fields from full domain model (not in TaskType):
  executionDetails?: ExecutionDetails;
  strategy?: string;
  journalLog?: string; // was journal_log
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
