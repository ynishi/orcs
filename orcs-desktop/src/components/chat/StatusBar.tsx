import { Paper, Group, Badge, Text, Divider, Tooltip } from '@mantine/core';
import { StatusInfo } from '../../types/status';
import { GitInfo } from '../../types/git';
import { getConversationModeOption, getTalkStyleOption } from '../../types/conversation';

interface StatusBarProps {
  status: StatusInfo;
  gitInfo?: GitInfo;
  participatingAgentsCount?: number;
  totalPersonas?: number;
  autoMode?: boolean;
  conversationMode?: string;
  talkStyle?: string | null;
  executionStrategy?: string;
}

export function StatusBar({ status, gitInfo, participatingAgentsCount = 0, totalPersonas = 0, autoMode = false, conversationMode = 'normal', talkStyle = null, executionStrategy = 'sequential' }: StatusBarProps) {
  // 接続状態に応じたバッジカラー
  const getConnectionColor = () => {
    switch (status.connection) {
      case 'connected':
        return 'green';
      case 'disconnected':
        return 'red';
      case 'connecting':
        return 'yellow';
      default:
        return 'gray';
    }
  };

  // 接続状態のアイコン
  const getConnectionIcon = () => {
    switch (status.connection) {
      case 'connected':
        return '●';
      case 'disconnected':
        return '○';
      case 'connecting':
        return '◐';
      default:
        return '○';
    }
  };

  // Execution Strategy の絵文字とTooltip
  const getExecutionStrategyDisplay = () => {
    switch (executionStrategy) {
      case 'broadcast':
        return { icon: '📢', label: 'Broadcast' };
      case 'sequential':
        return { icon: '➡️', label: 'Sequential' };
      case 'mentioned':
        return { icon: '👤', label: 'Mentioned (@mention to specify)' };
      default:
        return { icon: '➡️', label: 'Sequential' };
    }
  };

  return (
    <Paper p="xs" radius="md" withBorder style={{ backgroundColor: '#f8f9fa' }}>
      <Group gap="md" wrap="nowrap">
        {/* 接続状態 */}
        <Group gap={6} wrap="nowrap">
          <Text size="sm" c={getConnectionColor()} fw={700}>
            {getConnectionIcon()}
          </Text>
          <Text size="sm" fw={500}>
            {status.connection.charAt(0).toUpperCase() + status.connection.slice(1)}
          </Text>
        </Group>

        <Divider orientation="vertical" />

        {/* アクティブタスク */}
        <Group gap={6} wrap="nowrap">
          <Text size="sm" c="dimmed">
            Tasks:
          </Text>
          <Badge color={status.activeTasks > 0 ? 'blue' : 'gray'} size="sm" variant="filled">
            {status.activeTasks}
          </Badge>
        </Group>

        <Divider orientation="vertical" />

        {/* エージェント（参加中の人数/全体） */}
        <Group gap={6} wrap="nowrap">
          <Text size="sm" c="dimmed">
            Personas:
          </Text>
          <Badge color={participatingAgentsCount > 0 ? 'green' : 'gray'} size="sm" variant="filled">
            {participatingAgentsCount}/{totalPersonas}
          </Badge>
        </Group>

        <Divider orientation="vertical" />

        {/* ステータス（Idle/Awaiting/Thinking等） */}
        <Group gap={6} wrap="nowrap">
          <Text size="sm" c="dimmed">
            Status:
          </Text>
          <Badge
            color={
              status.mode === 'Idle' ? 'gray' :
              status.mode === 'Awaiting' ? 'yellow' :
              status.mode === 'Thinking' ? 'blue' : 'gray'
            }
            size="sm"
            variant="filled"
          >
            {status.mode}
          </Badge>
        </Group>

        {/* AUTOモード */}
        <Divider orientation="vertical" />
        <Tooltip label={autoMode ? 'AUTO: ON' : 'AUTO: OFF'} withArrow>
          <Text size="lg" c={autoMode ? 'green' : 'red'} fw={700} style={{ cursor: 'pointer' }}>
            ●
          </Text>
        </Tooltip>

        {/* Talk Style */}
        <Divider orientation="vertical" />
        <Tooltip label={talkStyle ? (getTalkStyleOption(talkStyle as any)?.label || talkStyle) : 'No Talk Style'} withArrow>
          <Text size="lg" style={{ cursor: 'pointer' }}>
            {talkStyle ? (getTalkStyleOption(talkStyle as any)?.icon || '💬') : '💬'}
          </Text>
        </Tooltip>

        {/* Execution Strategy */}
        <Divider orientation="vertical" />
        <Tooltip label={getExecutionStrategyDisplay().label} withArrow>
          <Text size="lg" style={{ cursor: 'pointer' }}>
            {getExecutionStrategyDisplay().icon}
          </Text>
        </Tooltip>

        {/* Conversation Mode (Response Style) */}
        <Divider orientation="vertical" />
        <Tooltip label={getConversationModeOption(conversationMode as any)?.label || conversationMode} withArrow>
          <Text size="lg" style={{ cursor: 'pointer' }}>
            {getConversationModeOption(conversationMode as any)?.icon || '💬'}
          </Text>
        </Tooltip>

        {/* Git リポジトリ情報 */}
        {gitInfo?.is_repo && (
          <>
            <Divider orientation="vertical" />
            <Group gap={6} wrap="nowrap">
              <Text size="sm" c="dimmed">
                🌿
              </Text>
              <Text size="sm" fw={500} style={{ fontFamily: 'monospace' }}>
                {gitInfo.repo_name || 'Unknown'}
              </Text>
              {gitInfo.branch && (
                <>
                  <Text size="sm" c="dimmed">
                    @
                  </Text>
                  <Badge color="blue" size="sm" variant="light">
                    {gitInfo.branch}
                  </Badge>
                </>
              )}
            </Group>
          </>
        )}
      </Group>
    </Paper>
  );
}
