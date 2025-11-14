/**
 * AITextField Component
 *
 * AI機能が統合されたテキスト入力フィールド
 *
 * @example
 * ```tsx
 * <AITextField
 *   value={value}
 *   onChange={setValue}
 *   context={{ scope: 'UserBio', type: 'long_text' }}
 *   placeholder="自己紹介を入力..."
 * />
 * ```
 */

import { Textarea, Group, ActionIcon, Popover, Stack, Button, Text } from '@mantine/core';
import { IconSparkles } from '@tabler/icons-react';
import { useAIRegister } from '../core/hooks/useAIRegister';
import { useAIContext } from '../core/context/AIContext';
import type { AIContextInfo } from '../core/types/ai';

export interface AITextFieldProps {
  /** 現在の値 */
  value: string;

  /** 値変更ハンドラ */
  onChange: (value: string) => void;

  /** AIコンテキスト */
  context: AIContextInfo;

  /** プレースホルダー */
  placeholder?: string;

  /** 最小行数 */
  minRows?: number;

  /** 最大行数 */
  maxRows?: number;

  /** 無効化 */
  disabled?: boolean;

  /** エラー */
  error?: string;

  /** ラベル */
  label?: string;

  /** 説明 */
  description?: string;
}

/**
 * AI機能統合テキストフィールド
 */
export function AITextField({
  value,
  onChange,
  context,
  placeholder,
  minRows = 3,
  maxRows = 10,
  disabled = false,
  error,
  label,
  description,
}: AITextFieldProps) {
  const { directions } = useAIContext();

  const ai = useAIRegister({
    context,
    getValue: () => value,
    setValue: onChange,
    enabled: !disabled,
    onError: (err) => {
      console.error('[AITextField] Error:', err);
      // TODO: エラートーストを表示
    },
  });

  return (
    <Stack gap="xs">
      <Group gap="xs" align="flex-start" wrap="nowrap">
        {/* メインのテキストエリア */}
        <Textarea
          value={value}
          onChange={(e) => onChange(e.currentTarget.value)}
          placeholder={placeholder}
          minRows={minRows}
          maxRows={maxRows}
          disabled={disabled}
          error={error}
          label={label}
          description={description}
          style={{ flex: 1 }}
        />

        {/* ✨ AIトリガーアイコン */}
        <Popover
          opened={ai.menuProps.isOpen}
          onClose={ai.menuProps.onClose}
          position="bottom-end"
          withArrow
          shadow="md"
        >
          <Popover.Target>
            <ActionIcon
              variant={ai.triggerProps.isActive ? 'filled' : 'light'}
              color="violet"
              size="lg"
              onClick={ai.triggerProps.onClick}
              loading={ai.state.isLoading}
              disabled={disabled}
              title="AI機能を開く"
            >
              <IconSparkles size={20} />
            </ActionIcon>
          </Popover.Target>

          <Popover.Dropdown>
            <Stack gap="xs">
              {/* エラー表示 */}
              {ai.state.error && (
                <Text size="xs" c="red">
                  {ai.state.error.message}
                </Text>
              )}

              {/* 基本アクション */}
              <Group gap="xs">
                <Button
                  leftSection="💫"
                  size="xs"
                  variant="light"
                  onClick={() => ai.actions.generate()}
                  loading={ai.state.isLoading}
                  disabled={disabled}
                >
                  生成
                </Button>
                <Button
                  leftSection="🖌️"
                  size="xs"
                  variant="light"
                  onClick={() => ai.actions.refine()}
                  loading={ai.state.isLoading}
                  disabled={disabled}
                >
                  修正
                </Button>
                <ActionIcon
                  size="md"
                  variant="subtle"
                  onClick={ai.actions.undo}
                  disabled={!ai.state.canUndo || ai.state.isLoading}
                  title="元に戻す"
                >
                  ←
                </ActionIcon>
                <ActionIcon
                  size="md"
                  variant="subtle"
                  onClick={ai.actions.showHistory}
                  disabled={ai.state.history.length === 0}
                  title="履歴を表示"
                >
                  🗒️
                </ActionIcon>
              </Group>

              {/* 🏷️ 方向性メニュー */}
              <Popover position="left-start" withArrow>
                <Popover.Target>
                  <Button size="xs" variant="subtle" disabled={ai.state.isLoading}>
                    🏷️ 方向性を指定
                  </Button>
                </Popover.Target>
                <Popover.Dropdown>
                  <Stack gap="xs">
                    <Text size="xs" fw={600} c="dimmed">
                      方向性を選んでアクション
                    </Text>
                    {directions.map((dir) => (
                      <Group key={dir} gap="xs" justify="space-between">
                        <Text size="sm">{dir}</Text>
                        <Group gap="xs">
                          <ActionIcon
                            size="sm"
                            variant="light"
                            color="violet"
                            onClick={() => ai.actions.generate(dir)}
                            loading={ai.state.isLoading}
                            title={`${dir}で生成`}
                          >
                            💫
                          </ActionIcon>
                          <ActionIcon
                            size="sm"
                            variant="light"
                            color="blue"
                            onClick={() => ai.actions.refine(dir)}
                            loading={ai.state.isLoading}
                            title={`${dir}で修正`}
                          >
                            🖌️
                          </ActionIcon>
                        </Group>
                      </Group>
                    ))}
                  </Stack>
                </Popover.Dropdown>
              </Popover>

              {/* 💬 チャットを開く */}
              <Button
                size="xs"
                variant="subtle"
                onClick={ai.actions.showChat}
                disabled={ai.state.isLoading}
              >
                💬 チャットで相談
              </Button>

              {/* 履歴情報 */}
              {ai.state.history.length > 0 && (
                <Text size="xs" c="dimmed">
                  履歴: {ai.state.history.length}件
                </Text>
              )}
            </Stack>
          </Popover.Dropdown>
        </Popover>
      </Group>
    </Stack>
  );
}
