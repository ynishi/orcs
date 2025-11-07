import { useState, useRef, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { notifications } from '@mantine/notifications';
import { handleSystemMessage, conversationMessage, commandMessage, shellOutputMessage, MessageSeverity } from './utils/systemMessage';
import {
  Textarea,
  Button,
  ScrollArea,
  Stack,
  Text,
  Container,
  Box,
  Group,
  CopyButton,
  ActionIcon,
  Tooltip,
  AppShell,
  Burger,
  Badge,
  CloseButton,
  Paper,
  Loader,
} from "@mantine/core";
import { useDisclosure } from '@mantine/hooks';
import "./App.css";
import { Message, MessageType } from "./types/message";
import { StatusInfo, getDefaultStatus } from "./types/status";
import { Task } from "./types/task";
import { Agent } from "./types/agent";
import { Session, getMessageCount } from "./types/session";
import { GitInfo } from "./types/git";
import { Navbar } from "./components/navigation/Navbar";
import { WorkspaceSwitcher } from "./components/workspace/WorkspaceSwitcher";
import { SettingsMenu } from "./components/settings/SettingsMenu";
import { parseCommand, isValidCommand, getCommandHelp } from "./utils/commandParser";
import { filterCommandsWithCustom, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention } from "./utils/mentionParser";
import { useSessions } from "./hooks/useSessions";
import { useWorkspace } from "./hooks/useWorkspace";
import { convertSessionToMessages, isIdleMode } from "./types/session";
import { SlashCommand, ExpandedSlashCommand } from "./types/slash_command";
import { useTabContext } from "./context/TabContext";
import { Tabs } from "@mantine/core";
import { ChatPanel } from "./components/chat/ChatPanel";

type InteractionResult =
  | { type: 'NewDialogueMessages'; data: { author: string; content: string }[] }
  | { type: 'NewMessage'; data: string }
  | { type: 'ModeChanged'; data: { [key: string]: any } }
  | { type: 'TasksToDispatch'; data: { tasks: string[] } }
  | { type: 'NoOp' };

function App() {
  // グローバル状態（タブ非依存）
  const [status, setStatus] = useState<StatusInfo>(getDefaultStatus());
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [filteredCommands, setFilteredCommands] = useState<CommandDefinition[]>([]);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(0);
  const [showAgentSuggestions, setShowAgentSuggestions] = useState(false);
  const [filteredAgents, setFilteredAgents] = useState<Agent[]>([]);
  const [selectedAgentIndex, setSelectedAgentIndex] = useState(0);
  const [navbarOpened, { toggle: toggleNavbar }] = useDisclosure(true);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [userNickname, setUserNickname] = useState<string>('You');
  const [gitInfo, setGitInfo] = useState<GitInfo>({
    is_repo: false,
    branch: null,
    repo_name: null,
  });
  const [customCommands, setCustomCommands] = useState<SlashCommand[]>([]);
  const [conversationMode, setConversationMode] = useState<string>('normal');
  const [talkStyle, setTalkStyle] = useState<string | null>(null);
  const [executionStrategy, setExecutionStrategy] = useState<string>('sequential');
  const [personas, setPersonas] = useState<import('./types/agent').PersonaConfig[]>([]);
  const [activeParticipantIds, setActiveParticipantIds] = useState<string[]>([]);

  // セッション管理をカスタムフックに切り替え
  const {
    sessions,
    currentSessionId,
    loading: sessionsLoading,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
    saveCurrentSession,
    refreshSessions,
  } = useSessions();

  // ワークスペース管理
  const { workspace, allWorkspaces, files: workspaceFiles, refresh: refreshWorkspace, refreshWorkspaces, switchWorkspace } = useWorkspace();
  const [includeWorkspaceInPrompt, setIncludeWorkspaceInPrompt] = useState<boolean>(false);

  // タブ管理
  const {
    tabs,
    activeTabId,
    openTab,
    closeTab,
    switchTab: switchToTab,
    updateTabMessages,
    updateTabTitle,
    addMessageToTab,
    updateTabInput,
    updateTabAttachedFiles,
    addAttachedFileToTab,
    removeAttachedFileFromTab,
    setTabDragging,
    setTabThinking,
    getActiveTab,
  } = useTabContext();

  const [autoMode, setAutoMode] = useState<boolean>(false);
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // キーボードショートカット for タブ操作
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      // Cmd/Ctrl + W: 現在のタブを閉じる
      if (modKey && e.key === 'w' && activeTabId) {
        e.preventDefault();
        const activeTab = tabs.find(t => t.id === activeTabId);
        if (activeTab) {
          if (activeTab.isDirty) {
            if (window.confirm(`"${activeTab.title}" has unsaved changes. Close anyway?`)) {
              closeTab(activeTabId);
            }
          } else {
            closeTab(activeTabId);
          }
        }
      }

      // Cmd/Ctrl + Tab: 次のタブ
      if (modKey && e.key === 'Tab' && !e.shiftKey && tabs.length > 1) {
        e.preventDefault();
        const currentIndex = tabs.findIndex(t => t.id === activeTabId);
        const nextIndex = (currentIndex + 1) % tabs.length;
        switchToTab(tabs[nextIndex].id);
      }

      // Cmd/Ctrl + Shift + Tab: 前のタブ
      if (modKey && e.key === 'Tab' && e.shiftKey && tabs.length > 1) {
        e.preventDefault();
        const currentIndex = tabs.findIndex(t => t.id === activeTabId);
        const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
        switchToTab(tabs[prevIndex].id);
      }

      // Cmd/Ctrl + 1-9: n番目のタブ
      if (modKey && e.key >= '1' && e.key <= '9') {
        e.preventDefault();
        const index = parseInt(e.key) - 1;
        if (index < tabs.length) {
          switchToTab(tabs[index].id);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [tabs, activeTabId, closeTab, switchToTab]);

  // Auto-scroll to bottom when active tab's messages change
  useEffect(() => {
    const activeTab = getActiveTab();
    if (viewport.current && activeTab) {
      viewport.current.scrollTo({
        top: viewport.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [tabs, activeTabId, getActiveTab]);

  // Listen for real-time dialogue turn events from backend
  // Use ref to ensure only one listener is registered
  const listenerRegistered = useRef(false);
  const addMessageRef = useRef(addMessage);

  // 最新のaddMessageをrefに保持（クロージャーの問題を回避）
  useEffect(() => {
    addMessageRef.current = addMessage;
  }, [addMessage]);

  useEffect(() => {
    // Skip if listener already registered (prevents duplicate in React Strict Mode)
    if (listenerRegistered.current) {
      console.log('[EFFECT] Listener already registered, skipping');
      return;
    }

    let unlisten: (() => void) | undefined;
    listenerRegistered.current = true;

    console.log('[EFFECT] Setting up dialogue-turn listener');

    const setupListener = async () => {
      unlisten = await listen<{ author: string; content: string }>('dialogue-turn', (event) => {
        console.log('[STREAM] Event received:', event.payload.author);
        console.log('[STREAM] Adding message:', event.payload.author);

        // If author is empty, it's an error message
        if (event.payload.author === '') {
          // 最新のaddMessageを使用
          addMessageRef.current('error', '', event.payload.content);

          // Show error toast
          notifications.show({
            title: 'Agent Error',
            message: event.payload.content,
            color: 'red',
            icon: '❌',
            autoClose: 10000,
          });
        } else {
          // 最新のaddMessageを使用
          addMessageRef.current('ai', event.payload.author, event.payload.content);
        }
      });
      console.log('[EFFECT] Listener setup complete');
    };

    setupListener();

    return () => {
      console.log('[EFFECT] Cleanup: removing listener');
      if (unlisten) {
        unlisten();
      }
    };
  }, []); // 依存配列を空にして、一度だけ登録

  // Load user nickname from backend on startup
  useEffect(() => {
    const loadNickname = async () => {
      try {
        const nickname = await invoke<string>('get_user_nickname');
        setUserNickname(nickname);
      } catch (error) {
        console.error('Failed to load user nickname:', error);
      }
    };
    loadNickname();
  }, []);

  // Load Git repository information on startup
  useEffect(() => {
    const loadGitInfo = async () => {
      try {
        const info = await invoke<GitInfo>('get_git_info');
        setGitInfo(info);
      } catch (error) {
        console.error('Failed to load Git info:', error);
      }
    };
    loadGitInfo();
  }, []);

  // Load conversation mode, talk style, and execution strategy on session change
  useEffect(() => {
    const loadConversationSettings = async () => {
      if (!currentSessionId) return;

      try {
        const mode = await invoke<string>('get_conversation_mode');
        setConversationMode(mode);
      } catch (error) {
        console.error('Failed to load conversation mode:', error);
      }

      try {
        const style = await invoke<string | null>('get_talk_style');
        setTalkStyle(style);
      } catch (error) {
        console.error('Failed to load talk style:', error);
      }

      // Note: execution_strategy is now loaded from Session object in loadActiveSessionMessages effect
    };
    loadConversationSettings();
  }, [currentSessionId]);

  // Load active session messages on startup or when currentSessionId changes
  useEffect(() => {
    const loadActiveSessionMessages = async () => {
      if (!currentSessionId || sessionsLoading) {
        return;
      }

      try {
        console.log('[App] Loading messages for active session:', currentSessionId);
        const activeSession = sessions.find(s => s.id === currentSessionId);
        if (activeSession) {
          // Enrich participant_icons from current personas if missing
          if (!activeSession.participant_icons || Object.keys(activeSession.participant_icons).length === 0) {
            activeSession.participant_icons = {};
            personas.forEach(persona => {
              if (persona.icon && activeSession.participants[persona.id]) {
                activeSession.participant_icons[persona.id] = persona.icon;
              }
            });
          }
          console.log('[App] Session participant_icons:', activeSession.participant_icons);
          const restoredMessages = convertSessionToMessages(activeSession, userNickname);
          
          // 既にタブが開いていない場合のみ、タブとして開く
          const existingTab = tabs.find(tab => tab.sessionId === currentSessionId);
          if (!existingTab) {
            openTab(activeSession, restoredMessages, true);
            console.log('[App] Opened tab for active session with', restoredMessages.length, 'messages');
          } else {
            console.log('[App] Tab already exists for session', currentSessionId);
          }

          // Restore execution strategy from session
          if (activeSession.execution_strategy) {
            setExecutionStrategy(activeSession.execution_strategy);
            console.log('[App] Restored execution strategy:', activeSession.execution_strategy);
          }
        }
      } catch (error) {
        console.error('[App] Failed to load active session messages:', error);
      }
    };

    loadActiveSessionMessages();
  }, [currentSessionId, sessions, sessionsLoading, userNickname, personas, tabs, openTab]);

  const refreshCustomCommands = useCallback(async () => {
    try {
      const commands = await invoke<SlashCommand[]>('list_slash_commands');
      setCustomCommands(commands);
      console.log('[App] Loaded custom commands:', commands.length);
    } catch (error) {
      console.error('Failed to load custom slash commands:', error);
    }
  }, []);

  // Load custom slash commands on startup
  useEffect(() => {
    refreshCustomCommands();
  }, [refreshCustomCommands]);

  // Load personas and active participants
  const refreshPersonas = useCallback(async () => {
    try {
      const personasList = await invoke<import('./types/agent').PersonaConfig[]>('get_personas');
      const activeIds = await invoke<string[]>('get_active_participants');
      setPersonas(personasList);
      setActiveParticipantIds(activeIds);
      // Note: execution_strategy is loaded from Session object, not from backend command
    } catch (error) {
      console.error('Failed to load personas:', error);
    }
  }, []);

  useEffect(() => {
    refreshPersonas();
  }, [refreshPersonas]);

  // Load tasks
  const refreshTasks = useCallback(async () => {
    try {
      const tasksList = await invoke<Task[]>('list_tasks');
      setTasks(tasksList);
      console.log('[App] Loaded tasks:', tasksList.length);
    } catch (error) {
      console.error('Failed to load tasks:', error);
    }
  }, []);

  useEffect(() => {
    refreshTasks();
  }, [refreshTasks]);

  // Listen for task events (real-time task status updates)
  useEffect(() => {
    console.log('[App] Setting up task-event listener');
    const unlisten = listen<any>('task-event', async (event) => {
      console.log('[App] task-event received:', event.payload);
      const payload = event.payload;
      console.log('[App] Event details - target:', payload.target, 'level:', payload.level, 'message:', payload.message);
      console.log('[App] Event fields:', payload.fields);

      // Refresh task list to show updated status
      console.log('[App] Refreshing tasks...');
      await refreshTasks();
      console.log('[App] Tasks refreshed');
    });

    return () => {
      console.log('[App] Cleaning up task-event listener');
      unlisten.then(fn => fn());
    };
  }, [refreshTasks]);

  // Listen for workspace-switched events to refresh workspace data and Git info
  useEffect(() => {
    const unlisten = listen<string>('workspace-switched', async () => {
      console.log('[App] workspace-switched event received, refreshing workspace and Git info');
      console.log('[App] Calling refreshWorkspace...');
      await refreshWorkspace();
      console.log('[App] Calling refreshWorkspaces...');
      await refreshWorkspaces();

      // Refresh session list (workspace-specific sessions)
      console.log('[App] Refreshing sessions...');
      await refreshSessions();

      // Load active session (which should have been switched by the backend)
      try {
        console.log('[App] Loading active session...');
        const activeSession = await invoke<Session | null>('get_active_session');
        if (activeSession) {
          console.log('[App] Active session loaded:', activeSession.id);
          // Switch to this session (this will update currentSessionId and open a tab)
          await switchSession(activeSession.id);
        } else {
          console.log('[App] No active session');
          setTasks([]);
        }
      } catch (error) {
        console.error('[App] Failed to load active session:', error);
      }

      // Reload Git info for the new workspace
      try {
        console.log('[App] Reloading Git info...');
        const info = await invoke<GitInfo>('get_git_info');
        setGitInfo(info);
        console.log('[App] Git info reloaded:', info);
      } catch (error) {
        console.error('[App] Failed to reload Git info:', error);
      }

      console.log('[App] Workspace refresh complete');
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [refreshWorkspace, refreshWorkspaces, refreshSessions]);

  // 入力内容が変更されたときにコマンド/エージェントサジェストを更新
  useEffect(() => {
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const spaceIndex = input.indexOf(' ');
    const isCommandPhase = input.startsWith('/') && (spaceIndex === -1 || cursorPosition <= spaceIndex);

    // コマンドサジェスト（コマンド名入力中のみ表示）
    if (isCommandPhase) {
      const commands = filterCommandsWithCustom(input, customCommands);
      setFilteredCommands(commands);
      setShowSuggestions(commands.length > 0);
      setSelectedSuggestionIndex(0);
      setShowAgentSuggestions(false);
    } else {
      setShowSuggestions(false);
    }

    // エージェントサジェスト（@メンション）
    const mentionFilter = getCurrentMention(input, cursorPosition);

    if (mentionFilter !== null) {
      // Filter personas by name (case-insensitive)
      const filtered: Agent[] = personas
        .filter(p => p.name.toLowerCase().includes(mentionFilter.toLowerCase()))
        .map(p => ({
          id: p.id,
          name: p.name,
          status: activeParticipantIds.includes(p.id) ? 'running' as const : 'idle' as const,
          description: `${p.role} - ${p.background}`,
          isActive: activeParticipantIds.includes(p.id),
        }));
      setFilteredAgents(filtered);
      setShowAgentSuggestions(filtered.length > 0);
      setSelectedAgentIndex(0);
    } else {
      setShowAgentSuggestions(false);
    }
  }, [input, customCommands, personas, activeParticipantIds]);

  // メッセージを追加するヘルパー関数
  const addMessage = useCallback((type: MessageType, author: string, text: string) => {
    // アクティブなタブにメッセージを追加
    if (!activeTabId) return;

    // Find persona by name to get icon and base_color
    const persona = personas.find(p => p.name === author);

    const newMessage: Message = {
      id: `${Date.now()}-${Math.random()}`,
      type,
      author,
      text,
      timestamp: new Date(),
      icon: persona?.icon,
      baseColor: persona?.base_color,
    };
    
    addMessageToTab(activeTabId, newMessage);
  }, [personas, activeTabId, addMessageToTab]);

  const processInput = useCallback(
    async (rawInput: string, attachedFiles: File[] = []) => {
      if (!rawInput.trim() && attachedFiles.length === 0) {
        return;
      }

      const currentFiles = [...attachedFiles];

      const mentions = extractMentions(rawInput);
      if (mentions.length > 0) {
        console.log('[MENTION EVENT] Agents mentioned:', mentions.map(m => m.agentName));
      }

      const parsed = parseCommand(rawInput);
      let backendInput = rawInput;
      let promptCommandExecuted = false;

      if (parsed.isCommand && parsed.command) {
        handleSystemMessage(commandMessage(rawInput), addMessage);

        const isBuiltinCommand = isValidCommand(parsed.command);

        if (isBuiltinCommand) {
          switch (parsed.command) {
            case 'help':
              handleSystemMessage(conversationMessage(getCommandHelp()), addMessage);
              await saveCurrentSession();
              return;
            case 'status':
              handleSystemMessage(conversationMessage(`Connection: ${status.connection}\nTasks: ${status.activeTasks}\nAgent: ${status.currentAgent}\nApp Status: ${status.mode}`), addMessage);
              await saveCurrentSession();
              return;
            case 'task':
              // /task command is deprecated - tasks are now created via 🚀 button
              handleSystemMessage(conversationMessage('Use the 🚀 button on messages to execute them as tasks', 'info'), addMessage);
              await saveCurrentSession();
              return;
            case 'expert':
              // Create adhoc expert persona
              if (parsed.args && parsed.args.length > 0) {
                const expertise = parsed.args.join(' ');
                try {
                  // Show progress in toast (not persisted)
                  notifications.show({
                    title: 'Creating Expert',
                    message: `Generating expert for: ${expertise}...`,
                    color: 'blue',
                    autoClose: false,
                    id: 'expert-creation',
                  });

                  const persona = await invoke<import('./types/agent').PersonaConfig>('create_adhoc_persona', { expertise });

                  // Hide progress notification
                  notifications.hide('expert-creation');

                  // Persist success message to session
                  await invoke('append_system_messages', {
                    messages: [{
                      content: `🔶 Expert persona created: ${persona.name} ${persona.icon || '🔶'}\nRole: ${persona.role}\nBackground: ${persona.background}`,
                      messageType: 'info',
                      severity: 'info',
                    }]
                  });

                  await refreshPersonas();
                  await refreshSessions();
                } catch (error) {
                  console.error('Failed to create expert:', error);
                  notifications.hide('expert-creation');

                  // Persist error message to session
                  await invoke('append_system_messages', {
                    messages: [{
                      content: `❌ Failed to create expert: ${error}`,
                      messageType: 'error',
                      severity: 'error',
                    }]
                  });
                }
              } else {
                handleSystemMessage(conversationMessage('Usage: /expert <expertise>\nExample: /expert 映画制作プロセス', 'error'), addMessage);
              }
              await saveCurrentSession();
              return;
            case 'blueprint':
              // Convert task/discussion into BlueprintWorkflow format
              if (parsed.args && parsed.args.length > 0) {
                const taskDescription = parsed.args.join(' ');
                const blueprintPrompt = `# Task: Create BlueprintWorkflow for ORCS Task Execution

Convert the following into a BlueprintWorkflow format:

${taskDescription}

## Output Format

Provide a BlueprintWorkflow with:

1. **Goal**: Clear, measurable goal statement
2. **Workflow Steps**: Numbered steps with clear deliverables
3. **Output Type**: Classify each step (📋 Clarification, 💡 Proposal, 📝 Documentation, 🔧 Implementation, ✅ Validation)
4. **Dependencies**: Note which steps can run in parallel
5. **Estimated Time**: Total execution time estimate

Example format:
\`\`\`
Goal: [Goal statement]

Workflow:
1. **[Step Name]** (📋 Type): [Description]
2. **[Step Name]** (💡 Type): [Description]
...

Dependencies: 1→2→3 (or note parallel opportunities)
Estimated time: X minutes
\`\`\`

Generate the BlueprintWorkflow now.`;

                // Add blueprint generation request as a user message
                setInput(blueprintPrompt);
                // Trigger send
                setTimeout(() => {
                  const textarea = document.querySelector('textarea');
                  if (textarea) {
                    const event = new KeyboardEvent('keydown', { key: 'Enter', ctrlKey: true });
                    textarea.dispatchEvent(event);
                  }
                }, 100);
              } else {
                handleSystemMessage(conversationMessage('Usage: /blueprint <task description>\nExample: /blueprint Create technical article about Rust', 'error'), addMessage);
              }
              await saveCurrentSession();
              return;
            case 'workspace':
              if (parsed.args && parsed.args.length > 0) {
                const workspaceName = parsed.args.join(' ');
                const targetWorkspace = allWorkspaces.find(ws =>
                  ws.name.toLowerCase() === workspaceName.toLowerCase()
                );
                if (targetWorkspace && currentSessionId) {
                  try {
                    await switchWorkspace(currentSessionId, targetWorkspace.id);
                    handleSystemMessage(conversationMessage(`✅ Switched to workspace: ${targetWorkspace.name}`), addMessage);
                  } catch (err) {
                    handleSystemMessage(conversationMessage(`Failed to switch workspace: ${err}`, 'error'), addMessage);
                  }
                } else if (!targetWorkspace) {
                  handleSystemMessage(conversationMessage(`Workspace not found: ${workspaceName}\n\nAvailable workspaces:\n${allWorkspaces.map(ws => `- ${ws.name}`).join('\n')}`, 'error'), addMessage);
                } else {
                  handleSystemMessage(conversationMessage('No active session', 'error'), addMessage);
                }
              } else {
                const workspaceList = allWorkspaces.map(ws =>
                  `${ws.id === workspace?.id ? '📍' : '  '} ${ws.name}${ws.isFavorite ? ' ⭐' : ''}`
                ).join('\n');
                handleSystemMessage(conversationMessage(`Available workspaces:\n${workspaceList}\n\nUsage: /workspace <name>`), addMessage);
              }
              await saveCurrentSession();
              return;
            case 'files':
              const fileList = workspaceFiles.length > 0
                ? workspaceFiles.map(f => `📄 ${f.name} (${(f.size / 1024).toFixed(2)} KB)${f.author ? ` - by ${f.author}` : ''}`).join('\n')
                : 'No files in current workspace';
              handleSystemMessage(conversationMessage(`Files in workspace "${workspace?.name}":\n${fileList}`), addMessage);
              await saveCurrentSession();
              return;
            case 'mode':
              if (parsed.args && parsed.args.length > 0) {
                const mode = parsed.args[0].toLowerCase();
                const validModes = ['normal', 'concise', 'brief', 'discussion'];

                if (!validModes.includes(mode)) {
                  handleSystemMessage(conversationMessage(`Invalid mode: ${mode}\n\nAvailable modes:\n- normal (通常)\n- concise (簡潔・300文字)\n- brief (極簡潔・150文字)\n- discussion (議論)`, 'error'), addMessage);
                  return;
                }

                try {
                  await invoke('set_conversation_mode', { mode });
                  setConversationMode(mode);
                  const modeLabels: Record<string, string> = {
                    normal: '通常 (Normal)',
                    concise: '簡潔 (300文字)',
                    brief: '極簡潔 (150文字)',
                    discussion: '議論 (Discussion)',
                  };
                  handleSystemMessage(conversationMessage(`✅ Conversation mode changed to: ${modeLabels[mode]}`), addMessage);
                } catch (error) {
                  handleSystemMessage(conversationMessage(`Failed to set conversation mode: ${error}`, 'error'), addMessage);
                }
              } else {
                try {
                  const currentMode = await invoke<string>('get_conversation_mode');
                  const modeLabels: Record<string, string> = {
                    normal: '通常 (Normal)',
                    concise: '簡潔 (300文字)',
                    brief: '極簡潔 (150文字)',
                    discussion: '議論 (Discussion)',
                  };
                  handleSystemMessage(conversationMessage(`Current mode: ${modeLabels[currentMode] || currentMode}\n\nUsage: /mode <normal|concise|brief|discussion>`), addMessage);
                } catch (error) {
                  handleSystemMessage(conversationMessage('Usage: /mode <normal|concise|brief|discussion>', 'error'), addMessage);
                }
              }
              await saveCurrentSession();
              return;
            case 'talk':
              if (parsed.args && parsed.args.length > 0) {
                const style = parsed.args[0].toLowerCase();
                const validStyles = ['brainstorm', 'casual', 'decision_making', 'debate', 'problem_solving', 'review', 'planning', 'none'];

                if (!validStyles.includes(style)) {
                  handleSystemMessage(conversationMessage(`Invalid style: ${style}\n\nAvailable styles:\n- brainstorm (ブレインストーミング)\n- casual (カジュアル)\n- decision_making (意思決定)\n- debate (議論)\n- problem_solving (問題解決)\n- review (レビュー)\n- planning (計画)\n- none (解除)`, 'error'), addMessage);
                  await saveCurrentSession();
                  return;
                }

                try {
                  const styleValue = style === 'none' ? null : style;
                  await invoke('set_talk_style', { style: styleValue });
                  setTalkStyle(styleValue);
                  const styleLabels: Record<string, string> = {
                    brainstorm: 'ブレインストーミング (Brainstorm)',
                    casual: 'カジュアル (Casual)',
                    decision_making: '意思決定 (Decision Making)',
                    debate: '議論 (Debate)',
                    problem_solving: '問題解決 (Problem Solving)',
                    review: 'レビュー (Review)',
                    planning: '計画 (Planning)',
                    none: '解除 (None)',
                  };
                  handleSystemMessage(conversationMessage(`✅ Talk style changed to: ${styleLabels[style]}`), addMessage);
                } catch (error) {
                  handleSystemMessage(conversationMessage(`Failed to set talk style: ${error}`, 'error'), addMessage);
                }
              } else {
                try {
                  const currentStyle = await invoke<string | null>('get_talk_style');
                  const styleLabels: Record<string, string> = {
                    brainstorm: 'ブレインストーミング (Brainstorm)',
                    casual: 'カジュアル (Casual)',
                    decision_making: '意思決定 (Decision Making)',
                    debate: '議論 (Debate)',
                    problem_solving: '問題解決 (Problem Solving)',
                    review: 'レビュー (Review)',
                    planning: '計画 (Planning)',
                  };
                  const currentLabel = currentStyle ? (styleLabels[currentStyle] || currentStyle) : 'Not set';
                  handleSystemMessage(conversationMessage(`Current talk style: ${currentLabel}\n\nUsage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>`), addMessage);
                } catch (error) {
                  handleSystemMessage(conversationMessage('Usage: /talk <brainstorm|casual|decision_making|debate|problem_solving|review|planning|none>', 'error'), addMessage);
                }
              }
              await saveCurrentSession();
              return;
            default:
              break;
          }
        } else {
          const persistedSystemMessages: { content: string; messageType: MessageType; severity?: MessageSeverity }[] = [];
          const persistMessages = async () => {
            if (persistedSystemMessages.length === 0) {
              return;
            }
            const messagesToPersist = [...persistedSystemMessages];
            persistedSystemMessages.length = 0;
            try {
              await invoke('append_system_messages', { messages: messagesToPersist });
            } catch (persistError) {
              console.error('Failed to persist slash command messages:', persistError);
              persistedSystemMessages.unshift(...messagesToPersist);
            }
          };
          const queuePersistedMessage = (
            content: string,
            messageType: MessageType,
            severity?: MessageSeverity
          ) => {
            persistedSystemMessages.push({ content, messageType, severity });
          };

          try {
            const customCommand = await invoke<SlashCommand | null>('get_slash_command', { name: parsed.command });

            if (!customCommand) {
              const messageText = `Unknown command: /${parsed.command}\n\nType /help for available commands.`;
              handleSystemMessage(conversationMessage(messageText, 'error'), addMessage);
              queuePersistedMessage(messageText, 'error', 'error');
              await persistMessages();
              await saveCurrentSession();
              return;
            }

            const argsText = parsed.args?.join(' ') ?? '';
            const expanded = await invoke<ExpandedSlashCommand>('expand_command_template', {
              commandName: customCommand.name,
              args: argsText,
            });

            if (customCommand.type === 'prompt') {
              const messageText = `✨ Executing custom command: /${customCommand.name}`;
              handleSystemMessage(conversationMessage(messageText), addMessage);
              queuePersistedMessage(messageText, 'system');
              backendInput = expanded.content;
              promptCommandExecuted = true;
            } else {
              const executingMessage = `⚡ Executing shell command: /${customCommand.name}`;
              handleSystemMessage(conversationMessage(executingMessage), addMessage);
              queuePersistedMessage(executingMessage, 'command');
              if (expanded.workingDir) {
                const cwdMessage = `(cwd: ${expanded.workingDir})`;
                handleSystemMessage(shellOutputMessage(cwdMessage), addMessage);
                queuePersistedMessage(cwdMessage, 'shell_output');
              }
              const commandLine = `$ ${expanded.content}`;
              handleSystemMessage(shellOutputMessage(commandLine), addMessage);
              queuePersistedMessage(commandLine, 'shell_output');

              try {
                const output = await invoke<string>('execute_shell_command', {
                  command: expanded.content,
                  workingDir: expanded.workingDir ?? null,
                });
                const outputText = output || '(no output)';
                handleSystemMessage(shellOutputMessage(outputText), addMessage);
                queuePersistedMessage(outputText, 'shell_output');
              } catch (shellError) {
                const errorMessage = `Shell command failed: ${shellError}`;
                handleSystemMessage(conversationMessage(errorMessage, 'error'), addMessage);
                queuePersistedMessage(errorMessage, 'error', 'error');
              }
              await persistMessages();
              await saveCurrentSession();
              return;
            }
          } catch (error) {
            console.error('Failed to execute custom command:', error);
            const errorMessage = `Failed to execute command: ${error}`;
            handleSystemMessage(conversationMessage(errorMessage, 'error'), addMessage);
            queuePersistedMessage(errorMessage, 'error', 'error');
            await persistMessages();
            await saveCurrentSession();
            return;
          }

          await persistMessages();
        }
      }

      if (promptCommandExecuted && !backendInput.trim()) {
        handleSystemMessage(conversationMessage(`Command ${rawInput} produced empty content.`, 'error'), addMessage);
        await saveCurrentSession();
        return;
      }

      let messageText = backendInput;

      if (currentFiles.length > 0) {
        const fileInfo = currentFiles.map(f => `📎 ${f.name} (${(f.size / 1024).toFixed(1)} KB)`).join('\n');
        messageText = backendInput ? `${backendInput}\n\n${fileInfo}` : fileInfo;
      }

      if (includeWorkspaceInPrompt && workspaceFiles.length > 0) {
        const uploadedDir = workspace?.workspaceDir
          ? `${workspace.workspaceDir}/resources/uploaded/`
          : '~/.orcs/workspaces/{workspace-id}/resources/uploaded/';

        const workspaceInfo = [
          '',
          '---',
          'Available workspace files:',
          ...workspaceFiles.map(f => `  - ${f.name} (${(f.size / 1024).toFixed(1)} KB)`),
          '',
          `Workspace location: ${uploadedDir}`,
        ].join('\n');
        messageText = messageText + workspaceInfo;
      }

      addMessage('user', userNickname, messageText);

      // アクティブなタブのAI思考状態を設定
      if (activeTabId) {
        setTabThinking(activeTabId, true, 'AI Assistant');
      }

      try {
        // Upload files to workspace and get paths
        const filePaths: string[] = [];
        if (currentFiles.length > 0 && workspace) {
          for (const file of currentFiles) {
            try {
              const arrayBuffer = await file.arrayBuffer();
              const fileData = Array.from(new Uint8Array(arrayBuffer));
              const uploadedFile = await invoke<{ path: string }>("upload_file_from_bytes", {
                workspaceId: workspace.id,
                filename: file.name,
                fileData: fileData,
                sessionId: currentSessionId || null,
                messageTimestamp: null,
                author: null,
              });
              filePaths.push(uploadedFile.path);
              console.log('[FILE] Uploaded file:', file.name, 'to', uploadedFile.path);
            } catch (uploadError) {
              console.error('[FILE] Failed to upload file:', file.name, uploadError);
              addMessage('error', 'System', `Failed to upload file ${file.name}: ${uploadError}`);
            }
          }
        }

        const result = await invoke<InteractionResult>("handle_input", {
          input: backendInput,
          filePaths: filePaths.length > 0 ? filePaths : null,
        });

        if (result.type === 'NewDialogueMessages') {
          console.log('[BATCH] Received', result.data.length, 'messages (already streamed)');
          // Note: Errors are also returned as NewDialogueMessages (empty array) after streaming
        } else if (result.type === 'NewMessage') {
          // This should not happen anymore, but keep for backward compatibility
          console.error('[ERROR] Backend returned error:', result.data);
          addMessage('error', '', result.data);

          notifications.show({
            title: 'Agent Error',
            message: result.data,
            color: 'red',
            icon: '❌',
            autoClose: 10000,
          });
        }

        await saveCurrentSession();
      } catch (error) {
        console.error("Error calling backend:", error);
        addMessage('error', 'System', `Error: ${error}`);
      } finally {
        // アクティブなタブのAI思考状態を解除
        if (activeTabId) {
          setTabThinking(activeTabId, false);
        }
      }
    },
    [
      addMessage,
      allWorkspaces,
      currentSessionId,
      getCommandHelp,
      includeWorkspaceInPrompt,
      invoke,
      saveCurrentSession,
      setTabThinking,
      activeTabId,
      setStatus,
      setTasks,
      status.activeTasks,
      status.connection,
      status.currentAgent,
      status.mode,
      switchWorkspace,
      userNickname,
      workspace,
      workspaceFiles,
    ]
  );

  // スレッド全体をテキストとして取得
  const getThreadAsText = () => {
    // アクティブなタブのメッセージを取得
    const activeTab = getActiveTab();
    if (!activeTab) return '';
    
    return activeTab.messages
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  };

  // キーボードイベントハンドラー
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Cmd+Enter または Ctrl+Enter で送信
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSubmit(e as any);
      return;
    }

    // エージェントサジェスト表示中のキーボード操作
    if (showAgentSuggestions) {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          setSelectedAgentIndex((prev) =>
            prev > 0 ? prev - 1 : filteredAgents.length - 1
          );
          break;
        case 'ArrowDown':
          e.preventDefault();
          setSelectedAgentIndex((prev) =>
            prev < filteredAgents.length - 1 ? prev + 1 : 0
          );
          break;
        case 'Tab':
          e.preventDefault();
          selectAgent(filteredAgents[selectedAgentIndex]);
          break;
        case 'Enter':
          if (!e.shiftKey && !e.metaKey && !e.ctrlKey) {
            e.preventDefault();
            selectAgent(filteredAgents[selectedAgentIndex]);
          }
          break;
        case 'Escape':
          e.preventDefault();
          setShowAgentSuggestions(false);
          break;
      }
      return;
    }

    // コマンドサジェスト表示中のキーボード操作
    if (showSuggestions) {
      switch (e.key) {
        case 'ArrowUp':
          e.preventDefault();
          setSelectedSuggestionIndex((prev) =>
            prev > 0 ? prev - 1 : filteredCommands.length - 1
          );
          break;
        case 'ArrowDown':
          e.preventDefault();
          setSelectedSuggestionIndex((prev) =>
            prev < filteredCommands.length - 1 ? prev + 1 : 0
          );
          break;
        case 'Tab':
          e.preventDefault();
          selectCommand(filteredCommands[selectedSuggestionIndex]);
          break;
        case 'Enter':
          if (!e.shiftKey && !e.metaKey && !e.ctrlKey) {
            e.preventDefault();
            selectCommand(filteredCommands[selectedSuggestionIndex]);
          }
          break;
        case 'Escape':
          e.preventDefault();
          setShowSuggestions(false);
          break;
      }
      return;
    }
  };

  // コマンドを選択
  const selectCommand = (command: CommandDefinition) => {
    setInput(`/${command.name} `);
    setShowSuggestions(false);
    textareaRef.current?.focus();
  };

  // エージェントを選択
  const selectAgent = (agent: Agent) => {
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const beforeCursor = input.slice(0, cursorPosition);
    const afterCursor = input.slice(cursorPosition);
    const lastAtIndex = beforeCursor.lastIndexOf('@');

    if (lastAtIndex !== -1) {
      const newInput = beforeCursor.slice(0, lastAtIndex) + `@${agent.name} ` + afterCursor;
      setInput(newInput);
    }

    setShowAgentSuggestions(false);
    textareaRef.current?.focus();
  };

  // ドラッグ&ドロップハンドラー
  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (activeTabId) {
      setTabDragging(activeTabId, true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (activeTabId) {
      setTabDragging(activeTabId, false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    if (!activeTabId) return;
    setTabDragging(activeTabId, false);

    const files = Array.from(e.dataTransfer.files);

    if (files.length > 0) {
      const activeTab = getActiveTab();
      if (activeTab) {
        updateTabAttachedFiles(activeTabId, [...activeTab.attachedFiles, ...files]);
        addMessage('system', 'System', `📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`);
      }
    }
  };

  const removeAttachedFile = (index: number) => {
    if (activeTabId) {
      removeAttachedFileFromTab(activeTabId, index);
    }
  };

  // ファイル選択ボタンのハンドラー
  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []);
    if (files.length > 0 && activeTabId) {
      const activeTab = getActiveTab();
      if (activeTab) {
        updateTabAttachedFiles(activeTabId, [...activeTab.attachedFiles, ...files]);
        addMessage('system', 'System', `📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`);
      }
    }
  };

  // Workspace からファイルをアタッチするハンドラー
  const handleAttachFileFromWorkspace = (file: File) => {
    if (!activeTabId) return;
    
    addAttachedFileToTab(activeTabId, file);

    // Show toast notification instead of adding to chat history
    notifications.show({
      title: 'File Attached',
      message: `${file.name} from workspace`,
      color: 'blue',
      icon: '📎',
    });
  };

  // ワークスペースファイルからセッションに移動するハンドラー
  const handleGoToSessionFromFile = (sessionId: string) => {
    const session = sessions.find(s => s.id === sessionId);
    if (session) {
      handleSessionSelect(session);
    } else {
      addMessage('error', 'System', `Session not found: ${sessionId}`);
    }
  };

  // メッセージをワークスペースに保存するハンドラー
  const handleSaveMessageToWorkspace = async (message: Message) => {
    try {
      // ファイル名を生成（タイムスタンプ + 作者名）
      const timestamp = message.timestamp.toISOString().replace(/[:.]/g, '-');
      const filename = `${timestamp}_${message.author}_${message.type}.txt`;

      // メッセージテキストをバイト配列に変換
      const encoder = new TextEncoder();
      const data = encoder.encode(message.text);
      const fileData = Array.from(data);

      // ワークスペースIDを取得
      const workspace = await invoke<{ id: string }>('get_current_workspace');

      // ワークスペースに保存（セッションID、メッセージタイムスタンプ、作者を含める）
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: currentSessionId || null,
        messageTimestamp: message.timestamp.toISOString(),
        author: message.author,
      });

      // ワークスペースのファイルリストを更新
      await refreshWorkspace();

      // Toast notification instead of system message
      notifications.show({
        title: 'File saved',
        message: `${filename}`,
        color: 'green',
        icon: '💾',
      });
    } catch (err) {
      console.error('Failed to save message to workspace:', err);
      notifications.show({
        title: 'Failed to save message',
        message: String(err),
        color: 'red',
      });
    }
  };

  // Task実行ハンドラー
  const handleExecuteAsTask = async (message: Message) => {
    try {
      addMessage('system', 'SYSTEM', `🚀 Executing task: "${message.text.slice(0, 50)}..."`);

      // TODO: Backend command implementation
      const result = await invoke<string>('execute_message_as_task', {
        messageContent: message.text,
      });

      addMessage('system', 'SYSTEM', `✅ Task completed: ${result}`);

      notifications.show({
        title: 'Task Executed',
        message: 'Task execution completed successfully',
        color: 'green',
        icon: '✅',
      });
    } catch (err) {
      console.error('Failed to execute task:', err);
      addMessage('error', '', `❌ Task execution failed: ${String(err)}`);

      notifications.show({
        title: 'Task Execution Failed',
        message: String(err),
        color: 'red',
        icon: '❌',
      });
    }
  };

  // タスク操作ハンドラー
  const handleTaskToggle = async (taskId: string) => {
    // Tasks are managed by backend - toggle is not supported for execution tasks
    // This is kept for compatibility but does nothing
    console.log('[App] Task toggle not supported for execution tasks:', taskId);
  };

  const handleTaskDelete = async (taskId: string) => {
    // Delete task from backend
    try {
      await invoke('delete_task', { taskId });
      await refreshTasks();
      notifications.show({
        title: 'Task Deleted',
        message: 'Task has been removed',
        color: 'blue',
        autoClose: 2000,
      });
    } catch (error) {
      console.error('[App] Failed to delete task:', error);
      notifications.show({
        title: 'Failed to Delete Task',
        message: String(error),
        color: 'red',
      });
    }
  };

  // セッション操作ハンドラー（タブ対応版）
  const handleSessionSelect = async (session: Session) => {
    try {
      // セッションを切り替え（バックエンドで履歴付きSessionDataを取得）
      const fullSession = await switchSession(session.id);

      // メッセージ履歴を復元
      const restoredMessages = convertSessionToMessages(fullSession, userNickname);

      // タブを開く（既に開いていればフォーカス）
      const tabId = openTab(fullSession, restoredMessages);

      // Show toast notification
      notifications.show({
        title: 'Session Opened',
        message: `${session.title} (${restoredMessages.length} messages)`,
        color: 'blue',
        icon: '📂',
      });
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to switch session: ${err}`,
        color: 'red',
      });
    }
  };

  const handleSessionDelete = async (sessionId: string) => {
    try {
      await deleteSession(sessionId);

      // タブも閉じる
      const tab = tabs.find(t => t.sessionId === sessionId);
      if (tab) {
        closeTab(tab.id);
      }

      // Show toast notification
      notifications.show({
        title: 'Session Deleted',
        message: 'The session has been removed',
        color: 'red',
        icon: '🗑️',
      });
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to delete session: ${err}`,
        color: 'red',
      });
    }
  };

  const handleSessionRename = async (sessionId: string, newTitle: string) => {
    try {
      await renameSession(sessionId, newTitle);

      // タブのタイトルも更新
      const tab = tabs.find(t => t.sessionId === sessionId);
      if (tab) {
        updateTabTitle(tab.id, newTitle);
      }
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to rename session: ${err}`,
        color: 'red',
      });
    }
  };

  const handleNewSession = async () => {
    try {
      const newSessionId = await createSession();
      // 新しいセッションは自動的にタブとして開かれる（loadActiveSessionMessagesのuseEffectで）
      // Show toast notification
      notifications.show({
        title: 'New Session Created',
        message: 'Started a fresh conversation',
        color: 'green',
        icon: '✨',
      });
    } catch (err) {
      addMessage('error', 'System', `Failed to create session: ${err}`);
    }
  };

  const handleConversationModeChange = (mode: string) => {
    setConversationMode(mode);
  };

  const handleTalkStyleChange = (style: string | null) => {
    setTalkStyle(style);
  };

  const handleStrategyChange = (strategy: string) => {
    setExecutionStrategy(strategy);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // アクティブなタブの状態を取得
    const activeTab = getActiveTab();
    if (!activeTab) return;

    if (!activeTab.input.trim() && activeTab.attachedFiles.length === 0) {
      return;
    }

    const currentInput = activeTab.input;
    const currentFiles = [...activeTab.attachedFiles];

    // Check for @mentions and auto-add inactive personas
    const mentions = extractMentions(currentInput);
    for (const mention of mentions) {
      const persona = personas.find(p => p.name === mention.agentName);
      if (persona && !activeParticipantIds.includes(persona.id)) {
        try {
          await invoke('add_participant', { personaId: persona.id });
          addMessage('system', 'System', `${persona.name} が参加しました`);
          // Refresh participants list and sessions to update participant_icons/colors
          await refreshPersonas();
          await refreshSessions();
        } catch (error) {
          console.error(`Failed to add participant ${persona.name}:`, error);
        }
      }
    }

    // アクティブなタブの入力状態をクリア
    updateTabInput(activeTab.id, "");
    updateTabAttachedFiles(activeTab.id, []);
    setShowSuggestions(false);
    setShowAgentSuggestions(false);
    await processInput(currentInput, currentFiles);
  };

  const handleRunSlashCommand = useCallback(
    async (command: SlashCommand, args: string) => {
      setShowSuggestions(false);
      setShowAgentSuggestions(false);
      setInput('');
      const trimmedArgs = args.trim();
      const commandInput = trimmedArgs ? `/${command.name} ${trimmedArgs}` : `/${command.name}`;
      await processInput(commandInput);
    },
    [processInput]
  );

  // セッションローディング中の表示
  if (sessionsLoading) {
    return (
      <Container size="md" h="100vh" style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <Stack align="center" gap="md">
          <Loader size="lg" />
          <Text>Loading sessions...</Text>
        </Stack>
      </Container>
    );
  }

  return (
    <AppShell
      navbar={{
        width: 280,
        breakpoint: 'sm',
        collapsed: { mobile: !navbarOpened, desktop: !navbarOpened },
      }}
      padding="md"
    >
      {/* 左ペイン */}
      <AppShell.Navbar>
        <Navbar
          sessions={sessions}
          currentSessionId={currentSessionId}
          currentWorkspaceId={workspace?.id}
          onSessionSelect={handleSessionSelect}
          onSessionDelete={handleSessionDelete}
          onSessionRename={handleSessionRename}
          onNewSession={handleNewSession}
          tasks={tasks}
          onTaskToggle={handleTaskToggle}
          onTaskDelete={handleTaskDelete}
          onRefreshTasks={refreshTasks}
          onAttachFile={handleAttachFileFromWorkspace}
          includeWorkspaceInPrompt={includeWorkspaceInPrompt}
          onToggleIncludeWorkspaceInPrompt={setIncludeWorkspaceInPrompt}
          onGoToSession={handleGoToSessionFromFile}
          onRefreshWorkspace={refreshWorkspace}
          onMessage={addMessage}
          onSlashCommandsUpdated={refreshCustomCommands}
          onRunSlashCommand={handleRunSlashCommand}
          onConversationModeChange={handleConversationModeChange}
          onTalkStyleChange={handleTalkStyleChange}
          onStrategyChange={handleStrategyChange}
          personas={personas}
          activeParticipantIds={activeParticipantIds}
          executionStrategy={executionStrategy}
          onRefreshPersonas={refreshPersonas}
          onRefreshSessions={refreshSessions}
        />
      </AppShell.Navbar>

      {/* メインコンテンツ */}
      <AppShell.Main>
        <Container size="md" h="100vh" p="md" style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <Stack style={{ flex: 1, minHeight: 0 }} gap="md">
            <Group gap="sm" justify="space-between">
              <Group gap="sm">
                <Burger opened={navbarOpened} onClick={toggleNavbar} size="sm" />
                <Text size="xl" fw={700}>ORCS</Text>
              </Group>
              <Group gap="md">
                {/* Workspace Switcher */}
                <Group gap="xs">
                  <WorkspaceSwitcher sessionId={currentSessionId} />
                  {workspace && (
                    <>
                      <Text size="sm" c="dimmed">Workspace:</Text>
                      <Badge size="sm" variant="dot" color="green">
                        {workspace.name}
                      </Badge>
                    </>
                  )}
                </Group>

                {/* Session Info */}
                {currentSessionId && (
                  <Group gap="xs">
                    <Text size="sm" c="dimmed">Session:</Text>
                    <Badge size="lg" variant="light">
                      {sessions.find(s => s.id === currentSessionId)?.title || 'Untitled'}
                    </Badge>
                    <Badge size="sm" color="gray" variant="outline">
                      {getMessageCount(sessions.find(s => s.id === currentSessionId)!) || 0} msgs
                    </Badge>
                  </Group>
                )}

                {/* Settings Menu */}
                <SettingsMenu
                  onSelectSession={handleSessionSelect}
                />
              </Group>
            </Group>

            {/* タブ領域 */}
            {tabs.length === 0 ? (
              <Box style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <Stack align="center" gap="md">
                  <Text size="xl" c="dimmed">No session opened</Text>
                  <Text size="sm" c="dimmed">Select a session from the sidebar to start chatting</Text>
                </Stack>
              </Box>
            ) : (
              <Tabs
                value={activeTabId}
                onChange={async (value) => {
                  if (!value) return;
                  
                  // タブを切り替え
                  switchToTab(value);
                  
                  // バックエンドのセッションも切り替え
                  const tab = tabs.find(t => t.id === value);
                  if (tab) {
                    try {
                      await switchSession(tab.sessionId);
                    } catch (err) {
                      console.error('Failed to switch backend session:', err);
                    }
                  }
                }}
                style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}
              >
                <Tabs.List style={{ overflowX: 'auto', flexWrap: 'nowrap' }}>
                  {tabs.map((tab) => (
                    <Tabs.Tab
                      key={tab.id}
                      value={tab.id}
                      style={{
                        minWidth: '120px',
                        maxWidth: '200px',
                      }}
                      leftSection={tab.isDirty ? '●' : undefined}
                      rightSection={
                        <CloseButton
                          size="xs"
                          onClick={(e) => {
                            e.stopPropagation();
                            // 未保存の場合は確認
                            if (tab.isDirty) {
                              if (window.confirm(`"${tab.title}" has unsaved changes. Close anyway?`)) {
                                closeTab(tab.id);
                              }
                            } else {
                              closeTab(tab.id);
                            }
                          }}
                        />
                      }
                    >
                      <Text truncate style={{ maxWidth: '100%' }}>
                        {tab.title}
                      </Text>
                    </Tabs.Tab>
                  ))}
                </Tabs.List>

                {tabs.map((tab) => (
                  <Tabs.Panel key={tab.id} value={tab.id} style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
                    <ChatPanel
                      tab={tab}
                      status={status}
                      userNickname={userNickname}
                      gitInfo={gitInfo}
                      autoMode={autoMode}
                      conversationMode={conversationMode}
                      talkStyle={talkStyle}
                      executionStrategy={executionStrategy}
                      personas={personas}
                      activeParticipantIds={activeParticipantIds}
                      workspace={workspace}
                      showSuggestions={showSuggestions}
                      filteredCommands={filteredCommands}
                      selectedSuggestionIndex={selectedSuggestionIndex}
                      showAgentSuggestions={showAgentSuggestions}
                      filteredAgents={filteredAgents}
                      selectedAgentIndex={selectedAgentIndex}
                      onSubmit={handleSubmit}
                      onInputChange={(value) => {
                        if (activeTabId) {
                          updateTabInput(activeTabId, value);
                        }
                      }}
                      onKeyDown={handleKeyDown}
                      onFileSelect={handleFileSelect}
                      onRemoveFile={removeAttachedFile}
                      onDragOver={handleDragOver}
                      onDragLeave={handleDragLeave}
                      onDrop={handleDrop}
                      onSaveMessageToWorkspace={handleSaveMessageToWorkspace}
                      onExecuteAsTask={handleExecuteAsTask}
                      onAutoModeChange={setAutoMode}
                      onSelectCommand={selectCommand}
                      onSelectAgent={selectAgent}
                      onHoverSuggestion={setSelectedSuggestionIndex}
                    />
                  </Tabs.Panel>
                ))}
              </Tabs>
            )}
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
