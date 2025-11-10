import { Stack, ScrollArea, Group, Text, Box, ActionIcon, Tooltip, Badge, Switch } from '@mantine/core';
import { notifications } from '@mantine/notifications';
import { Task, getTaskIcon } from '../../types/task';
import { Session } from '../../types/session';
import { Workspace } from '../../types/workspace';
import { useState } from 'react';

interface TaskListProps {
  tasks: Task[];
  sessions?: Session[];
  workspaces?: Workspace[];
  currentWorkspaceId?: string;
  onTaskToggle?: (taskId: string) => void;
  onTaskDelete?: (taskId: string) => void;
  onRefresh?: () => void;
}

export function TaskList({ tasks, sessions, workspaces, currentWorkspaceId, onTaskDelete, onRefresh }: TaskListProps) {
  const [filterCurrentWorkspace, setFilterCurrentWorkspace] = useState(false);

  // Get workspace info for a task
  const getTaskWorkspace = (task: Task): Workspace | undefined => {
    if (!sessions || !workspaces) return undefined;
    const session = sessions.find(s => s.id === task.session_id);
    if (!session) return undefined;
    return workspaces.find(w => w.id === session.workspace_id);
  };

  // Filter tasks by workspace if enabled
  const filteredTasks = filterCurrentWorkspace && currentWorkspaceId
    ? tasks.filter(task => {
        const workspace = getTaskWorkspace(task);
        return workspace?.id === currentWorkspaceId;
      })
    : tasks;

  const activeTasks = filteredTasks.filter(t => t.status === 'Running' || t.status === 'Pending');
  const completedTasks = filteredTasks.filter(t => t.status === 'Completed');
  const failedTasks = filteredTasks.filter(t => t.status === 'Failed');

  const handleCopyTaskOutput = async (task: Task) => {
    try {
      let output = `# Task: ${task.title}\n\n`;
      output += `Status: ${task.status}\n`;
      output += `Steps executed: ${task.steps_executed}\n`;
      output += `Created: ${new Date(task.created_at).toLocaleString()}\n`;
      output += `Updated: ${new Date(task.updated_at).toLocaleString()}\n\n`;

      if (task.result) {
        output += `## Summary\n${task.result}\n\n`;
      }

      if (task.error) {
        output += `## Error\n${task.error}\n\n`;
      }

      if (task.execution_details?.context) {
        output += `## Execution Context\n`;
        const context = task.execution_details.context;
        for (const [key, value] of Object.entries(context)) {
          output += `\n### ${key}\n`;
          if (typeof value === 'string') {
            output += value + '\n';
          } else {
            output += JSON.stringify(value, null, 2) + '\n';
          }
        }
      }

      await navigator.clipboard.writeText(output);
      notifications.show({
        title: 'Copied!',
        message: 'Task output copied to clipboard',
        color: 'green',
        autoClose: 2000,
      });
    } catch (error) {
      console.error('Failed to copy task output:', error);
      notifications.show({
        title: 'Failed to Copy',
        message: String(error),
        color: 'red',
      });
    }
  };

  const renderTask = (task: Task) => {
    const workspace = getTaskWorkspace(task);

    return (
      <Tooltip
        label={workspace ? `Working Dir: ${workspace.rootPath}` : 'Workspace not found'}
        withArrow
        position="top"
      >
        <Group
          key={task.id}
          gap="sm"
          wrap="nowrap"
          p="xs"
          style={{
            borderRadius: '8px',
            backgroundColor: task.status === 'Completed' ? '#f1f3f5' : 'transparent',
            transition: 'background-color 0.15s ease',
            cursor: 'default',
          }}
        >
          {/* ステータスアイコン */}
          <Text size="lg">{getTaskIcon(task.status)}</Text>

          {/* タスク内容 */}
          <Box style={{ flex: 1, minWidth: 0 }}>
            <Text
              size="sm"
              truncate
              fw={task.status === 'Running' ? 600 : 400}
              style={{
                textDecoration: task.status === 'Completed' ? 'line-through' : 'none',
                color: task.status === 'Completed' ? '#868e96' : task.status === 'Failed' ? '#fa5252' : undefined,
              }}
            >
              {task.title}
            </Text>
            <Group gap="xs" mt={2}>
              {workspace && (
                <>
                  <Badge size="xs" variant="light" color="blue">
                    {workspace.name}
                  </Badge>
                  <Text size="xs" c="dimmed">•</Text>
                </>
              )}
              <Text size="xs" c="dimmed">
                {task.steps_executed} steps
              </Text>
              <Text size="xs" c="dimmed">•</Text>
              <Text size="xs" c="dimmed">
                {formatDate(task.updated_at)}
              </Text>
            </Group>
          </Box>

      {/* コピーボタン */}
      {(task.status === 'Completed' || task.status === 'Failed') && (
        <Tooltip label="Copy output" withArrow>
          <ActionIcon
            size="sm"
            variant="subtle"
            color="blue"
            onClick={(e) => {
              e.stopPropagation();
              handleCopyTaskOutput(task);
            }}
          >
            📋
          </ActionIcon>
        </Tooltip>
      )}

          {/* 削除ボタン */}
          <ActionIcon
            size="sm"
            variant="subtle"
            color="red"
            onClick={() => onTaskDelete?.(task.id)}
          >
            🗑️
          </ActionIcon>
        </Group>
      </Tooltip>
    );
  };

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Group justify="space-between" px="md" pt="md">
        <Text size="lg" fw={700}>
          Tasks
        </Text>
        <Group gap="xs">
          <Text size="sm" c="dimmed">
            {activeTasks.length} active
          </Text>
          <Tooltip label="Refresh tasks" withArrow>
            <ActionIcon
              color="blue"
              variant="light"
              onClick={onRefresh}
              size="xs"
            >
              🔄
            </ActionIcon>
          </Tooltip>
        </Group>
      </Group>

      {/* フィルタオプション */}
      {currentWorkspaceId && (
        <Box px="md">
          <Switch
            size="xs"
            label="Current Workspace Only"
            checked={filterCurrentWorkspace}
            onChange={(e) => setFilterCurrentWorkspace(e.currentTarget.checked)}
          />
        </Box>
      )}

      {/* タスクリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* アクティブタスク */}
          {activeTasks.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                ACTIVE
              </Text>
              <Stack gap={4}>
                {activeTasks.map(renderTask)}
              </Stack>
            </Box>
          )}

          {/* 失敗タスク */}
          {failedTasks.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                FAILED
              </Text>
              <Stack gap={4}>
                {failedTasks.map(renderTask)}
              </Stack>
            </Box>
          )}

          {/* 完了タスク */}
          {completedTasks.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                COMPLETED
              </Text>
              <Stack gap={4}>
                {completedTasks.map(renderTask)}
              </Stack>
            </Box>
          )}

          {/* 空の状態 */}
          {tasks.length === 0 && (
            <Box p="md" style={{ textAlign: 'center' }}>
              <Text size="sm" c="dimmed">
                No tasks yet
              </Text>
              <Text size="xs" c="dimmed" mt="xs">
                Click 🚀 on a message to execute it as a task
              </Text>
            </Box>
          )}
        </Stack>
      </ScrollArea>

      {/* フッター */}
      <Box px="md" pb="md">
        <Text size="xs" c="dimmed">
          {tasks.length} total tasks
        </Text>
      </Box>
    </Stack>
  );
}

// 日付フォーマット
function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return date.toLocaleDateString();
}
