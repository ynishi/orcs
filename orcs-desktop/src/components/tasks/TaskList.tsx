import { Stack, ScrollArea, Group, Text, Box, ActionIcon, Tooltip, Badge, Switch } from '@mantine/core';
import { IconDeviceFloppy, IconFolder, IconClipboard, IconChartBar, IconFileText, IconTrash } from '@tabler/icons-react';
import { notifications } from '@mantine/notifications';
import { Task, TaskProgress } from '../../types/task';
import { Session } from '../../types/session';
import { Workspace } from '../../types/workspace';
import { useState } from 'react';

interface TaskListProps {
  tasks: Task[];
  taskProgress?: Map<string, TaskProgress>;
  sessions?: Session[];
  workspaces?: Workspace[];
  currentWorkspaceId?: string;
  onTaskToggle?: (taskId: string) => void;
  onTaskDelete?: (taskId: string) => void;
  onSaveToWorkspace?: (task: Task) => Promise<void>;
  onRefresh?: () => void;
}

export function TaskList({ tasks, taskProgress, sessions, workspaces, currentWorkspaceId, onTaskDelete, onSaveToWorkspace }: TaskListProps) {
  const [filterCurrentWorkspace, setFilterCurrentWorkspace] = useState(false);

  // Get workspace info for a task
  const getTaskWorkspace = (task: Task): Workspace | undefined => {
    if (!sessions || !workspaces) return undefined;
    const session = sessions.find(s => s.id === task.sessionId);
    if (!session) return undefined;
    return workspaces.find(w => w.id === session.workspaceId);
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
      output += `Steps executed: ${task.stepsExecuted}\n`;
      output += `Created: ${new Date(task.createdAt).toLocaleString()}\n`;
      output += `Updated: ${new Date(task.updatedAt).toLocaleString()}\n\n`;

      if (task.result) {
        output += `## Summary\n${task.result}\n\n`;
      }

      if (task.error) {
        output += `## Error\n${task.error}\n\n`;
      }

      if (task.executionDetails?.context) {
        output += `## Execution Context\n`;
        const context = task.executionDetails.context;
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

  const handleDownloadStrategy = (task: Task) => {
    if (!task.strategy) return;

    try {
      const blob = new Blob([task.strategy], { type: 'application/json' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `task-${task.id}-strategy.json`;
      link.click();
      window.URL.revokeObjectURL(url);

      notifications.show({
        title: 'Downloaded!',
        message: 'Strategy downloaded successfully',
        color: 'green',
        autoClose: 2000,
      });
    } catch (error) {
      console.error('Failed to download strategy:', error);
      notifications.show({
        title: 'Failed to Download',
        message: String(error),
        color: 'red',
      });
    }
  };

  const handleDownloadJournalLog = (task: Task) => {
    if (!task.journalLog) return;

    try {
      const blob = new Blob([task.journalLog], { type: 'text/plain' });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `task-${task.id}-journal.log`;
      link.click();
      window.URL.revokeObjectURL(url);

      notifications.show({
        title: 'Downloaded!',
        message: 'Journal log downloaded successfully',
        color: 'green',
        autoClose: 2000,
      });
    } catch (error) {
      console.error('Failed to download journal log:', error);
      notifications.show({
        title: 'Failed to Download',
        message: String(error),
        color: 'red',
      });
    }
  };

  const renderTask = (task: Task) => {
    const workspace = getTaskWorkspace(task);
    const progress = taskProgress?.get(task.id);

    return (
      <Box
        key={task.id}
        style={{
          borderRadius: '8px',
          border: '1px solid var(--mantine-color-gray-3)',
          backgroundColor: 'white',
          transition: 'all 0.15s ease',
          overflow: 'hidden',
        }}
      >
        {/* ヘッダー（アクションボタン） */}
        <Group
          gap="xs"
          px="md"
          py="xs"
          justify="flex-end"
          style={{
            backgroundColor: task.status === 'Completed' ? '#f8f9fa' : task.status === 'Failed' ? '#fff5f5' : '#f8f9fa',
            borderBottom: '1px solid var(--mantine-color-gray-3)',
          }}
        >
          {/* Working Dir Tooltip */}
          {workspace && (
            <Tooltip label={`Working Dir: ${workspace.rootPath}`} withArrow>
              <Box style={{ marginRight: 'auto', display: 'flex', alignItems: 'center' }}>
                <IconFolder size={14} color="gray" />
              </Box>
            </Tooltip>
          )}

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
                <IconClipboard size={16} />
              </ActionIcon>
            </Tooltip>
          )}

          {/* Strategyダウンロードボタン */}
          {task.strategy && (
            <Tooltip label="Download Strategy" withArrow>
              <ActionIcon
                size="sm"
                variant="subtle"
                color="violet"
                onClick={(e) => {
                  e.stopPropagation();
                  handleDownloadStrategy(task);
                }}
              >
                <IconChartBar size={16} />
              </ActionIcon>
            </Tooltip>
          )}

          {/* JournalLogダウンロードボタン */}
          {task.journalLog && (
            <Tooltip label="Download Journal Log" withArrow>
              <ActionIcon
                size="sm"
                variant="subtle"
                color="grape"
                onClick={(e) => {
                  e.stopPropagation();
                  handleDownloadJournalLog(task);
                }}
              >
                <IconFileText size={16} />
              </ActionIcon>
            </Tooltip>
          )}

          {/* Workspaceに保存ボタン */}
          {onSaveToWorkspace && (
            <Tooltip label="Save to Workspace" withArrow>
              <ActionIcon
                size="sm"
                variant="subtle"
                color="blue"
                onClick={() => onSaveToWorkspace(task)}
              >
                <IconDeviceFloppy size={16} />
              </ActionIcon>
            </Tooltip>
          )}

          {/* 削除ボタン */}
          <Tooltip label="Delete" withArrow>
            <ActionIcon
              size="sm"
              variant="subtle"
              color="red"
              onClick={() => onTaskDelete?.(task.id)}
            >
              <IconTrash size={16} />
            </ActionIcon>
          </Tooltip>
        </Group>

        {/* コンテンツエリア */}
        <Box p="md">
          <Group gap="sm" wrap="nowrap">
            {/* タスク内容 */}
            <Box style={{ flex: 1, minWidth: 0 }}>
              <Text
                size="sm"
                fw={task.status === 'Running' || task.status === 'Pending' ? 600 : 400}
                style={{
                  textDecoration: task.status === 'Completed' ? 'line-through' : 'none',
                  color: task.status === 'Completed' ? '#868e96' : task.status === 'Failed' ? '#fa5252' : undefined,
                }}
              >
                {task.title}
              </Text>

              {/* Planning状態の表示 */}
              {task.status === 'Pending' && (
                <Box mt={4} p={4} style={{ backgroundColor: '#e3fafc', borderRadius: '4px' }}>
                  <Text size="xs" c="cyan" fw={500}>
                    Generating execution strategy...
                  </Text>
                </Box>
              )}

              {/* リアルタイム進捗表示 (Running中のみ) */}
              {true && progress && (
                <Box mt={4} p={4} style={{ backgroundColor: '#e7f5ff', borderRadius: '4px' }}>
                  <Stack gap={2}>
                    {progress.currentWave !== undefined && (
                      <Text size="xs" c="blue" fw={500}>
                        Wave {progress.currentWave}
                      </Text>
                    )}
                    {progress.currentStep && (
                      <Text size="xs" c="dimmed">
                        Step: {progress.currentStep}
                      </Text>
                    )}
                    {progress.currentAgent && (
                      <Text size="xs" c="dimmed">
                        Agent: {progress.currentAgent}
                      </Text>
                    )}
                    {progress.lastMessage && (
                      <Text size="xs" c="dimmed" lineClamp={1}>
                        {progress.lastMessage}
                      </Text>
                    )}
                  </Stack>
                </Box>
              )}

              <Group gap="xs" mt={4}>
                {/* Status Badge for active tasks */}
                {(task.status === 'Pending' || task.status === 'Running') && (
                  <>
                    <Badge
                      size="xs"
                      variant="dot"
                      color={
                        task.status === 'Pending' ? 'gray' :
                        'blue'
                      }
                    >
                      {task.status}
                    </Badge>
                    {workspace && <Text size="xs" c="dimmed">•</Text>}
                  </>
                )}

                {workspace && (
                  <>
                    <Badge size="xs" variant="light" color="blue">
                      {workspace.name}
                    </Badge>
                    <Text size="xs" c="dimmed">•</Text>
                  </>
                )}

                {/* Progress display: show live progress for running tasks */}
                {task.status === 'Running' && progress?.currentWave !== undefined ? (
                  <Text size="xs" c="blue" fw={500}>
                    Wave {progress.currentWave}
                  </Text>
                ) : (
                  <Text size="xs" c="dimmed">
                    {task.stepsExecuted} {task.stepsExecuted === 1 ? 'step' : 'steps'}
                  </Text>
                )}

                <Text size="xs" c="dimmed">•</Text>
                <Text size="xs" c="dimmed">
                  {formatDate(task.updatedAt)}
                </Text>
              </Group>
            </Box>
          </Group>
        </Box>
      </Box>
    );
  };

  return (
    <Stack gap="md" h="100%">
      {/* ヘッダー */}
      <Group justify="space-between" px="md" pt="md">
        <Group gap="xs">
          <Text size="lg" fw={700}>
            Tasks
          </Text>
          <Badge size="xs" variant="light" color="orange">
            BETA
          </Badge>
          <Tooltip label="Tasks use a fixed model and may be slower" withArrow>
            <Text size="xs" c="dimmed" style={{ cursor: 'help' }}>
              ⓘ
            </Text>
          </Tooltip>
        </Group>
        <Text size="sm" c="dimmed">
          {activeTasks.length} active
        </Text>
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
              <Stack gap="xs">
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
              <Stack gap="xs">
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
              <Stack gap="xs">
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
                Click on a message to execute it as a task
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
