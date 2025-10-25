import { Stack, ScrollArea, Group, Text, Box, Checkbox, ActionIcon } from '@mantine/core';
import { Task, getTaskColor } from '../../types/task';

interface TaskListProps {
  tasks: Task[];
  onTaskToggle?: (taskId: string) => void;
  onTaskDelete?: (taskId: string) => void;
}

export function TaskList({ tasks, onTaskToggle, onTaskDelete }: TaskListProps) {
  const pendingTasks = tasks.filter(t => t.status !== 'completed');
  const completedTasks = tasks.filter(t => t.status === 'completed');

  const renderTask = (task: Task) => (
    <Group
      key={task.id}
      gap="sm"
      wrap="nowrap"
      p="xs"
      style={{
        borderRadius: '8px',
        backgroundColor: task.status === 'completed' ? '#f1f3f5' : 'transparent',
        transition: 'background-color 0.15s ease',
      }}
    >
      {/* チェックボックス */}
      <Checkbox
        checked={task.status === 'completed'}
        onChange={() => onTaskToggle?.(task.id)}
        size="sm"
        color={getTaskColor(task.status)}
      />

      {/* タスク内容 */}
      <Box style={{ flex: 1, minWidth: 0 }}>
        <Text
          size="sm"
          truncate
          style={{
            textDecoration: task.status === 'completed' ? 'line-through' : 'none',
            color: task.status === 'completed' ? '#868e96' : undefined,
          }}
        >
          {task.description}
        </Text>
        <Text size="xs" c="dimmed">
          {task.createdAt.toLocaleTimeString()}
        </Text>
      </Box>

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
  );

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Group justify="space-between" px="md" pt="md">
        <Text size="lg" fw={700}>
          Tasks
        </Text>
        <Text size="sm" c="dimmed">
          {pendingTasks.length} active
        </Text>
      </Group>

      {/* タスクリスト */}
      <ScrollArea style={{ flex: 1 }} px="sm">
        <Stack gap="md">
          {/* アクティブタスク */}
          {pendingTasks.length > 0 && (
            <Box>
              <Text size="xs" fw={600} c="dimmed" mb="xs" px="xs">
                ACTIVE
              </Text>
              <Stack gap={4}>
                {pendingTasks.map(renderTask)}
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
                Use /task to create a new task
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
