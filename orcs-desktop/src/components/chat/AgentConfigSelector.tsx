/**
 * AgentConfigSelector - AI実行時の設定を選択するコンポーネント
 * Summary/ActionPlan/Expertise実行時に使用するBackend/Model/Optionsを指定
 */
import { useState, useMemo } from 'react';
import {
  ActionIcon,
  Tooltip,
  Popover,
  Stack,
  Select,
  Switch,
  Text,
  Group,
} from '@mantine/core';
import { IconSettings } from '@tabler/icons-react';
import type { PersonaBackend, GeminiOptions } from '../../types/agent';

/**
 * Agent設定
 */
export interface AgentConfig {
  backend: PersonaBackend;
  modelName?: string;
  geminiOptions?: GeminiOptions;
}

/**
 * Props
 */
interface AgentConfigSelectorProps {
  value: AgentConfig;
  onChange: (config: AgentConfig) => void;
}

/**
 * デフォルト設定
 */
const DEFAULT_CONFIG: AgentConfig = {
  backend: 'gemini_api',
  modelName: 'gemini-3-pro-preview',
  geminiOptions: {
    thinking_level: 'HIGH',
    google_search: true,
  },
};

/**
 * Backend別のModel選択肢
 */
const MODEL_OPTIONS: Record<PersonaBackend, { value: string; label: string }[]> = {
  claude_api: [
    { value: 'claude-3-opus-20240229', label: 'Claude 3 Opus' },
    { value: 'claude-3-sonnet-20240229', label: 'Claude 3 Sonnet' },
    { value: 'claude-3-haiku-20240307', label: 'Claude 3 Haiku' },
  ],
  claude_cli: [
    { value: 'default', label: 'Default (CLI)' },
  ],
  gemini_api: [
    { value: 'gemini-3-pro-preview', label: 'Gemini 3 Pro (Preview)' },
    { value: 'gemini-2-flash-exp', label: 'Gemini 2 Flash (Experimental)' },
    { value: 'gemini-2-pro-exp', label: 'Gemini 2 Pro (Experimental)' },
    { value: 'gemini-1.5-pro', label: 'Gemini 1.5 Pro' },
    { value: 'gemini-1.5-flash', label: 'Gemini 1.5 Flash' },
  ],
  gemini_cli: [
    { value: 'default', label: 'Default (CLI)' },
  ],
  open_ai_api: [
    { value: 'gpt-4', label: 'GPT-4' },
    { value: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
    { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo' },
  ],
  codex_cli: [
    { value: 'default', label: 'Default (CLI)' },
  ],
};

/**
 * Thinking Level選択肢
 */
const THINKING_LEVEL_OPTIONS = [
  { value: 'LOW', label: '🧠 Low' },
  { value: 'MEDIUM', label: '🧠🧠 Medium' },
  { value: 'HIGH', label: '🧠🧠🧠 High' },
];

/**
 * 設定がデフォルトと異なるかチェック
 */
function isConfigChanged(config: AgentConfig): boolean {
  if (config.backend !== DEFAULT_CONFIG.backend) return true;
  if (config.modelName !== DEFAULT_CONFIG.modelName) return true;
  if (config.geminiOptions?.thinking_level !== DEFAULT_CONFIG.geminiOptions?.thinking_level) return true;
  if (config.geminiOptions?.google_search !== DEFAULT_CONFIG.geminiOptions?.google_search) return true;
  return false;
}

/**
 * Backend表示名
 */
const BACKEND_LABELS: Record<PersonaBackend, string> = {
  claude_api: '🌟 Claude API',
  claude_cli: '🌟 Claude CLI',
  gemini_api: '💎 Gemini API',
  gemini_cli: '💎 Gemini CLI',
  open_ai_api: '🤖 OpenAI API',
  codex_cli: '⚡ Codex CLI',
};

export function AgentConfigSelector({ value, onChange }: AgentConfigSelectorProps) {
  const [opened, setOpened] = useState(false);

  // 現在のBackendのModel選択肢
  const modelOptions = useMemo(() => MODEL_OPTIONS[value.backend] || [], [value.backend]);

  // 設定変更検出
  const isChanged = useMemo(() => isConfigChanged(value), [value]);

  // Backend変更ハンドラー
  const handleBackendChange = (backend: string | null) => {
    if (!backend) return;

    const newBackend = backend as PersonaBackend;
    const newModelOptions = MODEL_OPTIONS[newBackend] || [];
    const newModelName = newModelOptions[0]?.value || undefined;

    onChange({
      backend: newBackend,
      modelName: newModelName,
      geminiOptions:
        newBackend === 'gemini_api'
          ? {
              thinking_level: 'HIGH',
              google_search: true,
            }
          : undefined,
    });
  };

  // Model変更ハンドラー
  const handleModelChange = (modelName: string | null) => {
    onChange({
      ...value,
      modelName: modelName || undefined,
    });
  };

  // Thinking Level変更ハンドラー
  const handleThinkingLevelChange = (level: string | null) => {
    if (!level) return;

    onChange({
      ...value,
      geminiOptions: {
        ...value.geminiOptions,
        thinking_level: level,
      },
    });
  };

  // Google Search変更ハンドラー
  const handleGoogleSearchChange = (checked: boolean) => {
    onChange({
      ...value,
      geminiOptions: {
        ...value.geminiOptions,
        google_search: checked,
      },
    });
  };

  return (
    <Popover
      position="top-end"
      width={280}
      shadow="md"
      opened={opened}
      onChange={setOpened}
    >
      <Popover.Target>
        <Tooltip label="Agent Configuration" withArrow>
          <ActionIcon
            color={isChanged ? 'blue' : 'gray'}
            variant="filled"
            onClick={() => setOpened(!opened)}
            size="lg"
          >
            <IconSettings size={18} />
          </ActionIcon>
        </Tooltip>
      </Popover.Target>

      <Popover.Dropdown>
        <Stack gap="md">
          {/* Backend選択 */}
          <Select
            label={
              <Group gap={4}>
                <Text size="sm" fw={500}>
                  Backend
                </Text>
              </Group>
            }
            value={value.backend}
            onChange={handleBackendChange}
            data={Object.entries(BACKEND_LABELS).map(([key, label]) => ({
              value: key,
              label,
            }))}
            size="xs"
          />

          {/* Model選択 */}
          {modelOptions.length > 0 && (
            <Select
              label={
                <Group gap={4}>
                  <Text size="sm" fw={500}>
                    Model
                  </Text>
                </Group>
              }
              value={value.modelName}
              onChange={handleModelChange}
              data={modelOptions}
              size="xs"
            />
          )}

          {/* Gemini Options（gemini_api時のみ表示） */}
          {value.backend === 'gemini_api' && (
            <>
              <Select
                label={
                  <Group gap={4}>
                    <Text size="sm" fw={500}>
                      Thinking Level
                    </Text>
                  </Group>
                }
                value={value.geminiOptions?.thinking_level || 'HIGH'}
                onChange={handleThinkingLevelChange}
                data={THINKING_LEVEL_OPTIONS}
                size="xs"
              />

              <Switch
                label="Google Search"
                checked={value.geminiOptions?.google_search ?? true}
                onChange={(e) => handleGoogleSearchChange(e.currentTarget.checked)}
                size="sm"
              />
            </>
          )}

          {/* 設定変更の表示 */}
          {isChanged && (
            <Text size="xs" c="blue" ta="center">
              ✨ Custom Configuration
            </Text>
          )}
        </Stack>
      </Popover.Dropdown>
    </Popover>
  );
}
