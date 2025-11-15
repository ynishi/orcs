import { Paper, Group, Badge, Text, Divider, Tooltip, Menu, Checkbox, ScrollArea } from '@mantine/core';
import { StatusInfo } from '../../types/status';
import { GitInfo } from '../../types/git';
import { PersonaConfig } from '../../types/agent';
import { DEFAULT_STYLE_ICON, DEFAULT_STYLE_LABEL, getConversationModeOption, getTalkStyleOption, TALK_STYLES, EXECUTION_STRATEGIES, CONVERSATION_MODES } from '../../types/conversation';

interface StatusBarProps {
  status: StatusInfo;
  gitInfo?: GitInfo;
  participatingAgentsCount?: number;
  totalPersonas?: number;
  autoMode?: boolean;
  conversationMode?: string;
  talkStyle?: string | null;
  executionStrategy?: string;
  personas?: PersonaConfig[];
  activeParticipantIds?: string[];
  onTalkStyleChange?: (style: string | null) => void;
  onExecutionStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onToggleParticipant?: (personaId: string, isChecked: boolean) => void;
}

export function StatusBar({ status, gitInfo, participatingAgentsCount = 0, totalPersonas = 0, autoMode = false, conversationMode = 'normal', talkStyle = null, executionStrategy = 'sequential', personas = [], activeParticipantIds = [], onTalkStyleChange, onExecutionStrategyChange, onConversationModeChange, onToggleParticipant }: StatusBarProps) {
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
          <Menu position="top" withArrow closeOnItemClick={false}>
            <Menu.Target>
              <Badge
                color={participatingAgentsCount > 0 ? 'green' : 'gray'}
                size="sm"
                variant="filled"
                style={{ cursor: 'pointer' }}
              >
                {participatingAgentsCount}/{totalPersonas}
              </Badge>
            </Menu.Target>
            <Menu.Dropdown>
              <ScrollArea h={Math.min(personas.length * 40 + 20, 300)}>
                {personas.map((persona) => {
                  const isActive = activeParticipantIds.includes(persona.id);
                  return (
                    <Menu.Item
                      key={persona.id}
                      onClick={(e) => {
                        e.preventDefault();
                        onToggleParticipant?.(persona.id, !isActive);
                      }}
                    >
                      <Group gap="xs" wrap="nowrap">
                        <Checkbox
                          checked={isActive}
                          onChange={() => {}}
                          onClick={(e) => e.stopPropagation()}
                          size="sm"
                        />
                        <Text size="sm" truncate style={{ flex: 1 }}>
                          {persona.icon || '👤'} {persona.name} - {persona.role.length > 20 ? persona.role.substring(0, 20) + '...' : persona.role}
                        </Text>
                      </Group>
                    </Menu.Item>
                  );
                })}
              </ScrollArea>
            </Menu.Dropdown>
          </Menu>
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
        <Menu position="top" withArrow>
          <Menu.Target>
            <Tooltip label={talkStyle ? (getTalkStyleOption(talkStyle as any)?.label || talkStyle) : DEFAULT_STYLE_LABEL} withArrow>
              <Text size="lg" style={{ cursor: 'pointer' }}>
                {talkStyle ? (getTalkStyleOption(talkStyle as any)?.icon || DEFAULT_STYLE_ICON) : DEFAULT_STYLE_ICON}
              </Text>
            </Tooltip>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item key={DEFAULT_STYLE_LABEL} onClick={() => onTalkStyleChange?.(null)}>
              {DEFAULT_STYLE_ICON} {DEFAULT_STYLE_LABEL}
            </Menu.Item>
            {TALK_STYLES.map((style) => (
              <Menu.Item key={style.value} onClick={() => onTalkStyleChange?.(style.value)}>
                {style.icon} {style.label}
              </Menu.Item>
            ))}
          </Menu.Dropdown>
        </Menu>

        {/* Execution Strategy */}
        <Divider orientation="vertical" />
        <Menu position="top" withArrow>
          <Menu.Target>
            <Tooltip label={getExecutionStrategyDisplay().label} withArrow>
              <Text size="lg" style={{ cursor: 'pointer' }}>
                {getExecutionStrategyDisplay().icon}
              </Text>
            </Tooltip>
          </Menu.Target>
          <Menu.Dropdown>
            {EXECUTION_STRATEGIES.map((strategy) => (
              <Menu.Item key={strategy.value} onClick={() => onExecutionStrategyChange?.(strategy.value)}>
                {strategy.icon} {strategy.label}
              </Menu.Item>
            ))}
          </Menu.Dropdown>
        </Menu>

        {/* Conversation Mode (Response Style) */}
        <Divider orientation="vertical" />
        <Menu position="top" withArrow>
          <Menu.Target>
            <Tooltip label={getConversationModeOption(conversationMode as any)?.label || conversationMode} withArrow>
              <Text size="lg" style={{ cursor: 'pointer' }}>
                {getConversationModeOption(conversationMode as any)?.icon || '🗨️'}
              </Text>
            </Tooltip>
          </Menu.Target>
          <Menu.Dropdown>
            {CONVERSATION_MODES.map((mode) => (
              <Menu.Item key={mode.value} onClick={() => onConversationModeChange?.(mode.value)}>
                {mode.icon} {mode.label}
              </Menu.Item>
            ))}
          </Menu.Dropdown>
        </Menu>

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
