/**
 * ChatPanel - 1つのタブ（セッション）のチャット画面を管理
 * TabContextから状態を取得し、軽量なプレゼンテーション層として機能
 */
import { useRef, useEffect, useState, useCallback, memo } from 'react';
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
import { IconSettings, IconClipboardList, IconFileText, IconBulb, IconFileCode, IconVolume, IconVolumeOff, IconPlayerPlay, IconPlayerStop, IconFile, IconCheck, IconPaperclip, IconFileExport, IconSearch } from '@tabler/icons-react';
import { MessageItem } from './MessageItem';
import { StatusBar } from './StatusBar';
import { AgentConfigSelector } from './AgentConfigSelector';
import type { AgentConfig } from './AgentConfigSelector';
import { CommandSuggestions } from './CommandSuggestions';
import { AgentSuggestions } from './AgentSuggestions';
import { ThinkingIndicator } from './ThinkingIndicator';
import { AutoChatSettingsModal } from './AutoChatSettingsModal';
import { SlashCommandEditorModal } from '../slash_commands/SlashCommandEditorModal';
import { QuickActionDock } from './QuickActionDock';
import { useTabContext, useTabInput } from '../../context/TabContext';
import type { SessionTab } from '../../context/TabContext';
import type { StatusInfo } from '../../types/status';
import type { GitInfo } from '../../types/git';
import type { Workspace } from '../../types/workspace';
import type { CommandDefinition } from '../../types/command';
import type { Agent } from '../../types/agent';
import type { PersonaConfig } from '../../types/agent';
import type { AutoChatConfig, ContextMode } from '../../types/session';
import type { Message } from '../../types/message';
import type { SlashCommand } from '../../types/slash_command';
import { notifications } from '@mantine/notifications';

interface ChatPanelProps {
  tab: SessionTab;
  isActive: boolean; // Whether this tab is currently active
  currentSessionId: string | null; // Backend's active session ID
  status: StatusInfo;
  userNickname: string;
  gitInfo: GitInfo;
  autoMode: boolean;
  conversationMode: string;
  talkStyle: string | null;
  executionStrategy: string;
  contextMode: ContextMode;
  sandboxState?: import('../../bindings/generated').SandboxState | null;
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
  onTalkStyleChange?: (style: string | null) => void;
  onExecutionStrategyChange?: (strategy: string) => void;
  onConversationModeChange?: (mode: string) => void;
  onContextModeChange?: (mode: ContextMode) => void;
  onToggleParticipant?: (personaId: string, isChecked: boolean) => void;
  dialoguePresets?: import('../../types/conversation').DialoguePreset[];
  onApplyPreset?: (presetId: string) => void;
  onMentionPersona?: (personaId: string) => void;
  onInvokePersona?: (personaId: string) => void;
  onSelectCommand: (command: CommandDefinition) => void;
  onSelectAgent: (agent: Agent) => void;
  onHoverSuggestion: (index: number) => void;
  onSaveSessionToWorkspace?: () => void;
  onPasteAndAttach?: () => Promise<void>;
}

interface MessageListProps {
  messages: Message[];
  onSaveMessageToWorkspace: (message: Message) => Promise<void>;
  onExecuteAsTask: (message: Message) => Promise<void>;
  onCreateSlashCommand: (message: Message) => void;
  onCreatePersona: (message: Message) => void;
  onRedo: (message: Message) => void;
  onCloseMessage: (message: Message, isClosed: boolean) => Promise<void>;
  workspaceRootPath?: string;
}

const MessageList = memo(
  ({
    messages,
    onSaveMessageToWorkspace,
    onExecuteAsTask,
    onCreateSlashCommand,
    onCreatePersona,
    onRedo,
    onCloseMessage,
    workspaceRootPath,
  }: MessageListProps) => (
    <>
      {messages.map((message) => (
        <MessageItem
          key={message.id}
          message={message}
          onSaveToWorkspace={onSaveMessageToWorkspace}
          onExecuteAsTask={onExecuteAsTask}
          onCreateSlashCommand={onCreateSlashCommand}
          onCreatePersona={onCreatePersona}
          onRedo={onRedo}
          onCloseMessage={onCloseMessage}
          workspaceRootPath={workspaceRootPath}
        />
      ))}
    </>
  ),
  (prev, next) =>
    prev.messages === next.messages &&
    prev.workspaceRootPath === next.workspaceRootPath &&
    onSaveExecHandlersEqual(prev, next)
);

function onSaveExecHandlersEqual(
  prev: MessageListProps,
  next: MessageListProps
): boolean {
  return (
    prev.onSaveMessageToWorkspace === next.onSaveMessageToWorkspace &&
    prev.onExecuteAsTask === next.onExecuteAsTask &&
    prev.onCreateSlashCommand === next.onCreateSlashCommand &&
    prev.onCreatePersona === next.onCreatePersona &&
    prev.onRedo === next.onRedo
  );
}

export const ChatPanel = memo(function ChatPanel({
  tab,
  isActive,
  currentSessionId,
  status,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  // @ts-ignore - userNickname may be used in future features
  userNickname,
  gitInfo,
  autoMode,
  conversationMode,
  talkStyle,
  executionStrategy,
  contextMode,
  sandboxState,
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
  onTalkStyleChange,
  onExecutionStrategyChange,
  onConversationModeChange,
  onContextModeChange,
  onToggleParticipant,
  dialoguePresets,
  onApplyPreset,
  onMentionPersona,
  onInvokePersona,
  onSelectCommand,
  onSelectAgent,
  onHoverSuggestion,
  onSaveSessionToWorkspace,
  onPasteAndAttach,
}: ChatPanelProps) {
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const previousMessageCount = useRef<number>(0);
  const previousTabId = useRef<string>(''); // Empty string to detect first render
  const hasScrolledForTab = useRef<Set<string>>(new Set()); // Track which tabs have been scrolled

  // TabContext for adding messages and managing tab state
  const { addMessageToTab, setTabThinking, updateTabAttachedFiles, updateTabMessages, updateTabInput } = useTabContext();
  // Performance: activeTabInput を直接 subscribe（App.tsx は subscribe しない → キー入力で App が再レンダリングされない）
  const { activeTabInput } = useTabInput();

  // AutoChat settings state
  const [autoChatSettingsOpened, setAutoChatSettingsOpened] = useState(false);
  const [autoChatConfig, setAutoChatConfig] = useState<AutoChatConfig | null>(null);

  // Mute state
  const [isMuted, setIsMuted] = useState(false);

  // SlashCommand creation state
  const [slashCommandModalOpened, setSlashCommandModalOpened] = useState(false);
  const [slashCommandDraft, setSlashCommandDraft] = useState<Partial<SlashCommand> | null>(null);

  // Hover state for thread command icons with auto-hide
  const [isMessageAreaHovered, setIsMessageAreaHovered] = useState(false);
  const [showThreadActions, setShowThreadActions] = useState(false);
  const hideTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Agent configuration for Summary/ActionPlan/Expertise
  const [agentConfig, setAgentConfig] = useState<AgentConfig>({
    backend: 'gemini_api',
    modelName: 'gemini-3-pro-preview',
    geminiOptions: {
      thinking_level: 'HIGH',
      google_search: true,
    },
    sessionScope: 'full',
  });

  // SlashCommands for QuickActionDock
  const [availableSlashCommands, setAvailableSlashCommands] = useState<SlashCommand[]>([]);

  // Load available slash commands
  useEffect(() => {
    const loadSlashCommands = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const commands = await invoke<SlashCommand[]>('list_slash_commands');
        setAvailableSlashCommands(commands);
      } catch (error) {
        console.error('[ChatPanel] Failed to load slash commands:', error);
      }
    };

    loadSlashCommands();
  }, []);

  // Handle QuickAction command execution
  const handleExecuteQuickAction = useCallback(async (commandName: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Get the command details
      const command = availableSlashCommands.find((c) => c.name === commandName);
      if (!command) {
        throw new Error(`Command not found: ${commandName}`);
      }

      // Expand the command template
      const expanded = await invoke<{ content: string; workingDir?: string }>('expand_command_template', {
        commandName,
        args: null,
      });

      // Handle based on command type
      if (command.type === 'prompt') {
        // For prompt commands, send the expanded content as user input
        onInputChange(expanded.content);
      } else if (command.type === 'shell') {
        // For shell commands, execute and show output
        setTabThinking(tab.id, true, `Running /${commandName}`, true);
        const output = await invoke<string>('execute_shell_command', {
          command: expanded.content,
          workingDir: expanded.workingDir,
        });

        // Add output as system message
        const outputMessage: Message = {
          id: `${Date.now()}-shell-output`,
          type: 'ai',
          author: `Shell: /${commandName}`,
          text: `\`\`\`\n${output}\n\`\`\``,
          timestamp: new Date(),
        };
        addMessageToTab(tab.id, outputMessage);
        setTabThinking(tab.id, false);

        notifications.show({
          title: 'Command Executed',
          message: `/${commandName} completed`,
          color: 'green',
        });
      } else if (command.type === 'task') {
        // For task commands, execute as task
        setTabThinking(tab.id, true, `Executing /${commandName}`, true);
        await invoke<string>('execute_task_command', {
          commandName,
          args: null,
        });
        setTabThinking(tab.id, false);

        notifications.show({
          title: 'Task Started',
          message: `/${commandName} task is running`,
          color: 'blue',
        });
      } else if (command.type === 'action') {
        // For action commands, execute with session context
        setTabThinking(tab.id, true, `Executing /${commandName}`, true);
        const threadContent = getThreadAsText();
        const actionResponse = await invoke<{ result: string; personaInfo?: { name: string; icon?: string; backend: string } }>('execute_action_command', {
          commandName,
          threadContent,
          args: null,
          prevOutput: null,
        });
        setTabThinking(tab.id, false);

        // Build command header with persona info if available
        let commandHeader = `${command.icon || '⚡'} /${commandName}`;
        if (actionResponse.personaInfo) {
          const pi = actionResponse.personaInfo;
          commandHeader += ` by ${pi.icon || '👤'} ${pi.name} (${pi.backend})`;
        }
        commandHeader += '\n\n';

        const resultMessage: Message = {
          id: `${Date.now()}-action-result`,
          type: 'action_result',
          author: 'SYSTEM',
          text: commandHeader + actionResponse.result,
          timestamp: new Date(),
        };
        addMessageToTab(tab.id, resultMessage);

        notifications.show({
          title: 'Action Completed',
          message: `/${commandName} completed`,
          color: 'green',
        });
      }
    } catch (error) {
      console.error('[ChatPanel] Failed to execute quick action:', error);
      setTabThinking(tab.id, false);
      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : `Failed to execute /${commandName}`,
        color: 'red',
      });
    }
  }, [availableSlashCommands, tab.id, onInputChange, setTabThinking, addMessageToTab]);

  // Load AutoChat config from backend when tab changes
  // Only load for active tab to avoid Session ID mismatch errors
  useEffect(() => {
    if (!isActive) {
      // Skip loading for inactive tabs
      return;
    }

    // Only load when backend session matches tab session
    if (currentSessionId !== tab.sessionId) {
      // Backend session hasn't switched yet, wait for next trigger
      console.log('[ChatPanel] Skipping AutoChat config load: session mismatch', {
        tabSessionId: tab.sessionId.substring(0, 8),
        currentSessionId: currentSessionId?.substring(0, 8) || 'null',
      });
      return;
    }

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
  }, [tab.sessionId, currentSessionId, isActive]); // Reload when tab changes, backend session changes, or becomes active

  // Load mute status from backend when tab changes
  // Only load for active tab to reduce unnecessary backend calls
  useEffect(() => {
    if (!isActive) {
      return;
    }

    const loadMuteStatus = async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const status = await invoke<boolean>('get_mute_status');
        setIsMuted(status);
      } catch (error) {
        console.error('[ChatPanel] Failed to load mute status:', error);
        setIsMuted(false);
      }
    };

    loadMuteStatus();
  }, [tab.sessionId, isActive]);

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

  // Handle mute toggle
  const handleToggleMute = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const newMuteStatus = await invoke<boolean>('toggle_mute');
      setIsMuted(newMuteStatus);

      notifications.show({
        title: newMuteStatus ? 'Session Muted' : 'Session Unmuted',
        message: newMuteStatus
          ? 'AI responses are disabled. Messages will be recorded only.'
          : 'AI responses are enabled.',
        color: newMuteStatus ? 'gray' : 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to toggle mute:', error);
      notifications.show({
        title: 'Error',
        message: 'Failed to toggle mute status',
        color: 'red',
      });
    }
  };

  // Handle AutoChat start
  const handleAutoModeToggle = async () => {
    if (autoMode) {
      // Turn off AutoChat
      onAutoModeChange(false);
      return;
    }

    // Turn on AutoChat and start
    const input = tab.input.trim();
    const filePaths = tab.attachedFiles.length > 0 ? tab.attachedFiles.map(f => f.name) : undefined;

    console.log('[ChatPanel] Starting AutoChat with input:', input);

    // Clear input and attached files (same as normal send)
    updateTabInput(tab.id, '');
    updateTabAttachedFiles(tab.id, []);

    // Add user message ONLY if user actually entered something
    if (input) {
      const userMessage: Message = {
        id: `${Date.now()}-${Math.random()}`,
        type: 'user',
        author: userNickname,
        text: input,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, userMessage);
    }

    // Add system message (AutoChat開始通知)
    const maxIterations = autoChatConfig?.max_iterations || 5;
    const startMessage: Message = {
      id: `${Date.now()}-${Math.random()}-start`,
      type: 'system',
      author: 'System',
      text: `🤖 AutoChat開始: Agent同士で${maxIterations}回の対話を進めます。`,
      timestamp: new Date(),
    };
    addMessageToTab(tab.id, startMessage);

    // Turn on autoMode
    onAutoModeChange(true);

    // Set thinking state (display ThinkingIndicator)
    setTabThinking(tab.id, true, 'AutoChat');

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Start AutoChat (this is a long-running operation)
      // Backend will emit dialogue-turn events with AutoChat progress
      await invoke('start_auto_chat', {
        input,
        filePaths,
      });

      console.log('[ChatPanel] AutoChat completed');
    } catch (error) {
      console.error('[ChatPanel] AutoChat failed:', error);
      onAutoModeChange(false); // Turn off autoMode on error
    } finally {
      // Clear thinking state when done
      setTabThinking(tab.id, false);
    }
  };

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

  const getThreadAsText = useCallback((scope?: 'full' | 'recent'): string => {
    const effectiveScope = scope || agentConfig.sessionScope || 'full';
    const messagesToUse = effectiveScope === 'recent'
      ? tab.messages.slice(-10)  // Recent 10 messages
      : tab.messages;            // Full session

    return messagesToUse
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  }, [tab.messages, agentConfig.sessionScope]);

  // Handle cancelling current operation
  const handleCancelOperation = useCallback(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('cancel_current_operation');

      notifications.show({
        title: 'Operation Cancelled',
        message: 'The current operation has been cancelled.',
        color: 'orange',
      });

      // Clear thinking state
      setTabThinking(tab.id, false);
    } catch (error) {
      console.error('[ChatPanel] Failed to cancel operation:', error);
      notifications.show({
        title: 'Error',
        message: 'Failed to cancel operation',
        color: 'red',
      });
    }
  }, [tab.id, setTabThinking]);

  // Handle generating summary from thread
  const handleGenerateSummary = useCallback(async () => {
    const threadContent = getThreadAsText();

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Persist system message for Summary generation request
      await invoke('append_system_messages', {
        messages: [
          {
            content: '📝 Generating summary from conversation...',
            messageType: 'info',
            severity: 'info',
          },
        ],
      });

      // Set thinking state
      setTabThinking(tab.id, true, 'Summary', true); // Non-interactive command

      // Call backend to generate summary
      const summary = await invoke<string>('generate_summary', {
        threadContent,
        sessionId: tab.sessionId,
        agentConfig: {
          backend: agentConfig.backend,
          modelName: agentConfig.modelName,
          geminiOptions: agentConfig.geminiOptions,
        },
      });

      // Persist AI response with summary
      await invoke('append_system_messages', {
        messages: [
          {
            content: summary,
            messageType: 'ai_response',
            severity: 'info',
          },
        ],
      });

      // Add summary message to frontend tab
      const summaryMessage: Message = {
        id: `${Date.now()}-summary`,
        type: 'ai',
        author: 'Summary',
        text: summary,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, summaryMessage);

      notifications.show({
        title: 'Success',
        message: 'Summary generated successfully!',
        color: 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to generate summary:', error);

      // Persist error message
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('append_system_messages', {
        messages: [
          {
            content: `❌ Failed to generate summary: ${error instanceof Error ? error.message : 'Unknown error'}`,
            messageType: 'error',
            severity: 'error',
          },
        ],
      }).catch((e: unknown) => console.error('[ChatPanel] Failed to persist error message:', e));

      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to generate summary',
        color: 'red',
      });
    } finally {
      setTabThinking(tab.id, false);
    }
  }, [getThreadAsText, tab.id, tab.sessionId, agentConfig, setTabThinking, addMessageToTab]);

  // Handle generating action plan from thread
  const handleGenerateActionPlan = useCallback(async () => {
    const threadContent = getThreadAsText();

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Persist system message for ActionPlan generation request
      await invoke('append_system_messages', {
        messages: [
          {
            content: '📋 Generating ActionPlan from conversation...',
            messageType: 'info',
            severity: 'info',
          },
        ],
      });

      // Set thinking state
      setTabThinking(tab.id, true, 'ActionPlan', true); // Non-interactive command

      // Call backend to generate action plan
      const actionPlan = await invoke<string>('generate_action_plan', {
        threadContent,
        sessionId: tab.sessionId,
        agentConfig: {
          backend: agentConfig.backend,
          modelName: agentConfig.modelName,
          geminiOptions: agentConfig.geminiOptions,
        },
      });

      // Persist AI response with action plan
      await invoke('append_system_messages', {
        messages: [
          {
            content: actionPlan,
            messageType: 'ai_response',
            severity: 'info',
          },
        ],
      });

      // Add action plan message to frontend tab
      const actionPlanMessage: Message = {
        id: `${Date.now()}-actionplan`,
        type: 'ai',
        author: 'ActionPlan',
        text: actionPlan,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, actionPlanMessage);

      notifications.show({
        title: 'Success',
        message: 'ActionPlan generated successfully!',
        color: 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to generate ActionPlan:', error);

      // Persist error message
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('append_system_messages', {
        messages: [
          {
            content: `❌ Failed to generate ActionPlan: ${error instanceof Error ? error.message : 'Unknown error'}`,
            messageType: 'error',
            severity: 'error',
          },
        ],
      }).catch((e: unknown) => console.error('[ChatPanel] Failed to persist error message:', e));

      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to generate ActionPlan',
        color: 'red',
      });
    } finally {
      setTabThinking(tab.id, false);
    }
  }, [getThreadAsText, tab.id, tab.sessionId, agentConfig, setTabThinking, addMessageToTab]);

  // Handle generating expertise from thread
  const handleGenerateExpertise = useCallback(async () => {
    const threadContent = getThreadAsText();

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Persist system message for Expertise generation request
      await invoke('append_system_messages', {
        messages: [
          {
            content: '💡 Generating Expertise from conversation...',
            messageType: 'info',
            severity: 'info',
          },
        ],
      });

      // Set thinking state
      setTabThinking(tab.id, true, 'Expertise', true); // Non-interactive command

      // Call backend to generate expertise
      const expertise = await invoke<string>('generate_expertise', {
        threadContent,
        sessionId: tab.sessionId,
        agentConfig: {
          backend: agentConfig.backend,
          modelName: agentConfig.modelName,
          geminiOptions: agentConfig.geminiOptions,
        },
      });

      // Persist AI response with expertise
      await invoke('append_system_messages', {
        messages: [
          {
            content: expertise,
            messageType: 'ai_response',
            severity: 'info',
          },
        ],
      });

      // Add expertise message to frontend tab
      const expertiseMessage: Message = {
        id: `${Date.now()}-expertise`,
        type: 'ai',
        author: 'Expertise',
        text: expertise,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, expertiseMessage);

      notifications.show({
        title: 'Success',
        message: 'Expertise generated successfully!',
        color: 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to generate Expertise:', error);

      // Persist error message
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('append_system_messages', {
        messages: [
          {
            content: `❌ Failed to generate Expertise: ${error instanceof Error ? error.message : 'Unknown error'}`,
            messageType: 'error',
            severity: 'error',
          },
        ],
      }).catch((e: unknown) => console.error('[ChatPanel] Failed to persist error message:', e));

      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to generate Expertise',
        color: 'red',
      });
    } finally {
      setTabThinking(tab.id, false);
    }
  }, [getThreadAsText, tab.id, tab.sessionId, agentConfig, setTabThinking, addMessageToTab]);

  // Handle investigating workspace structure and status
  const handleInvestigateWorkspace = useCallback(async () => {
    if (!workspace) {
      notifications.show({
        title: 'Error',
        message: 'No workspace selected',
        color: 'red',
      });
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Persist system message for investigation request
      await invoke('append_system_messages', {
        messages: [
          {
            content: '🔍 Investigating workspace structure and status...',
            messageType: 'info',
            severity: 'info',
          },
        ],
      });

      // Set thinking state
      setTabThinking(tab.id, true, 'Investigating', true); // Non-interactive command

      // Call backend to investigate workspace
      const investigationResult = await invoke<any>('investigate_workspace', {
        workspaceId: workspace.id,
        investigationType: 'comprehensive',
      });

      // Format the investigation result for display
      const formattedResult = formatInvestigationResult(investigationResult);

      // Persist AI response with investigation result
      await invoke('append_system_messages', {
        messages: [
          {
            content: formattedResult,
            messageType: 'ai_response',
            severity: 'info',
          },
        ],
      });

      // Add investigation message to frontend tab
      const investigationMessage: Message = {
        id: `${Date.now()}-investigation`,
        type: 'ai',
        author: 'Workspace Investigation',
        text: formattedResult,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, investigationMessage);

      notifications.show({
        title: 'Success',
        message: 'Workspace investigation completed!',
        color: 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to investigate workspace:', error);

      // Persist error message
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('append_system_messages', {
        messages: [
          {
            content: `❌ Failed to investigate workspace: ${error instanceof Error ? error.message : 'Unknown error'}`,
            messageType: 'error',
            severity: 'error',
          },
        ],
      }).catch((e: unknown) => console.error('[ChatPanel] Failed to persist error message:', e));

      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to investigate workspace',
        color: 'red',
      });
    } finally {
      setTabThinking(tab.id, false);
    }
  }, [workspace, tab.id, setTabThinking, addMessageToTab]);

  // Helper function to format investigation result
  const formatInvestigationResult = (result: any): string => {
    // New format: Agent returns report directly as Markdown
    if (result.report) {
      return result.report;
    }

    // Fallback for unexpected format
    return `## Investigation Result\n\n${JSON.stringify(result, null, 2)}`;
  };

  // Handle generating Concept/Design Issue from thread
  const handleGenerateConceptIssue = useCallback(async () => {
    const threadContent = getThreadAsText();

    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Reset cancel flag before starting
      await invoke('reset_cancel_flag');

      // Persist system message for Concept/Design Issue generation request
      await invoke('append_system_messages', {
        messages: [
          {
            content: '📋 Generating comprehensive Concept/Design Issue from conversation...',
            messageType: 'info',
            severity: 'info',
          },
        ],
      });

      // Set thinking state
      setTabThinking(tab.id, true, 'Concept/Design Issue', true); // Non-interactive command

      // Call backend to generate concept/design issue
      const conceptIssue = await invoke<string>('generate_concept_issue', {
        threadContent,
        sessionId: tab.sessionId,
        agentConfig: {
          backend: agentConfig.backend,
          modelName: agentConfig.modelName,
          geminiOptions: agentConfig.geminiOptions,
        },
      });

      // Persist concept/design issue with AI response
      await invoke('append_system_messages', {
        messages: [
          {
            content: conceptIssue,
            messageType: 'ai_response',
            severity: 'info',
          },
        ],
      });

      // Add concept/design issue message to frontend tab
      const conceptIssueMessage: Message = {
        id: `${Date.now()}-concept-issue`,
        type: 'ai',
        author: 'Concept/Design Issue',
        text: conceptIssue,
        timestamp: new Date(),
      };
      addMessageToTab(tab.id, conceptIssueMessage);

      notifications.show({
        title: 'Success',
        message: 'Concept/Design Issue generated successfully!',
        color: 'green',
      });
    } catch (error) {
      console.error('[ChatPanel] Failed to generate Concept/Design Issue:', error);

      // Persist error message
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('append_system_messages', {
        messages: [
          {
            content: `❌ Failed to generate Concept/Design Issue: ${error instanceof Error ? error.message : 'Unknown error'}`,
            messageType: 'error',
            severity: 'error',
          },
        ],
      }).catch((e: unknown) => console.error('[ChatPanel] Failed to persist error message:', e));

      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to generate Concept/Design Issue',
        color: 'red',
      });
    } finally {
      setTabThinking(tab.id, false);
    }
  }, [getThreadAsText, tab.id, tab.sessionId, agentConfig, setTabThinking, addMessageToTab]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (hideTimeoutRef.current) {
        clearTimeout(hideTimeoutRef.current);
      }
    };
  }, []);

  // Handle creating a slash command from a message
  const handleCreateSlashCommand = useCallback((message: Message) => {
    setSlashCommandDraft({
      type: 'prompt',
      content: message.text,  // Use message content only, not entire thread
      icon: '⚡',
    });
    setSlashCommandModalOpened(true);
  }, []);

  // Handle creating a persona from a message (same as slash command for now)
  const handleCreatePersona = useCallback((message: Message) => {
    setSlashCommandDraft({
      type: 'prompt',
      content: message.text,  // Use message content only, not entire thread
      icon: '👤',
    });
    setSlashCommandModalOpened(true);
  }, []);

  // Handle redoing a message (re-send the message text)
  const handleRedo = useCallback((message: Message) => {
    onInputChange(message.text);
  }, [onInputChange]);

  // Handle close/open message (add/remove ~~ prefix)
  const handleCloseMessage = useCallback(async (message: Message, isClosed: boolean) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');

      // Calculate new content: add ~~ prefix if closing, remove if opening
      const currentText = message.text;
      const newContent = isClosed
        ? `~~${currentText}`
        : currentText.startsWith('~~')
          ? currentText.slice(2)
          : currentText;

      // Get the persona_id (author) - for user messages, it's the user nickname
      const personaId = message.author;
      // Convert timestamp to ISO string for backend
      const timestamp = message.timestamp.toISOString();

      // Update in backend
      await invoke('update_message_content', {
        personaId,
        timestamp,
        newContent,
      });

      // Update local messages
      const updatedMessages = tab.messages.map((m) =>
        m.id === message.id ? { ...m, text: newContent } : m
      );
      updateTabMessages(tab.id, updatedMessages);

    } catch (error) {
      console.error('Failed to close/open message:', error);
      notifications.show({
        title: 'Error',
        message: `Failed to ${isClosed ? 'close' : 'open'} message: ${error}`,
        color: 'red',
      });
    }
  }, [tab.messages, tab.id, updateTabMessages]);

  // Handle saving the new slash command
  const handleSaveSlashCommand = async (command: SlashCommand) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_slash_command', {
        command,
      });

      notifications.show({
        title: 'Success',
        message: `Slash command /${command.name} created successfully!`,
        color: 'green',
      });

      setSlashCommandModalOpened(false);
      setSlashCommandDraft(null);
    } catch (error) {
      console.error('Failed to save slash command:', error);
      notifications.show({
        title: 'Error',
        message: error instanceof Error ? error.message : 'Failed to save slash command',
        color: 'red',
      });
    }
  };

  return (
    <Stack gap="xs" style={{ height: '100%', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
      {/* メッセージエリア */}
      <Box
        style={{ flex: 1, position: 'relative', minHeight: 0 }}
        onDragOver={onDragOver}
        onDragLeave={onDragLeave}
        onDrop={onDrop}
        onMouseEnter={() => setIsMessageAreaHovered(true)}
        onMouseLeave={() => setIsMessageAreaHovered(false)}
        onMouseMove={() => {
          // Show actions on mouse move
          setShowThreadActions(true);

          // Clear existing timeout
          if (hideTimeoutRef.current) {
            clearTimeout(hideTimeoutRef.current);
          }

          // Hide after 3 seconds of no movement
          hideTimeoutRef.current = setTimeout(() => {
            setShowThreadActions(false);
          }, 3000);
        }}
      >
        <ScrollArea h="100%" viewportRef={viewport}>
          <Stack gap="sm" p="md">
            {isActive ? (
              // Active tab: Render full message list
              <MessageList
                messages={tab.messages}
                onSaveMessageToWorkspace={onSaveMessageToWorkspace}
                onExecuteAsTask={onExecuteAsTask}
                onCreateSlashCommand={handleCreateSlashCommand}
                onCreatePersona={handleCreatePersona}
                onRedo={handleRedo}
                onCloseMessage={handleCloseMessage}
                workspaceRootPath={workspace?.rootPath}
              />
            ) : (
              // Inactive tab: Lightweight placeholder to save rendering cost
              <Box p="md" c="dimmed" ta="center">
                <Text size="sm">{tab.messages.length} messages (tab inactive)</Text>
              </Box>
            )}
            {tab.isAiThinking && (activeParticipantIds.length > 0 || tab.isNonInteractiveCommand) && (
              <ThinkingIndicator personaName={tab.thinkingPersona} onCancel={handleCancelOperation} />
            )}
          </Stack>
        </ScrollArea>

        {/* Thread command icons (bottom-right floating) - Show on mouse move, hide after 3s */}
        {tab.messages.length > 0 && isMessageAreaHovered && showThreadActions && (
          <Paper
            shadow="md"
            p={8}
            style={{
              position: 'absolute',
              bottom: 16,
              right: 16,
              zIndex: 100,
              borderRadius: 8,
              transition: 'opacity 0.2s ease-in-out',
            }}
          >
            <Group gap={4}>
              {/* Quick Action Dock */}
              <QuickActionDock
                workspaceId={workspace?.id || null}
                slashCommands={availableSlashCommands}
                onExecuteCommand={handleExecuteQuickAction}
              />

              {/* Agent Configuration */}
              <AgentConfigSelector
                value={agentConfig}
                onChange={setAgentConfig}
              />

              <CopyButton value={getThreadAsText()}>
                {({ copied, copy }) => (
                  <Tooltip label={copied ? 'Copied!' : 'Copy Session'} withArrow>
                    <ActionIcon
                      variant="transparent"
                      onClick={copy}
                      size="lg"
                      style={{
                        color: copied ? 'var(--mantine-color-teal-6)' : 'var(--mantine-color-gray-6)',
                        borderRadius: '6px',
                        transition: 'all 0.15s ease',
                      }}
                      onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                      onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                    >
                      {copied ? <IconCheck size={20} /> : <IconFile size={20} />}
                    </ActionIcon>
                  </Tooltip>
                )}
              </CopyButton>

              <Tooltip label="Generate Summary" withArrow>
                <ActionIcon
                  variant="transparent"
                  onClick={handleGenerateSummary}
                  size="lg"
                  style={{
                    color: 'var(--mantine-color-gray-6)',
                    borderRadius: '6px',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  <IconFileText size={18} />
                </ActionIcon>
              </Tooltip>

              <Tooltip label="Generate ActionPlan" withArrow>
                <ActionIcon
                  variant="transparent"
                  onClick={handleGenerateActionPlan}
                  size="lg"
                  style={{
                    color: 'var(--mantine-color-gray-6)',
                    borderRadius: '6px',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  <IconClipboardList size={18} />
                </ActionIcon>
              </Tooltip>

              <Tooltip label="Generate Expertise" withArrow>
                <ActionIcon
                  variant="transparent"
                  onClick={handleGenerateExpertise}
                  size="lg"
                  style={{
                    color: 'var(--mantine-color-gray-6)',
                    borderRadius: '6px',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  <IconBulb size={18} />
                </ActionIcon>
              </Tooltip>

              <Tooltip label="Generate Concept/Design Issue" withArrow>
                <ActionIcon
                  variant="transparent"
                  onClick={handleGenerateConceptIssue}
                  size="lg"
                  style={{
                    color: 'var(--mantine-color-gray-6)',
                    borderRadius: '6px',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  <IconFileCode size={18} />
                </ActionIcon>
              </Tooltip>

              <Tooltip label="Investigate Workspace" withArrow>
                <ActionIcon
                  variant="transparent"
                  onClick={handleInvestigateWorkspace}
                  size="lg"
                  style={{
                    color: 'var(--mantine-color-gray-6)',
                    borderRadius: '6px',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                  onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                >
                  <IconSearch size={18} />
                </ActionIcon>
              </Tooltip>

              {onSaveSessionToWorkspace && (
                <Tooltip label="Save Session to Workspace" withArrow>
                  <ActionIcon
                    variant="transparent"
                    onClick={onSaveSessionToWorkspace}
                    size="lg"
                    style={{
                      color: 'var(--mantine-color-gray-6)',
                      borderRadius: '6px',
                      transition: 'all 0.15s ease',
                    }}
                    onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--mantine-color-blue-0)'; }}
                    onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
                  >
                    <IconFileExport size={18} />
                  </ActionIcon>
                </Tooltip>
              )}
            </Group>
          </Paper>
        )}

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
              value={activeTabInput}
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
              <Button variant="light" size="sm" component="label" leftSection={<IconPaperclip size={16} />}>
                Attach
                <input type="file" multiple hidden onChange={onFileSelect} />
              </Button>
            </Tooltip>

            {onPasteAndAttach && (
              <Tooltip label="Paste from clipboard & attach">
                <ActionIcon
                  variant="light"
                  size="lg"
                  onClick={() => { void onPasteAndAttach(); }}
                >
                  <IconClipboardList size={18} />
                </ActionIcon>
              </Tooltip>
            )}

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

            <Tooltip label={isMuted ? 'Unmute (Enable AI)' : 'Mute (Disable AI)'}>
              <ActionIcon
                color={isMuted ? 'gray' : 'blue'}
                variant={isMuted ? 'filled' : 'light'}
                onClick={handleToggleMute}
                size="lg"
              >
                {isMuted ? <IconVolumeOff size={20} /> : <IconVolume size={20} />}
              </ActionIcon>
            </Tooltip>

            <Tooltip label={autoMode ? 'Stop AUTO mode' : 'Start AUTO mode'}>
              <ActionIcon
                color={autoMode ? 'red' : 'green'}
                variant={autoMode ? 'filled' : 'light'}
                onClick={handleAutoModeToggle}
                size="lg"
              >
                {autoMode ? <IconPlayerStop size={20} /> : <IconPlayerPlay size={20} />}
              </ActionIcon>
            </Tooltip>
          </Group>
        </Stack>
      </form>

      <StatusBar
        status={{
          ...status,
          mode: tab.isAiThinking ? 'Thinking' : 'Idle',
        }}
        gitInfo={gitInfo}
        participatingAgentsCount={activeParticipantIds.length}
        totalPersonas={personas.length}
        autoMode={autoMode}
        conversationMode={conversationMode}
        talkStyle={talkStyle}
        executionStrategy={executionStrategy}
        contextMode={contextMode}
        sandboxState={sandboxState}
        personas={personas}
        activeParticipantIds={activeParticipantIds}
        dialoguePresets={dialoguePresets}
        onTalkStyleChange={onTalkStyleChange}
        onExecutionStrategyChange={onExecutionStrategyChange}
        onConversationModeChange={onConversationModeChange}
        onContextModeChange={onContextModeChange}
        onToggleParticipant={onToggleParticipant}
        onApplyPreset={onApplyPreset}
        onMentionPersona={onMentionPersona}
        onInvokePersona={onInvokePersona}
      />

      <AutoChatSettingsModal
        opened={autoChatSettingsOpened}
        onClose={() => setAutoChatSettingsOpened(false)}
        config={autoChatConfig}
        onSave={handleSaveAutoChatConfig}
      />

      <SlashCommandEditorModal
        opened={slashCommandModalOpened}
        onClose={() => {
          setSlashCommandModalOpened(false);
          setSlashCommandDraft(null);
        }}
        command={slashCommandDraft}
        onSave={handleSaveSlashCommand}
      />
    </Stack>
  );
});
