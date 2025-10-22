/**
 * タスクのステータス
 */
export type TaskStatus = 'pending' | 'in_progress' | 'completed';

/**
 * タスク
 */
export interface Task {
  id: string;
  description: string;
  status: TaskStatus;
  createdAt: Date;
}

/**
 * タスクステータスに応じたアイコンを取得
 */
export function getTaskIcon(status: TaskStatus): string {
  switch (status) {
    case 'pending':
      return '⬜';
    case 'in_progress':
      return '🔄';
    case 'completed':
      return '✅';
    default:
      return '⬜';
  }
}

/**
 * タスクステータスに応じたカラーを取得
 */
export function getTaskColor(status: TaskStatus): string {
  switch (status) {
    case 'pending':
      return 'gray';
    case 'in_progress':
      return 'blue';
    case 'completed':
      return 'green';
    default:
      return 'gray';
  }
}
