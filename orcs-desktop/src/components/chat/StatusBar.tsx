import { useState } from 'react';
import { Paper, Group, Badge, Text, Divider, Tooltip, Menu, Checkbox, ScrollArea, ActionIcon } from '@mantine/core';
import { IconAdjustments, IconAt, IconPlayerPlay } from '@tabler/icons-react';
import { StatusInfo } from '../../types/status';
import { GitInfo } from '../../types/git';
import { PersonaConfig } from '../../types/agent';
import { DEFAULT_STYLE_ICON, DEFAULT_STYLE_LABEL, getConversationModeOption, getTalkStyleOption, TALK_STYLES, EXECUTION_STRATEGIES, CONVERSATION_MODES, DialoguePreset, matchesPreset } from '../../types/conversation';
import type { ContextMode } from '../../types/session';

/**
 * Context Mode options for StatusBar
 */
const CONTEXT_MODES = [
  { value: 'rich', icon: '📚', label: 'Rich Context', description: 'Full context with all extensions' },
  { value: 'clean', icon: '🧹', label: 'Clean Context', description: 'Expertise only, minimal extensions' },
] as const;

interface StatusBarProps {
  status: StatusInfo;
  gitInfo?: GitInfo;
  participatingAgentsCount?: number;
  totalPersonas?: number;
  autoMode?: boolean;
  conversationMode?: string;
  talkStyle?: string | null;
  executionStrategy?: string;
  contextMode?: ContextMode;
  sandboxState?: import('../../bindings/generated').SandboxState | null;
  personas?: PersonaConfig[];
  activeParticipantIds?: string[];
  dialoguePresets?: DialoguePreset[];
  onTalkStyleChange?: (style: string | null) => void;
  onExecutionStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onContextModeChange?: (mode: ContextMode) => void;
  onToggleParticipant?: (personaId: string, isChecked: boolean) => void;
  onApplyPreset?: (presetId: string) => void;
  onMentionPersona?: (personaId: string) => void;
  onInvokePersona?: (personaId: string) => void;
}

export function StatusBar({ status, gitInfo, participatingAgentsCount = 0, totalPersonas = 0, autoMode = false, conversationMode = 'normal', talkStyle = null, executionStrategy = 'sequential', contextMode = 'rich', sandboxState = null, personas = [], activeParticipantIds = [], dialoguePresets = [], onTalkStyleChange, onExecutionStrategyChange, onConversationModeChange, onContextModeChange, onToggleParticipant, onApplyPreset, onMentionPersona, onInvokePersona }: StatusBarProps) {
  const [personasMenuOpened, setPersonasMenuOpened] = useState(false);

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
          <Menu position="top" withArrow closeOnItemClick={false} opened={personasMenuOpened} onChange={setPersonasMenuOpened}>
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
                          onChange={(e) => {
                            e.stopPropagation();
                            onToggleParticipant?.(persona.id, e.currentTarget.checked);
                          }}
                          size="sm"
                        />
                        <Text size="sm" truncate style={{ flex: 1, minWidth: 0 }}>
                          {persona.icon || '👤'} {persona.name} - {persona.role.length > 20 ? persona.role.substring(0, 20) + '...' : persona.role}
                        </Text>
                        <Group gap={4} wrap="nowrap">
                          <Tooltip label="Insert @mention" withArrow position="top">
                            <ActionIcon
                              size="xs"
                              variant="subtle"
                              color="blue"
                              onClick={(e) => {
                                e.stopPropagation();
                                onMentionPersona?.(persona.id);
                                setPersonasMenuOpened(false);
                              }}
                            >
                              <IconAt size={14} />
                            </ActionIcon>
                          </Tooltip>
                          <Tooltip label="Invoke now" withArrow position="top">
                            <ActionIcon
                              size="xs"
                              variant="subtle"
                              color="green"
                              onClick={(e) => {
                                e.stopPropagation();
                                onInvokePersona?.(persona.id);
                                setPersonasMenuOpened(false);
                              }}
                            >
                              <IconPlayerPlay size={14} />
                            </ActionIcon>
                          </Tooltip>
                        </Group>
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

        {/* Context Mode */}
        <Divider orientation="vertical" />
        <Menu position="top" withArrow>
          <Menu.Target>
            <Tooltip label={CONTEXT_MODES.find(m => m.value === contextMode)?.label || 'Rich Context'} withArrow>
              <Text size="lg" style={{ cursor: 'pointer' }}>
                {CONTEXT_MODES.find(m => m.value === contextMode)?.icon || '📚'}
              </Text>
            </Tooltip>
          </Menu.Target>
          <Menu.Dropdown>
            {CONTEXT_MODES.map((mode) => (
              <Menu.Item key={mode.value} onClick={() => onContextModeChange?.(mode.value)}>
                {mode.icon} {mode.label}
                <Text size="xs" c="dimmed" style={{ marginLeft: 8 }}>
                  {mode.description}
                </Text>
              </Menu.Item>
            ))}
          </Menu.Dropdown>
        </Menu>

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

        {/* Dialogue Preset */}
        <Divider orientation="vertical" />
        <Menu position="top" withArrow>
          <Menu.Target>
            <Tooltip label="Dialogue Presets" withArrow>
              <div style={{ cursor: 'pointer', display: 'flex', alignItems: 'center' }}>
                <IconAdjustments size={18} stroke={1.5} />
              </div>
            </Tooltip>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Label>System Presets</Menu.Label>
            {dialoguePresets
              .filter(p => p.source === 'system')
              .map((preset) => {
                const isMatching = matchesPreset(preset, executionStrategy, conversationMode, talkStyle);
                return (
                  <Menu.Item
                    key={preset.id}
                    onClick={() => onApplyPreset?.(preset.id)}
                    leftSection={isMatching ? '🟡' : '　'}
                  >
                    {preset.name}{preset.icon && ` ${preset.icon}`}
                    {preset.description && (
                      <Text size="xs" c="dimmed" style={{ marginLeft: 8 }}>
                        {preset.description}
                      </Text>
                    )}
                  </Menu.Item>
                );
              })}
            {dialoguePresets.filter(p => p.source === 'user').length > 0 && (
              <>
                <Menu.Divider />
                <Menu.Label>User Presets</Menu.Label>
                {dialoguePresets
                  .filter(p => p.source === 'user')
                  .map((preset) => {
                    const isMatching = matchesPreset(preset, executionStrategy, conversationMode, talkStyle);
                    return (
                      <Menu.Item
                        key={preset.id}
                        onClick={() => onApplyPreset?.(preset.id)}
                        leftSection={isMatching ? '🎨' : undefined}
                      >
                        {preset.name}{preset.icon && ` ${preset.icon}`}
                      </Menu.Item>
                    );
                  })}
              </>
            )}
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

        {/* Sandbox Mode Indicator */}
        {sandboxState && (
          <>
            <Divider orientation="vertical" />
            <Tooltip label={`Worktree: ${sandboxState.worktree_path}`} withArrow>
              <Badge color="orange" size="sm" variant="filled">
                🔬 Sandbox: {sandboxState.sandbox_branch}
              </Badge>
            </Tooltip>
          </>
        )}
      </Group>
    </Paper>
  );
}
