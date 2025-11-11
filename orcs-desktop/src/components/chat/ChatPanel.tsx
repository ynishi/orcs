/**
 * ChatPanel - 1つのタブ（セッション）のチャット画面を管理
 * TabContextから状態を取得し、軽量なプレゼンテーション層として機能
 */
import { useRef, useEffect, useState } from 'react';
import {
  Textarea,
  Button,
  ScrollArea,
  Stack,
  Box,
  Group,
  CopyButton,
  ActionIcon,
  Tooltip,
  Badge,
  CloseButton,
  Paper,
  Text,
} from '@mantine/core';
import { IconSettings } from '@tabler/icons-react';
import { MessageItem } from './MessageItem';
import { StatusBar } from './StatusBar';
import { CommandSuggestions } from './CommandSuggestions';
import { AgentSuggestions } from './AgentSuggestions';
import { ThinkingIndicator } from './ThinkingIndicator';
import { AutoChatSettingsModal } from './AutoChatSettingsModal';
import type { SessionTab } from '../../context/TabContext';
import type { StatusInfo } from '../../types/status';
import type { GitInfo } from '../../types/git';
import type { Workspace } from '../../types/workspace';
import type { CommandDefinition } from '../../types/command';
import type { Agent } from '../../types/agent';
import type { PersonaConfig } from '../../types/agent';
import type { AutoChatConfig } from '../../types/session';

interface ChatPanelProps {
  tab: SessionTab;
  status: StatusInfo;
  userNickname: string;
  gitInfo: GitInfo;
  autoMode: boolean;
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  personas: PersonaConfig[];
  activeParticipantIds: string[];
  workspace: Workspace | null;
  
  // サジェスト関連
  showSuggestions: boolean;
  filteredCommands: CommandDefinition[];
  selectedSuggestionIndex: number;
  showAgentSuggestions: boolean;
  filteredAgents: Agent[];
  selectedAgentIndex: number;
  
  // イベントハンドラー
  onSubmit: (e: React.FormEvent) => void;
  onInputChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  onFileSelect: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onRemoveFile: (index: number) => void;
  onDragOver: (e: React.DragEvent) => void;
  onDragLeave: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent) => void;
  onSaveMessageToWorkspace: (message: import('../../types/message').Message) => Promise<void>;
  onExecuteAsTask: (message: import('../../types/message').Message) => Promise<void>;
  onAutoModeChange: (autoMode: boolean) => void;
  onSelectCommand: (command: CommandDefinition) => void;
  onSelectAgent: (agent: Agent) => void;
  onHoverSuggestion: (index: number) => void;
}

export function ChatPanel({
  tab,
  status,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  // @ts-ignore - userNickname may be used in future features
  userNickname,
  gitInfo,
  autoMode,
  conversationMode,
  talkStyle,
  executionStrategy,
  personas,
  activeParticipantIds,
  workspace,
  showSuggestions,
  filteredCommands,
  selectedSuggestionIndex,
  showAgentSuggestions,
  filteredAgents,
  selectedAgentIndex,
  onSubmit,
  onInputChange,
  onKeyDown,
  onFileSelect,
  onRemoveFile,
  onDragOver,
  onDragLeave,
  onDrop,
  onSaveMessageToWorkspace,
  onExecuteAsTask,
  onAutoModeChange,
  onSelectCommand,
  onSelectAgent,
  onHoverSuggestion,
}: ChatPanelProps) {
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const previousMessageCount = useRef<number>(0);
  const previousTabId = useRef<string>(''); // Empty string to detect first render
  const hasScrolledForTab = useRef<Set<string>>(new Set()); // Track which tabs have been scrolled

  // AutoChat settings state
  const [autoChatSettingsOpened, setAutoChatSettingsOpened] = useState(false);
  const [autoChatConfig, setAutoChatConfig] = useState<AutoChatConfig | null>(null);
  const [autoChatIteration, setAutoChatIteration] = useState<number | undefined>(undefined);
  const autoChatPollingRef = useRef<number | null>(null);

  // Load AutoChat config from backend when tab changes
  useEffect(() => {
    const loadAutoChatConfig = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const config = await invoke<AutoChatConfig | null>('get_auto_chat_config', {
          sessionId: tab.sessionId,
        });

        // If no config exists, set default and save to backend
        if (!config) {
          const defaultConfig: AutoChatConfig = {
            max_iterations: 5,
            stop_condition: 'iteration_count',
            web_search_enabled: true,
          };
          setAutoChatConfig(defaultConfig);

          // Save default config to backend
          await invoke('update_auto_chat_config', {
            sessionId: tab.sessionId,
            config: defaultConfig,
          });
          console.log('[ChatPanel] Saved default AutoChat config to backend');
        } else {
          setAutoChatConfig(config);
        }
      } catch (error) {
        console.error('[ChatPanel] Failed to load AutoChat config:', error);
        // Set default on error
        const defaultConfig: AutoChatConfig = {
          max_iterations: 5,
          stop_condition: 'iteration_count',
          web_search_enabled: true,
        };
        setAutoChatConfig(defaultConfig);
      }
    };

    loadAutoChatConfig();
  }, [tab.sessionId]); // Reload when tab changes

  const handleSaveAutoChatConfig = async (config: AutoChatConfig) => {
    setAutoChatConfig(config);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_auto_chat_config', {
        sessionId: tab.sessionId,
        config,
      });
      console.log('[ChatPanel] AutoChat config saved successfully');
    } catch (error) {
      console.error('[ChatPanel] Failed to save AutoChat config:', error);
      // TODO: Show error notification to user
    }
  };

  // Start AutoChat when autoMode changes to true
  useEffect(() => {
    if (!autoMode) {
      // Stop polling when autoMode is off
      if (autoChatPollingRef.current) {
        clearInterval(autoChatPollingRef.current);
        autoChatPollingRef.current = null;
      }
      setAutoChatIteration(undefined);
      return;
    }

    // AutoMode turned ON - start AutoChat
    const startAutoChat = async () => {
      const input = tab.input.trim() || 'Continue the discussion.';
      const filePaths = tab.attachedFiles.length > 0 ? tab.attachedFiles : undefined;

      console.log('[ChatPanel] Starting AutoChat with input:', input);

      try {
        const { invoke } = await import('@tauri-apps/api/core');

        // Start polling iteration status
        autoChatPollingRef.current = setInterval(async () => {
          try {
            const iteration = await invoke<number | null>('get_auto_chat_status', {
              sessionId: tab.sessionId,
            });
            setAutoChatIteration(iteration ?? undefined);

            // Stop polling if iteration is null (AutoChat completed)
            if (iteration === null && autoChatPollingRef.current) {
              clearInterval(autoChatPollingRef.current);
              autoChatPollingRef.current = null;
              onAutoModeChange(false); // Turn off autoMode
            }
          } catch (error) {
            console.error('[ChatPanel] Failed to poll AutoChat status:', error);
          }
        }, 500); // Poll every 500ms

        // Start AutoChat (this is a long-running operation)
        await invoke('start_auto_chat', {
          input,
          filePaths,
        });

        console.log('[ChatPanel] AutoChat completed');
      } catch (error) {
        console.error('[ChatPanel] AutoChat failed:', error);
        onAutoModeChange(false); // Turn off autoMode on error

        if (autoChatPollingRef.current) {
          clearInterval(autoChatPollingRef.current);
          autoChatPollingRef.current = null;
        }
      }
    };

    startAutoChat();

    // Cleanup polling on unmount or when autoMode changes
    return () => {
      if (autoChatPollingRef.current) {
        clearInterval(autoChatPollingRef.current);
        autoChatPollingRef.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoMode, tab.sessionId, onAutoModeChange]); // Only re-run when autoMode/sessionId changes, NOT when input changes

  // Auto-scroll to bottom when new messages are added or tab is first opened
  useEffect(() => {
    const currentMessageCount = tab.messages.length;
    const isNewTab = tab.id !== previousTabId.current;
    const isFirstTimeOpeningThisTab = !hasScrolledForTab.current.has(tab.id);

    // Scroll if: (1) message count increased, OR (2) tab opened for first time
    if ((currentMessageCount > previousMessageCount.current || (isNewTab && isFirstTimeOpeningThisTab)) && viewport.current) {
      // Use setTimeout to ensure DOM is fully rendered
      setTimeout(() => {
        if (viewport.current) {
          viewport.current.scrollTo({
            top: viewport.current.scrollHeight,
            behavior: 'smooth',
          });
        }
      }, 0);

      // Mark this tab as scrolled
      if (isFirstTimeOpeningThisTab) {
        hasScrolledForTab.current.add(tab.id);
      }
    }

    previousMessageCount.current = currentMessageCount;
    previousTabId.current = tab.id;
  }, [tab.messages, tab.id]);

  const getThreadAsText = (): string => {
    return tab.messages
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  };

  return (
    <Stack gap="xs" style={{ height: '100%', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
      {/* メッセージエリア */}
      <Box
        style={{ flex: 1, position: 'relative', minHeight: 0 }}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
      >
        <ScrollArea h="100%" viewportRef={viewport}>
          <Stack gap="sm" p="md">
            {tab.messages.map((message) => (
              <MessageItem
                key={message.id}
                message={message}
                onSaveToWorkspace={onSaveMessageToWorkspace}
                onExecuteAsTask={onExecuteAsTask}
                workspaceRootPath={workspace?.rootPath}
              />
            ))}
            {tab.isAiThinking && <ThinkingIndicator personaName={tab.thinkingPersona} />}
          </Stack>
        </ScrollArea>

        {tab.isDragging && (
          <Paper
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: 'rgba(30, 144, 255, 0.1)',
              border: '3px dashed #1e90ff',
              borderRadius: '12px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              zIndex: 1000,
              pointerEvents: 'none',
            }}
          >
            <Text size="xl" fw={700} c="blue">
              📁 Drop files here
            </Text>
          </Paper>
        )}
      </Box>

      {/* 入力フォーム */}
      <form onSubmit={onSubmit}>
        <Stack gap="xs">
          {tab.attachedFiles.length > 0 && (
            <Group gap="xs">
              {tab.attachedFiles.map((file, index) => (
                <Badge
                  key={index}
                  size="lg"
                  variant="light"
                  rightSection={
                    <CloseButton size="xs" onClick={() => onRemoveFile(index)} />
                  }
                >
                  📎 {file.name}
                </Badge>
              ))}
            </Group>
          )}

          <Box style={{ position: 'relative' }}>
            {showSuggestions && (
              <CommandSuggestions
                commands={filteredCommands}
                selectedIndex={selectedSuggestionIndex}
                onSelect={onSelectCommand}
                onHover={onHoverSuggestion}
              />
            )}

            {showAgentSuggestions && (
              <AgentSuggestions
                agents={filteredAgents}
                selectedIndex={selectedAgentIndex}
                onSelect={onSelectAgent}
              />
            )}

            <Textarea
              ref={textareaRef}
              value={tab.input}
              onChange={(e) => onInputChange(e.currentTarget.value)}
              onKeyDown={onKeyDown}
              placeholder={
                executionStrategy === 'mentioned'
                  ? 'Type @PersonaName to mention, or /help for commands... (⌘+Enter to send)'
                  : 'Type your message or /help for commands... (⌘+Enter to send)'
              }
              size="md"
              minRows={1}
              maxRows={4}
              autosize
            />
          </Box>

          <Group gap="xs">
            <Tooltip label="Attach files">
              <Button variant="light" size="sm" component="label" leftSection="📎">
                Attach
                <input type="file" multiple hidden onChange={onFileSelect} />
              </Button>
            </Tooltip>

            <Button type="submit" style={{ flex: 1 }}>
              Send
            </Button>

            <Tooltip label="AutoChat settings">
              <ActionIcon
                variant="light"
                onClick={() => setAutoChatSettingsOpened(true)}
                size="lg"
              >
                <IconSettings size={18} />
              </ActionIcon>
            </Tooltip>

            <Tooltip label={autoMode ? 'Stop AUTO mode' : 'Start AUTO mode'}>
              <ActionIcon
                color={autoMode ? 'red' : 'green'}
                variant={autoMode ? 'filled' : 'light'}
                onClick={() => onAutoModeChange(!autoMode)}
                size="lg"
              >
                {autoMode ? '⏹️' : '▶️'}
              </ActionIcon>
            </Tooltip>

            <CopyButton value={getThreadAsText()}>
              {({ copied, copy }) => (
                <Tooltip label={copied ? 'Copied!' : 'Copy thread'}>
                  <ActionIcon
                    color={copied ? 'teal' : 'blue'}
                    variant="light"
                    onClick={copy}
                    size="lg"
                  >
                    {copied ? '✓' : '📄'}
                  </ActionIcon>
                </Tooltip>
              )}
            </CopyButton>
          </Group>
        </Stack>
      </form>

      <StatusBar
        status={status}
        gitInfo={gitInfo}
        participatingAgentsCount={activeParticipantIds.length}
        totalPersonas={personas.length}
        autoMode={autoMode}
        conversationMode={conversationMode}
        talkStyle={talkStyle}
        executionStrategy={executionStrategy}
        autoChatIteration={autoChatIteration}
        autoChatMaxIterations={autoChatConfig?.max_iterations}
      />

      <AutoChatSettingsModal
        opened={autoChatSettingsOpened}
        onClose={() => setAutoChatSettingsOpened(false)}
        config={autoChatConfig}
        onSave={handleSaveAutoChatConfig}
      />
    </Stack>
  );
}
