import { useState, useRef, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { notifications } from '@mantine/notifications';
import {
  Stack,
  Text,
  Container,
  Box,
  Group,
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
import { parseCommand } from "./utils/commandParser";
import { filterCommandsWithCustom, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention } from "./utils/mentionParser";
import { useSessions } from "./hooks/useSessions";
import { useWorkspace } from "./hooks/useWorkspace";
import { convertSessionToMessages } from "./types/session";
import { SlashCommand } from "./types/slash_command";
import { useTabContext } from "./context/TabContext";
import { useSlashCommands } from "./hooks/useSlashCommands";
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
  const [userProfile, setUserProfile] = useState<{ nickname: string; background: string } | null>(null);
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
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { workspace, allWorkspaces, files: workspaceFiles, refresh: refreshWorkspace, refreshWorkspaces, switchWorkspace: switchWorkspaceBackend } = useWorkspace();
  const [includeWorkspaceInPrompt, setIncludeWorkspaceInPrompt] = useState<boolean>(false);

  // タブ管理
  const {
    tabs,
    activeTabId,
    openTab,
    closeTab,
    switchTab: switchToTab,
    switchWorkspace: switchWorkspaceTabs,
    updateTabTitle,
    addMessageToTab,
    updateTabInput,
    updateTabAttachedFiles,
    addAttachedFileToTab,
    removeAttachedFileFromTab,
    setTabDragging,
    setTabThinking,
    getActiveTab,
    getVisibleTabs,
    getTabBySessionId,
  } = useTabContext();

  const [autoMode, setAutoMode] = useState<boolean>(false);
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const workspaceSwitchingRef = useRef(false);

  // メッセージを追加するヘルパー関数（early definition for useRef/useSlashCommands）
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

  // Load user profile from backend on startup
  useEffect(() => {
    const loadUserProfile = async () => {
      try {
        const profile = await invoke<{ nickname: string; background: string }>('get_user_profile');
        setUserProfile(profile);
        setUserNickname(profile.nickname);
      } catch (error) {
        console.error('Failed to load user profile:', error);
        // Fallback to nickname-only API
        try {
          const nickname = await invoke<string>('get_user_nickname');
          setUserNickname(nickname);
          setUserProfile({ nickname, background: '' });
        } catch (nicknameError) {
          console.error('Failed to load user nickname:', nicknameError);
        }
      }
    };
    loadUserProfile();
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
          const existingTab = getTabBySessionId(currentSessionId);
          if (!existingTab && workspace) {
            openTab(activeSession, restoredMessages, workspace.id, true);
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
  }, [currentSessionId, sessions, sessionsLoading, userNickname, personas, workspace, openTab, getTabBySessionId]);

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
    // セッションがない場合はスキップ（バックエンドが"No active session"エラーを返すため）
    if (!currentSessionId) {
      console.log('[refreshPersonas] No active session, skipping');
      return;
    }

    try {
      const personasList = await invoke<import('./types/agent').PersonaConfig[]>('get_personas');
      const activeIds = await invoke<string[]>('get_active_participants');
      setPersonas(personasList);
      setActiveParticipantIds(activeIds);
      // Note: execution_strategy is loaded from Session object, not from backend command
    } catch (error) {
      console.error('Failed to load personas:', error);
    }
  }, [currentSessionId]);

  // セッションが変わったら persona を再読み込み
  useEffect(() => {
    if (currentSessionId) {
    refreshPersonas();
    }
  }, [currentSessionId, refreshPersonas]);

  // 初回セッション自動作成（Workspace がある場合のみ）
  useEffect(() => {
    const initializeSession = async () => {
      // ローディング中はスキップ
      if (sessionsLoading) return;
      
      // Workspace があるが Session がない場合に自動作成
      if (workspace && sessions.length === 0) {
        console.log('[App] No sessions found, creating initial session for workspace');
        try {
          await createSession();
          console.log('[App] Initial session created');
        } catch (error) {
          console.error('[App] Failed to create initial session:', error);
        }
      }
    };
    
    initializeSession();
  }, [sessionsLoading, workspace, sessions.length, createSession]);

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
    const unlistenPromise = listen<string>('workspace-switched', async () => {
      if (workspaceSwitchingRef.current) {
        console.log('[App] workspace-switched event ignored (refresh already in progress)');
        return;
      }
      workspaceSwitchingRef.current = true;

      try {
        console.log('[App] workspace-switched event received, refreshing workspace and Git info');
        console.log('[App] Calling refreshWorkspace...');
        await refreshWorkspace();
        console.log('[App] Calling refreshWorkspaces...');
        await refreshWorkspaces();

        // Refresh session list (workspace-specific sessions)
        console.log('[App] Refreshing sessions...');
        await refreshSessions();

        // Get the updated workspace
        const updatedWorkspace = await invoke<any>('get_current_workspace');
        
        if (updatedWorkspace) {
          console.log('[App] Switching to workspace tabs:', updatedWorkspace.id);
          // Workspace切り替え：既存タブがあればフォーカス、なければnull
          switchWorkspaceTabs(updatedWorkspace.id);
        }

        // Load active session (which should have been switched by the backend)
        try {
          console.log('[App] Loading active session...');
          const activeSession = await invoke<Session | null>('get_active_session');
          if (activeSession && updatedWorkspace) {
            console.log('[App] Active session loaded:', activeSession.id);
            
            // 既にタブが開いているかチェック
            const existingTab = getTabBySessionId(activeSession.id);
            if (!existingTab) {
              // タブがなければ開く
              const restoredMessages = convertSessionToMessages(activeSession, userNickname);
              openTab(activeSession, restoredMessages, updatedWorkspace.id, true);
              console.log('[App] Opened tab for active session after workspace switch');
            } else {
              // 既にタブがあればフォーカス
              switchToTab(existingTab.id);
              console.log('[App] Focused existing tab for active session');
            }
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
      } finally {
        workspaceSwitchingRef.current = false;
      }
    });

    return () => {
      unlistenPromise.then(fn => fn());
    };
  }, [refreshWorkspace, refreshWorkspaces, refreshSessions, switchWorkspaceTabs, openTab, switchToTab, getTabBySessionId, userNickname]);

  // 入力内容が変更されたときにコマンド/エージェントサジェストを更新
  useEffect(() => {
    const activeTab = getActiveTab();
    const input = activeTab?.input || '';
    
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
  }, [tabs, activeTabId, getActiveTab, customCommands, personas, activeParticipantIds]);

  // SlashCommand処理（addMessage, refreshPersonasの定義後に配置）
  const { handleSlashCommand } = useSlashCommands({
    addMessage,
    saveCurrentSession,
    status,
    currentSessionId,
    workspace,
    allWorkspaces,
    workspaceFiles,
    switchWorkspace: switchWorkspaceBackend,
    setConversationMode,
    setTalkStyle,
    setInput: (value) => {
      if (activeTabId) {
        updateTabInput(activeTabId, value);
      }
    },
    refreshPersonas,
    refreshSessions,
  });

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

      // SlashCommandの処理（分離済み）
      const parsed = parseCommand(rawInput);
      let backendInput = rawInput;
      let promptCommandExecuted = false;

      if (parsed.isCommand && parsed.command) {
        promptCommandExecuted = await handleSlashCommand(rawInput);
        
        // SlashCommandの処理が完了
        // promptCommandがない場合（組み込みコマンド）は戻ってこない（handleSlashCommand内でreturn済み）
        // promptCommandがある場合のみ続行
        if (!promptCommandExecuted) {
          // 組み込みコマンドはhandleSlashCommand内で処理完了しているのでここには来ない
              return;
        }
      }

      if (promptCommandExecuted && !backendInput.trim()) {
        addMessage('error', 'System', `Command ${rawInput} produced empty content.`);
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
      switchWorkspaceBackend,
      userNickname,
      workspace,
      workspaceFiles,
    ]
  );

  // スレッド全体をテキストとして取得（将来の機能用に保持）
  // const getThreadAsText = () => {
  //   // アクティブなタブのメッセージを取得
  //   const activeTab = getActiveTab();
  //   if (!activeTab) return '';
  //   
  //   return activeTab.messages
  //     .map((msg) => {
  //       const time = msg.timestamp.toLocaleString();
  //       return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
  //     })
  //     .join('\n---\n\n');
  // };

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
    if (!activeTabId) return;
    updateTabInput(activeTabId, `/${command.name} `);
    setShowSuggestions(false);
    textareaRef.current?.focus();
  };

  // エージェントを選択
  const selectAgent = (agent: Agent) => {
    if (!activeTabId) return;
    const activeTab = getActiveTab();
    if (!activeTab) return;
    
    const input = activeTab.input;
    const cursorPosition = textareaRef.current?.selectionStart || input.length;
    const beforeCursor = input.slice(0, cursorPosition);
    const afterCursor = input.slice(cursorPosition);
    const lastAtIndex = beforeCursor.lastIndexOf('@');

    if (lastAtIndex !== -1) {
      const newInput = beforeCursor.slice(0, lastAtIndex) + `@${agent.name} ` + afterCursor;
      updateTabInput(activeTabId, newInput);
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
      console.log('[App] Session selected:', {
        sessionId: session.id.substring(0, 8),
        workspaceId: session.workspace_id.substring(0, 8),
        currentWorkspace: workspace?.id.substring(0, 8),
      });

      // 1. Workspace切り替え（必要なら）
      if (session.workspace_id !== workspace?.id) {
        console.log('[App] Switching workspace for session...');
        await switchWorkspaceBackend(session.id, session.workspace_id);
        // ↑ 'workspace-switched' イベント発火 → 既存リスナーで全体同期
      }

      // 2. セッションを切り替え（バックエンドで履歴付きSessionDataを取得）
      const fullSession = await switchSession(session.id);

      // 3. メッセージ履歴を復元
      const restoredMessages = convertSessionToMessages(fullSession, userNickname);

      // 4. タブを開く（session.workspace_idを使用）
      openTab(fullSession, restoredMessages, session.workspace_id);

      // Show toast notification
      notifications.show({
        title: 'Session Opened',
        message: `${session.title} (${restoredMessages.length} messages)`,
        color: 'blue',
        icon: '📂',
      });
    } catch (err) {
      console.error('[App] Failed to select session:', err);
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

  const handleToggleFavorite = async (sessionId: string) => {
    try {
      await invoke('toggle_session_favorite', { sessionId });
      await refreshSessions();
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to toggle favorite: ${err}`,
        color: 'red',
      });
    }
  };

  const handleToggleArchive = async (sessionId: string) => {
    try {
      await invoke('toggle_session_archive', { sessionId });
      await refreshSessions();
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to toggle archive: ${err}`,
        color: 'red',
      });
    }
  };

  const handleMoveSortOrder = async (sessionId: string, direction: 'up' | 'down') => {
    try {
      // Get current session list (filtered to favorites only)
      const favoriteSessions = sessions
        .filter(s => s.is_favorite && !s.is_archived)
        .sort((a, b) => {
          if (a.sort_order !== undefined && b.sort_order !== undefined) {
            return a.sort_order - b.sort_order;
          }
          if (a.sort_order !== undefined) return -1;
          if (b.sort_order !== undefined) return 1;
          return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
        });

      const currentIndex = favoriteSessions.findIndex(s => s.id === sessionId);
      if (currentIndex === -1) return;

      const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1;
      if (targetIndex < 0 || targetIndex >= favoriteSessions.length) return;

      // Reassign sort_order values
      const updates: Promise<void>[] = [];
      favoriteSessions.forEach((session, index) => {
        let newSortOrder: number;
        if (index === currentIndex) {
          newSortOrder = targetIndex;
        } else if (index === targetIndex) {
          newSortOrder = currentIndex;
        } else {
          newSortOrder = index;
        }
        updates.push(
          invoke('update_session_sort_order', { sessionId: session.id, sortOrder: newSortOrder })
        );
      });

      await Promise.all(updates);
      await refreshSessions();
    } catch (err) {
      notifications.show({
        title: 'Error',
        message: `Failed to update sort order: ${err}`,
        color: 'red',
      });
    }
  };

  const handleNewSession = async () => {
    try {
      await createSession();
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
      if (activeTabId) {
        updateTabInput(activeTabId, '');
      }
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
          workspaces={allWorkspaces}
          onSessionSelect={handleSessionSelect}
          onSessionDelete={handleSessionDelete}
          onSessionRename={handleSessionRename}
          onToggleFavorite={handleToggleFavorite}
          onToggleArchive={handleToggleArchive}
          onMoveSortOrder={handleMoveSortOrder}
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
                {/* User Profile */}
                {userProfile && (
                  <Group gap="xs">
                    <Text size="sm" c="dimmed">User:</Text>
                    <Badge size="sm" variant="light" color="blue">
                      {userProfile.nickname}
                    </Badge>
                  </Group>
                )}

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
                {!workspace ? (
                  // Workspace がない場合
                  <Paper p="xl" withBorder shadow="sm" style={{ maxWidth: 500 }}>
                    <Stack align="center" gap="md">
                      <Text size="xl" fw={700}>👋 Welcome to ORCS!</Text>
                      <Text c="dimmed" ta="center" size="sm">
                        右上のフォルダーアイコンからワークスペース（作業ディレクトリ）を開いてください
                  </Text>
                    </Stack>
                </Paper>
                ) : (
                  // Workspace はあるが Session がない場合
                  <Stack align="center" gap="md">
                    <Text size="xl" c="dimmed">No session opened</Text>
                    <Text size="sm" c="dimmed">左サイドバーからセッションを選択するか、新しいセッションを作成してください</Text>
                  </Stack>
            )}
          </Box>
          ) : (() => {
            // 現在のWorkspaceのタブのみを表示
            const visibleTabs = workspace ? getVisibleTabs(workspace.id) : [];
            
            return (
              <Tabs
                value={activeTabId}
                onChange={async (value) => {
                  if (!value) return;

                  const tab = tabs.find(t => t.id === value);
                  if (!tab) return;

                  console.log('[App] Tab switched:', {
                    tabId: value.substring(0, 8),
                    sessionId: tab.sessionId.substring(0, 8),
                    workspaceId: tab.workspaceId.substring(0, 8),
                    currentWorkspace: workspace?.id.substring(0, 8),
                  });

                  // 1. タブを切り替え
                  switchToTab(value);

                  // 2. バックエンドのセッションも切り替え
                  try {
                    await switchSession(tab.sessionId);
                    console.log('[App] Backend session switched');
                  } catch (err) {
                    console.error('[App] Failed to switch backend session:', err);
                    notifications.show({
                      title: 'Session Switch Failed',
                      message: String(err),
                      color: 'red',
                    });
                    return;
                  }

                  // 3. Workspace切り替え（必要な場合のみ）
                  if (tab.workspaceId !== workspace?.id) {
                    console.log('[App] Workspace differs, switching...', {
                      from: workspace?.id.substring(0, 8),
                      to: tab.workspaceId.substring(0, 8),
                    });

                    try {
                      await switchWorkspaceBackend(tab.sessionId, tab.workspaceId);
                      console.log('[App] Workspace switched, workspace-switched event will fire');
                      // ↑ 内部で 'workspace-switched' イベント発火
                      // ↓ 既存リスナー（L461-536）で全体同期
                    } catch (err) {
                      console.error('[App] Failed to switch workspace:', err);
                      notifications.show({
                        title: 'Workspace Switch Failed',
                        message: String(err),
                        color: 'red',
                      });
                    }
                  } else {
                    console.log('[App] Same workspace, no switch needed');
                  }
                }}
                style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}
              >
                <Tabs.List style={{ overflowX: 'auto', flexWrap: 'nowrap' }}>
                  {visibleTabs.map((tab) => (
                    <Tabs.Tab
                      key={tab.id}
                      value={tab.id}
                      style={{
                        minWidth: '120px',
                        maxWidth: '200px',
                      }}
                      leftSection={tab.isDirty ? '●' : undefined}
                        rightSection={
                          visibleTabs.length > 1 ? (
                            <CloseButton
                              size="xs"
                              onClick={async (e) => {
                                console.log('[App] CloseButton clicked:', {
                                  tabId: tab.id,
                                  title: tab.title,
                                  isDirty: tab.isDirty,
                                  visibleTabsCount: visibleTabs.length,
                                });
                                e.stopPropagation();

                                // 未保存の場合は確認
                                if (tab.isDirty) {
                                  if (!window.confirm(`"${tab.title}" has unsaved changes. Close anyway?`)) {
                                    console.log('[App] User cancelled close');
                                    return;
                                  }
                                }

                                // 1. 閉じるタブの情報を取得
                                const closingTab = tabs.find(t => t.id === tab.id);
                                if (!closingTab) return;

                                // 2. ActiveSessionのタブを閉じる場合
                                const isClosingActiveSession = closingTab.sessionId === currentSessionId;

                                console.log('[App] Calling closeTab for', tab.id, {
                                  isClosingActiveSession,
                                  currentSessionId: currentSessionId?.substring(0, 8),
                                  closingSessionId: closingTab.sessionId.substring(0, 8),
                                });

                                // 3. ActiveSessionだった場合、次のSessionを選択
                                if (isClosingActiveSession && workspace) {
                                  // 4a. 現在のWorkspace内の残りSession取得
                                  const remainingSessions = sessions.filter(
                                    s => s.workspace_id === workspace.id && s.id !== closingTab.sessionId
                                  );

                                  console.log('[App] Remaining sessions in workspace:', remainingSessions.length);

                                  if (remainingSessions.length > 0) {
                                    // 4b. 更新日時が直近のSessionを選択
                                    const sortedSessions = [...remainingSessions].sort(
                                      (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
                                    );
                                    const nextSession = sortedSessions[0];

                                    console.log('[App] Switching to next session:', nextSession.id.substring(0, 8), nextSession.title);

                                    try {
                                      // 4c. Backend Session切り替え
                                      await switchSession(nextSession.id);

                                      // 4d. 次のSessionのTabを開く（既に開いていればフォーカス）
                                      // openTab()は既存タブがあれば更新してフォーカス、なければ新規作成
                                      const messages = convertSessionToMessages(nextSession, userNickname);
                                      const newTabId = openTab(nextSession, messages, workspace.id, true);

                                      console.log('[App] Successfully switched to next session, tab:', newTabId.substring(0, 8));

                                      // 4e. 古いタブを閉じる（次のSessionに切り替え後）
                                      closeTab(tab.id);
                                    } catch (err) {
                                      console.error('[App] Failed to switch to next session:', err);
                                    }
                                  } else {
                                    // 4e. Workspace内にSessionがない場合、新規作成
                                    console.log('[App] No remaining sessions, creating new session');
                                    try {
                                      await createSession();
                                      console.log('[App] New session created');
                                    } catch (err) {
                                      console.error('[App] Failed to create new session:', err);
                                    }
                                  }
                                } else {
                                  // 非ActiveSessionのTab Closeの場合、単純に閉じる
                                  closeTab(tab.id);
                                }
                              }}
                            />
                          ) : undefined
                        }
                    >
                      <Text truncate style={{ maxWidth: '100%' }}>
                        {tab.title}
                      </Text>
                    </Tabs.Tab>
                  ))}
                </Tabs.List>

                {visibleTabs.map((tab) => (
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
            );
          })()}
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  );
}

export default App;
