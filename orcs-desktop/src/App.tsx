import { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check as checkForUpdates } from "@tauri-apps/plugin-updater";
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
  Paper,
  Loader,
  ActionIcon,
  Tooltip,
  Modal,
  Button,
} from "@mantine/core";
import { IconPlus } from '@tabler/icons-react';
import { useDisclosure } from '@mantine/hooks';
import "./App.css";
import { Message, MessageType, StreamingDialogueTurn } from "./types/message";
import { StatusInfo, getDefaultStatus } from "./types/status";
import { Task } from "./types/task";
import { Agent } from "./types/agent";
import { Session } from "./types/session";
import { useTaskStore } from "./stores/taskStore";
import { GitInfo } from "./types/git";
import { Navbar } from "./components/navigation/Navbar";
import { WorkspaceSwitcher } from "./components/workspace/WorkspaceSwitcher";
import { SettingsMenu } from "./components/settings/SettingsMenu";
import { parseCommand, extractSlashCommands } from "./utils/commandParser";
import { filterCommandsWithCustom, CommandDefinition } from "./types/command";
import { extractMentions, getCurrentMention, normalizeMentionsInText } from "./utils/mentionParser";
import { handleAndPersistSystemMessage, conversationMessage } from "./utils/systemMessage";
import { useSessions } from "./hooks/useSessions";
import { useWorkspace } from "./hooks/useWorkspace";
import { convertSessionToMessages, exportSessionToMarkdown, getAllMessages } from "./types/session";
import { SlashCommand } from "./types/slash_command";
import { useTabContext } from "./context/TabContext";
import { useSlashCommands } from "./hooks/useSlashCommands";
import { Tabs } from "@mantine/core";
import { ChatPanel } from "./components/chat/ChatPanel";
import type { SessionEvent } from "./types/session_event";
import { useAppStateStore } from "./stores/appStateStore";
import { useWorkspaceStore } from "./stores/workspaceStore";
import { useSessionSettingsStore } from "./stores/sessionSettingsStore";
import { usePersonaStore } from "./stores/personaStore";
import { useDebugStore } from "./stores/debugStore";

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
  const [closingTabId, setClosingTabId] = useState<string | null>(null);
  const [tabPreviewCache, setTabPreviewCache] = useState<Record<string, string>>({});
  const [sandboxExitModalOpened, setSandboxExitModalOpened] = useState(false);
  const [pendingSandboxExit, setPendingSandboxExit] = useState<import('./bindings/generated').SandboxState | null>(null);
  const [currentSandboxState, setCurrentSandboxState] = useState<import('./bindings/generated').SandboxState | null>(null);

  // Task Store
  const {
    initialize: initializeTaskStore,
    refreshTasks,
    deleteTask: deleteTaskFromStore,
    tasks: tasksMap,
    taskProgress: taskProgressMap,
  } = useTaskStore();

  // Convert Map to Array for ChatPanel compatibility
  const tasks = Array.from(tasksMap.values());
  const taskProgress = taskProgressMap;

  const [userNickname, setUserNickname] = useState<string>('You');
  const [userProfile, setUserProfile] = useState<{ nickname: string; background: string } | null>(null);
  const [gitInfo, setGitInfo] = useState<GitInfo>({
    is_repo: false,
    branch: null,
    repo_name: null,
  });
  const [customCommands, setCustomCommands] = useState<SlashCommand[]>([]);
  const [dialoguePresets, setDialoguePresets] = useState<import('./types/conversation').DialoguePreset[]>([]);

  // セッション管理をカスタムフックに切り替え
  const {
    sessions,
    // currentSessionId removed - use appStateStore
    loading: sessionsLoading,
    createSession,
    switchSession,
    deleteSession,
    renameSession,
    saveCurrentSession,
    refreshSessions,
    updateSession,
  } = useSessions();

  // Get currentSessionId from appStateStore (SSOT)
  const { appState } = useAppStateStore();
  const currentSessionId = appState?.activeSessionId ?? null;
  const isAppStateLoaded = useAppStateStore((state) => state.isLoaded);

  // ワークスペース管理 (Phase 4: simplified - no more refresh functions)
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { workspace, allWorkspaces, files: workspaceFiles, switchWorkspace: switchWorkspaceBackend } = useWorkspace();
  const [includeWorkspaceInPrompt, setIncludeWorkspaceInPrompt] = useState<boolean>(false);

  // AppState Store (Rust SSOT)
  const initializeAppState = useAppStateStore((state: { initialize: () => Promise<void> }) => state.initialize);

  // Workspace Store (Rust SSOT - Phase 4)
  const initializeWorkspace = useWorkspaceStore((state) => state.initialize);

  // Session Settings Store (Rust SSOT)
  const {
    talkStyle,
    conversationMode,
    executionStrategy,
    contextMode,
    activeParticipantIds,
    isLoaded: sessionSettingsLoaded,
    updateTalkStyle,
    updateConversationMode,
    updateExecutionStrategy,
    updateContextMode,
    toggleParticipant,
    refreshActiveParticipants,
    loadSettings: loadSessionSettings,
  } = useSessionSettingsStore();

  // Persona Store (Rust SSOT)
  const {
    personas,
    loadPersonas,
  } = usePersonaStore();

  // Debug Store
  const { debugSettings, initialize: initializeDebug } = useDebugStore();

  // Initialize AppState Store on mount
  useEffect(() => {
    initializeAppState().catch((error: unknown) => {
      console.error('[App] Failed to initialize AppState store:', error);
    });
  }, [initializeAppState]);

  // Initialize Debug Store on mount
  useEffect(() => {
    initializeDebug().catch((error: unknown) => {
      console.error('[App] Failed to initialize Debug store:', error);
    });
  }, [initializeDebug]);

  // Initialize Workspace Store on mount (Phase 4)
  useEffect(() => {
    initializeWorkspace().catch((error: unknown) => {
      console.error('[App] Failed to initialize Workspace store:', error);
    });
  }, [initializeWorkspace]);

  // Initialize Task Store on mount
  useEffect(() => {
    initializeTaskStore().catch((error: unknown) => {
      console.error('[App] Failed to initialize Task store:', error);
    });
  }, [initializeTaskStore]);

  // Initialize Persona Store on mount
  useEffect(() => {
    loadPersonas().catch((error: unknown) => {
      console.error('[App] Failed to load personas:', error);
    });
  }, [loadPersonas]);

  // Check for app updates on startup
  useEffect(() => {
    const checkForAppUpdates = async () => {
      try {
        console.log('[App] Checking for updates...');
        const update = await checkForUpdates();
        if (update) {
          console.log('[App] Update available:', update.version);
          // The built-in dialog will handle user interaction when dialog: true is set in tauri.conf.json
          // The updater will automatically prompt, download, and install if user accepts
        } else {
          console.log('[App] No updates available');
        }
      } catch (error) {
        // Silently fail - don't interrupt app startup for update check failures
        console.error('[App] Failed to check for updates:', error);
      }
    };

    checkForAppUpdates();
  }, []);

  // Check memory sync (Kaiba Batch) settings on startup (once only)
  const memorySyncCheckedRef = useRef(false);
  useEffect(() => {
    if (memorySyncCheckedRef.current) return;
    memorySyncCheckedRef.current = true;

    const checkMemorySyncSettings = async () => {
      try {
        const settings = await invoke<{ enabled: boolean; interval_secs: number }>('get_memory_sync_settings');
        if (settings.enabled) {
          notifications.show({
            title: 'Kaiba Batch Enabled',
            message: `Memory sync is active (interval: ${settings.interval_secs}s)`,
            color: 'blue',
            autoClose: 5000,
          });
        }
      } catch (error) {
        console.error('[App] Failed to check memory sync settings:', error);
      }
    };

    checkMemorySyncSettings();
  }, []);

  // Load sandbox state when active session changes
  useEffect(() => {
    const loadSandboxState = async () => {
      if (!currentSessionId) {
        setCurrentSandboxState(null);
        return;
      }

      try {
        const sandboxState = await invoke<import('./bindings/generated').SandboxState | null>('get_sandbox_state');
        setCurrentSandboxState(sandboxState);
      } catch (error) {
        console.error('[App] Failed to load sandbox state:', error);
        setCurrentSandboxState(null);
      }
    };

    loadSandboxState();
  }, [currentSessionId]);

  // Restore last selected workspace on app startup (Phase 3)
  useEffect(() => {
    const restoreLastWorkspace = async () => {
      // Skip if already restored
      if (workspaceRestoredRef.current) {
        return;
      }

      // Skip if appState not loaded
      if (!isAppStateLoaded || !appState) {
        return;
      }

      // Skip if no last selected workspace (initial app launch)
      if (!appState.lastSelectedWorkspaceId) {
        workspaceRestoredRef.current = true;
        return;
      }

      // Skip if current workspace already matches
      if (workspace && workspace.id === appState.lastSelectedWorkspaceId) {
        workspaceRestoredRef.current = true;
        return;
      }

      const lastWorkspaceId = appState.lastSelectedWorkspaceId;

      try {
        // Get active session (required for switchWorkspace)
        const activeSessionId = appState.activeSessionId;
        if (!activeSessionId) {
          workspaceRestoredRef.current = true;
          return;
        }

        await switchWorkspaceBackend(activeSessionId, lastWorkspaceId);
      } catch (error) {
        console.error('[App] Failed to restore last workspace:', error);
      }

      workspaceRestoredRef.current = true;
    };

    restoreLastWorkspace();
  }, [isAppStateLoaded, appState, workspace, switchWorkspaceBackend]);

  // タブ管理
  const {
    tabs,
    activeTabId,
    activeTabInput, // Performance: TabContext から直接取得（tabs の再計算を経由しない）
    openTab,
    closeTab,
    switchTab: switchToTab,
    switchWorkspace: switchWorkspaceTabs,
    updateTabTitle,
    updateTabMessages: _updateTabMessages,
    addMessageToTab,
    updateTabInput,
    updateTabAttachedFiles,
    addAttachedFileToTab,
    removeAttachedFileFromTab,
    setTabDragging,
    setTabThinking,
    getActiveTab,
    getTab: _getTab,
    getTabBySessionId,
  } = useTabContext();

  const [autoMode, setAutoMode] = useState<boolean>(false);
  const viewport = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const workspaceSwitchingRef = useRef(false);
  const workspaceRestoredRef = useRef(false);
  const tabsRestoredRef = useRef(false);
  const loadedSessionMessagesRef = useRef<Set<string>>(new Set()); // Phase 4: 読み込み済みセッションを記録

  // メッセージを追加するヘルパー関数（early definition for useRef/useSlashCommands）
  const addMessage = useCallback((type: MessageType, author: string, text: string, attachments?: import('./types/message').AttachedFile[]) => {
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
      attachments,
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
            setClosingTabId(activeTabId);
          } else {
            void closeTab(activeTabId); // Phase 2: closeTab is now async
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
  }, [tabs, activeTabId, switchToTab, closeTab]);

  // Handler for confirming tab close
  const handleConfirmCloseTab = async () => {
    if (!closingTabId) return;

    const tabId = closingTabId;
    setClosingTabId(null);

    // 1. 閉じるタブの情報を取得
    const closingTab = tabs.find(t => t.id === tabId);
    if (!closingTab) return;

    // 2. ActiveSessionのタブを閉じる場合
    const isClosingActiveSession = closingTab.sessionId === currentSessionId;

    // 3. ActiveSessionだった場合、次のSessionを選択
    if (isClosingActiveSession && workspace) {
      // 4a. 現在のWorkspace内の残りSession取得
      const remainingSessions = sessions.filter(
        s => s.workspaceId === workspace.id && s.id !== closingTab.sessionId
      );

      if (remainingSessions.length > 0) {
        // 4b. 更新日時が直近のSessionを選択
        const sortedSessions = [...remainingSessions].sort(
          (a, b) => b.updatedAt.localeCompare(a.updatedAt)
        );
        const nextSession = sortedSessions[0];
        await switchSession(nextSession.id);
      }
    }

    // 5. タブを閉じる
    await closeTab(tabId);
  };

  // Handler for cancelling tab close
  const handleCancelCloseTab = () => {
    setClosingTabId(null);
  };

  // Handler for sandbox exit with merge
  const handleSandboxExitMerge = async () => {
    if (!pendingSandboxExit) return;

    const sandboxState = pendingSandboxExit;
    setPendingSandboxExit(null);
    setSandboxExitModalOpened(false);

    try {
      await handleAndPersistSystemMessage(
        conversationMessage(
          `Exiting sandbox mode...\n\nBranch: ${sandboxState.sandbox_branch}\n\nMerging changes back to ${sandboxState.original_branch}...`,
          'info',
          '🚪'
        ),
        addMessage,
        invoke
      );

      await invoke('exit_sandbox_worktree', {
        options: {
          worktree_path: sandboxState.worktree_path,
          original_branch: sandboxState.original_branch,
          sandbox_branch: sandboxState.sandbox_branch,
          merge: true,
        },
      });

      await invoke('exit_sandbox_mode');

      // Clear current sandbox state
      setCurrentSandboxState(null);

      await handleAndPersistSystemMessage(
        conversationMessage(
          `Exited sandbox mode successfully!\n\nChanges have been merged to ${sandboxState.original_branch}.`,
          'success',
          '✅'
        ),
        addMessage,
        invoke
      );
      await saveCurrentSession();
    } catch (error) {
      console.error('Failed to exit sandbox mode:', error);
      await handleAndPersistSystemMessage(
        conversationMessage(
          `Failed to exit sandbox mode: ${error}\n\nYou may need to manually clean up the worktree.`,
          'error'
        ),
        addMessage,
        invoke
      );
      await saveCurrentSession();
    }
  };

  // Handler for sandbox exit with discard
  const handleSandboxExitDiscard = async () => {
    if (!pendingSandboxExit) return;

    const sandboxState = pendingSandboxExit;
    setPendingSandboxExit(null);
    setSandboxExitModalOpened(false);

    try {
      await handleAndPersistSystemMessage(
        conversationMessage(
          `Exiting sandbox mode...\n\nBranch: ${sandboxState.sandbox_branch}\n\nDiscarding all changes...`,
          'info',
          '🚪'
        ),
        addMessage,
        invoke
      );

      await invoke('exit_sandbox_worktree', {
        options: {
          worktree_path: sandboxState.worktree_path,
          original_branch: sandboxState.original_branch,
          sandbox_branch: sandboxState.sandbox_branch,
          merge: false,
        },
      });

      await invoke('exit_sandbox_mode');

      // Clear current sandbox state
      setCurrentSandboxState(null);

      await handleAndPersistSystemMessage(
        conversationMessage(
          `Exited sandbox mode successfully!\n\nChanges have been discarded.`,
          'success',
          '✅'
        ),
        addMessage,
        invoke
      );
      await saveCurrentSession();
    } catch (error) {
      console.error('Failed to exit sandbox mode:', error);
      await handleAndPersistSystemMessage(
        conversationMessage(
          `Failed to exit sandbox mode: ${error}\n\nYou may need to manually clean up the worktree.`,
          'error'
        ),
        addMessage,
        invoke
      );
      await saveCurrentSession();
    }
  };

  // Handler for cancelling sandbox exit
  const handleCancelSandboxExit = () => {
    setPendingSandboxExit(null);
    setSandboxExitModalOpened(false);
  };

  const activeTabScrollKey = useMemo(() => {
    const activeTab = tabs.find(t => t.id === activeTabId);
    if (!activeTab) {
      return null;
    }
    const lastMessageId =
      activeTab.messages.length > 0
        ? activeTab.messages[activeTab.messages.length - 1].id
        : 'no-messages';
    return `${activeTab.id}:${lastMessageId}`;
  }, [tabs, activeTabId]);

  // Auto-scroll to bottom when active tab's messages change
  useEffect(() => {
    if (!activeTabScrollKey) {
      return;
    }
    if (viewport.current) {
      viewport.current.scrollTo({
        top: viewport.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [activeTabScrollKey]);

  // Auto-scroll active tab into view when tab is switched
  useEffect(() => {
    if (!activeTabId) return;

    // Use setTimeout to ensure DOM is ready after tab switch
    const timeoutId = setTimeout(() => {
      const activeTabElement = document.querySelector(`[data-tab-id="${activeTabId}"]`);
      if (activeTabElement) {
        activeTabElement.scrollIntoView({
          behavior: 'smooth',
          inline: 'center',
          block: 'nearest',
        });
      }
    }, 100);

    return () => clearTimeout(timeoutId);
  }, [activeTabId]);

  // Listen for real-time dialogue turn events from backend
  // Use ref to ensure only one listener is registered
  const listenerRegistered = useRef(false);
  const addMessageToTabRef = useRef(addMessageToTab);
  const getTabBySessionIdRef = useRef(getTabBySessionId);
  const personasRef = useRef(personas);

  // Performance: Pre-cache persona names to avoid repeated string operations during input
  const personasWithCache = useMemo(() =>
    personas.map(p => ({
      ...p,
      _lowerName: p.name.toLowerCase(),
      _underscoreName: p.name.replace(/ /g, '_'),
      _lowerUnderscoreName: p.name.replace(/ /g, '_').toLowerCase(),
    })),
    [personas]
  );
  const personasCacheRef = useRef(personasWithCache);
  const activeParticipantIdsRef = useRef(activeParticipantIds);
  const currentSessionIdRef = useRef(currentSessionId);
  const handleSlashCommandRef =
    useRef<ReturnType<typeof useSlashCommands>['handleSlashCommand'] | null>(
      null
    );

  // 最新の関数をrefに保持（クロージャーの問題を回避）
  useEffect(() => {
    addMessageToTabRef.current = addMessageToTab;
  }, [addMessageToTab]);

  useEffect(() => {
    getTabBySessionIdRef.current = getTabBySessionId;
  }, [getTabBySessionId]);

  useEffect(() => {
    personasRef.current = personas;
  }, [personas]);

  useEffect(() => {
    personasCacheRef.current = personasWithCache;
  }, [personasWithCache]);

  useEffect(() => {
    activeParticipantIdsRef.current = activeParticipantIds;
  }, [activeParticipantIds]);

  // 最新のcurrentSessionIdをrefに保持
  useEffect(() => {
    currentSessionIdRef.current = currentSessionId;
  }, [currentSessionId]);

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
      unlisten = await listen<StreamingDialogueTurn>('dialogue-turn', (event) => {
        const turn = event.payload;

        // Find the tab for this session_id
        const targetTab = getTabBySessionIdRef.current(turn.sessionId);

        if (!targetTab) {
          console.log(`[STREAM] Ignoring message for session ${turn.sessionId} - no tab found`);
          return;
        }

        const isActiveSession = turn.sessionId === currentSessionIdRef.current;
        console.log('[STREAM] Event received:', turn.type, 'for session:', turn.sessionId.substring(0, 8), 'active:', isActiveSession);

        // Handle different turn types
        switch (turn.type) {
          case 'Chunk': {
            console.log('[STREAM] Adding message chunk:', turn.author);

            // Determine message type: System messages vs AI messages
            const isSystemMessage = turn.author === 'System';

            // Find persona by name to get icon and base_color (only for AI messages)
            const persona = !isSystemMessage ? personasRef.current.find(p => p.name === turn.author) : undefined;

            const newMessage: Message = {
              id: `${Date.now()}-${Math.random()}`,
              type: isSystemMessage ? 'system' : 'ai',
              author: turn.author,
              text: turn.content,
              timestamp: new Date(),
              icon: persona?.icon,
              baseColor: persona?.base_color,
              backend: persona?.backend,
              modelName: persona?.model_name,
            };

            addMessageToTabRef.current(targetTab.id, newMessage);

            // Agent responses can themselves issue SlashCommands. Detect and execute them
            if (
              !isSystemMessage &&
              turn.sessionId === currentSessionIdRef.current &&
              handleSlashCommandRef.current
            ) {
              const detectedCommands = extractSlashCommands(turn.content);
              console.log("detectedCommands", detectedCommands);
              if (detectedCommands.length > 0) {
                const actorName = turn.author || 'Agent';
                void (async () => {
                  for (const commandText of detectedCommands) {
                    try {
                      await handleSlashCommandRef.current?.(commandText, {
                        source: 'agent',
                        actorName,
                        autoSubmit: true,
                      });
                    } catch (error) {
                      console.error(
                        '[STREAM] Failed to execute agent slash command:',
                        error
                      );
                    }
                  }
                })();
              }
            }
            break;
          }

          case 'Error': {
            console.log('[STREAM] Error received:', turn.message);

            const errorMessage: Message = {
              id: `${Date.now()}-${Math.random()}`,
              type: 'error',
              author: '',
              text: turn.message,
              timestamp: new Date(),
            };

            addMessageToTabRef.current(targetTab.id, errorMessage);

            // Show error toast only for active session
            if (isActiveSession) {
              notifications.show({
                title: 'Agent Error',
                message: turn.message,
                color: 'red',
                icon: '❌',
                autoClose: 10000,
              });
            }
            break;
          }

          case 'Final':
            console.log('[STREAM] Streaming completed for session:', turn.sessionId.substring(0, 8));
            // Final turn just indicates completion, no action needed
            break;

          case 'AutoChatProgress':
            console.log('[STREAM] AutoChat progress:', turn.currentIteration, '/', turn.maxIterations);
            // Update TabContext AutoChat iteration state
            // TODO: Implement setTabAutoChatIteration call here
            break;

          case 'AutoChatComplete':
            console.log('[STREAM] AutoChat completed:', turn.totalIterations, 'iterations');
            // Turn off AutoChat mode
            setAutoMode(false);

            // Clear thinking state
            setTabThinking(targetTab.id, false);

            // Add system message to indicate completion
            const completionMessage: Message = {
              id: `${Date.now()}-${Math.random()}`,
              type: 'system',
              author: 'System',
              text: `AutoChat completed after ${turn.totalIterations} iterations.`,
              timestamp: new Date(),
            };

            addMessageToTabRef.current(targetTab.id, completionMessage);
            break;

          default:
            console.warn('[STREAM] Unknown turn type:', (turn as any).type);
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

  // Load session settings on session change (includes conversationMode, talkStyle, etc.)
  useEffect(() => {
    if (currentSessionId) {
      loadSessionSettings(currentSessionId).catch((error) => {
        console.error('Failed to load session settings:', error);
      });
    }
  }, [currentSessionId, loadSessionSettings]);

  // Load active session messages on startup or when currentSessionId changes
  useEffect(() => {
    const loadActiveSessionMessages = async () => {
      if (!currentSessionId || sessionsLoading || !sessionSettingsLoaded) {
        return;
      }

      try {
        const activeSession = sessions.find(s => s.id === currentSessionId);
        if (!activeSession) {
          return;
        }

        // Enrich participant_icons from current personas if missing
        if (!activeSession.participantIcons || Object.keys(activeSession.participantIcons).length === 0) {
          activeSession.participantIcons = {};
          personas.forEach(persona => {
            if (persona.icon && activeSession.participants[persona.id]) {
              activeSession.participantIcons[persona.id] = persona.icon;
            }
          });
        }

        // Phase 4: Check if messages already loaded to prevent infinite loop
        if (loadedSessionMessagesRef.current.has(currentSessionId)) {
          console.log('[App] Session messages already loaded, skipping:', currentSessionId.substring(0, 8));
          return;
        }

        // Phase 4: appState から tabId を取得
        const existingOpenTab = appState?.openTabs.find(t => t.sessionId === currentSessionId);

        // Check if tab already exists
        const existingTab = existingOpenTab ? getTabBySessionId(currentSessionId) : null;

        // If tab exists, check if messages are empty or need preview data
        if (existingTab) {
          // If messages are empty, we need to load them
          if (existingTab.messages.length === 0) {
            console.log('[App] Tab exists but messages are empty, loading...');
            // Continue to load messages
          } else {
            // Messages exist, check if they need preview data
            const needsPreviewData = existingTab.messages.some(msg =>
              msg.attachments && msg.attachments.length > 0 &&
              msg.attachments.some(att => !att.data)
            );

            if (!needsPreviewData) {
              console.log('[App] Tab exists with messages, no preview data needed');
              return;
            }
          }
        }

        const loadingSessionId = activeSession.id;
        let restoredMessages = convertSessionToMessages(activeSession, userNickname);

        // Load preview data for attached files BEFORE opening tab
        try {
          restoredMessages = await Promise.all(
            restoredMessages.map(async (message) => {
              if (message.attachments && message.attachments.length > 0) {
                const updatedAttachments = await Promise.all(
                  message.attachments.map(async (attachment) => {
                    if (attachment.data) return attachment; // Already has data

                    try {
                      const previewData = await invoke<{
                        name: string;
                        path: string;
                        mimeType: string;
                        size: number;
                        data: string;
                      }>("get_file_preview_data", {
                        filePath: attachment.path,
                      });

                      return {
                        name: previewData.name,
                        path: previewData.path,
                        mimeType: previewData.mimeType,
                        size: previewData.size,
                        data: previewData.data,
                      };
                    } catch (error) {
                      console.error('[SESSION LOAD] Failed to load preview data:', attachment.path, error);
                      return attachment; // Keep original if failed
                    }
                  })
                );
                return { ...message, attachments: updatedAttachments };
              }
              return message;
            })
          );
        } catch (error) {
          console.error('[SESSION LOAD] Error loading preview data:', error);
        }

        // Check if session is still current before opening tab
        if (currentSessionId !== loadingSessionId) {
          return;
        }

        // Open or update tab with preview data
        if (workspace) {
          if (existingTab) {
            // Tab exists, update messages and title
            console.log('[App] Updating existing tab with messages:', {
              tabId: existingTab.id.substring(0, 8),
              title: activeSession.title,
              messagesCount: restoredMessages.length,
            });
            void updateTabTitle(existingTab.id, activeSession.title); // Phase 3: updateTabTitle is now async
            _updateTabMessages(existingTab.id, restoredMessages);

            // Phase 4: Mark as loaded to prevent infinite loop
            loadedSessionMessagesRef.current.add(currentSessionId);
          } else {
            // Tab doesn't exist, create it
            void openTab(activeSession, restoredMessages, workspace.id); // Phase 2: openTab is now async

            // Phase 4: Mark as loaded
            loadedSessionMessagesRef.current.add(currentSessionId);
          }
        }

        // Note: executionStrategy is loaded via sessionSettingsStore in loadSessionSettings useEffect
      } catch (error) {
        console.error('[App] Failed to load active session messages:', error);
      }
    };

    loadActiveSessionMessages();
  }, [currentSessionId, sessionsLoading, sessionSettingsLoaded, userNickname, personas, workspace, openTab, getTabBySessionId]);
  // Note: `sessions` removed from deps to avoid unnecessary re-renders
  // We only use sessions.find() inside, which is called on-demand
  // Note: sessionSettingsLoaded ensures Store is ready before loading messages

  // Restore tabs from backend on app startup (Phase 2)
  useEffect(() => {
    const restoreTabsFromBackend = async () => {
      // Skip if already restored
      if (tabsRestoredRef.current) {
        return;
      }

      // Skip if appState not loaded
      if (!isAppStateLoaded || !appState) {
        return;
      }

      // Skip if sessions not loaded
      if (sessionsLoading) {
        return;
      }

      // Skip if workspace not loaded
      if (!workspace) {
        return;
      }

      // Skip if no tabs to restore (initial app launch)
      if (appState.openTabs.length === 0) {
        tabsRestoredRef.current = true;
        return;
      }

      console.log('[App] Restoring tabs from backend...', {
        tabsCount: appState.openTabs.length,
      });

      // Sort tabs by order
      const sortedTabs = [...appState.openTabs].sort((a, b) => a.order - b.order);

      for (const backendTab of sortedTabs) {
        // Phase 3: Backend SSOT - タブは自動的にレンダリングされるため initializeTabUIState 不要
        // Session データも SessionContext から自動的に取得される

        // Find session for this tab
        const session = sessions.find((s) => s.id === backendTab.sessionId);
        if (!session) {
          console.warn('[App] Session not found for tab:', backendTab.sessionId);
          continue;
        }

        // Only load full session data for the active tab
        const isActiveTab = appState.activeTabId === backendTab.id;
        if (!isActiveTab) {
          console.log('[App] Non-active tab will render with basic info:', {
            tabId: backendTab.id.substring(0, 8),
            title: session.title,
          });
          continue;
        }

        // Use session data from sessions array (includes personaHistories)
        // Note: sessions array is loaded by SessionContext with full data
        let restoredMessages = convertSessionToMessages(session, userNickname);
        console.log('[App] Restored messages for active tab:', {
          tabId: backendTab.id.substring(0, 8),
          sessionId: backendTab.sessionId.substring(0, 8),
          title: session.title,
          messagesCount: restoredMessages.length,
        });

        // Load preview data for attached files BEFORE opening tab
        try {
          restoredMessages = await Promise.all(
            restoredMessages.map(async (message) => {
              if (message.attachments && message.attachments.length > 0) {
                const updatedAttachments = await Promise.all(
                  message.attachments.map(async (attachment) => {
                    if (attachment.data) return attachment; // Already has data

                    try {
                      const previewData = await invoke<{
                        name: string;
                        path: string;
                        mimeType: string;
                        size: number;
                        data: string;
                      }>('get_file_preview_data', {
                        filePath: attachment.path,
                      });

                      return {
                        name: previewData.name,
                        path: previewData.path,
                        mimeType: previewData.mimeType,
                        size: previewData.size,
                        data: previewData.data,
                      };
                    } catch (error) {
                      console.error('[App] Failed to load preview data:', attachment.path, error);
                      return attachment; // Keep original if failed
                    }
                  })
                );
                return { ...message, attachments: updatedAttachments };
              }
              return message;
            })
          );
        } catch (error) {
          console.error('[App] Error loading preview data during tab restoration:', error);
        }

        // Update messages for the already-initialized active tab
        console.log('[App] Updating active tab with messages:', {
          tabId: backendTab.id.substring(0, 8),
          messagesCount: restoredMessages.length,
        });
        _updateTabMessages(backendTab.id, restoredMessages);
      }

      // Activate the tab that was active before app restart
      if (appState.activeTabId) {
        const activeBackendTab = appState.openTabs.find((t) => t.id === appState.activeTabId);
        if (activeBackendTab) {
          // Find local tab by session_id (since local tab IDs are different from backend tab IDs)
          const localTab = getTabBySessionId(activeBackendTab.sessionId);
          if (localTab) {
            switchToTab(localTab.id);
          }
        }
      }

      tabsRestoredRef.current = true;
    };

    restoreTabsFromBackend();
  }, [
    isAppStateLoaded,
    appState,
    sessionsLoading,
    sessions,
    workspace,
    userNickname,
    updateTabTitle,
    getTabBySessionId,
    switchToTab,
  ]);

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

  // Load dialogue presets
  const refreshDialoguePresets = useCallback(async () => {
    try {
      const presets = await invoke<import('./types/conversation').DialoguePreset[]>('get_dialogue_presets');
      setDialoguePresets(presets);
      console.log('[App] Loaded dialogue presets:', presets.length);
    } catch (error) {
      console.error('Failed to load dialogue presets:', error);
    }
  }, []);

  // Load dialogue presets on startup
  useEffect(() => {
    refreshDialoguePresets();
  }, [refreshDialoguePresets]);

  // Note: Personas and activeParticipantIds are now loaded via sessionSettingsStore
  // in loadSessionSettings useEffect (triggered on session change)

  // 初回セッション自動作成（Workspace がある場合のみ）
  useEffect(() => {
    const initializeSession = async () => {
      // ローディング中はスキップ
      if (sessionsLoading) return;
      
      // Workspace があるが Session がない場合に自動作成
      if (workspace && sessions.length === 0) {
        console.log('[App] No sessions found, creating initial session for workspace');
        try {
          await createSession(workspace.id);
          console.log('[App] Initial session created');
        } catch (error) {
          console.error('[App] Failed to create initial session:', error);
        }
      }
    };
    
    initializeSession();
  }, [sessionsLoading, workspace, sessions.length, createSession]);


  // Listen for workspace-switched events to refresh workspace data and Git info
  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    (async () => {
      unlistenFn = await listen<string>('workspace-switched', async () => {
      if (workspaceSwitchingRef.current) {
        console.log('[App] workspace-switched event ignored (refresh already in progress)');
        return;
      }
      workspaceSwitchingRef.current = true;

      try {
        console.log('[App] workspace-switched event received, refreshing Git info');
        // Phase 4: No need to refresh workspace manually - event-driven via workspace:update

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
              void openTab(activeSession, restoredMessages, updatedWorkspace.id); // Phase 2: openTab is now async
              console.log('[App] Opened tab for active session after workspace switch');
            } else {
              // 既にタブがあればフォーカス
              await switchToTab(existingTab.id);
              console.log('[App] Focused existing tab for active session');
            }
          } else {
            console.log('[App] No active session');
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
      console.log('[App] workspace-switched listener registered successfully');
    })();

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [refreshSessions, switchWorkspaceTabs, openTab, switchToTab, getTabBySessionId, userNickname]);

  // Performance: activeTabInput は TabContext から直接取得（上記の destructuring）
  // tabs の再計算を経由しないため、入力時のパフォーマンスが向上

  // 入力内容が変更されたときにコマンド/エージェントサジェストを更新（デバウンス化）
  useEffect(() => {
    const timer = setTimeout(() => {
      const input = activeTabInput;
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
        // Support both original name and underscore format (e.g., "Ayaka Nakamura" matches "Ayaka_Nakamura")
        // Performance: Use pre-cached names from personasCacheRef to avoid repeated string operations
        const lowerFilter = mentionFilter.toLowerCase();
        const currentParticipantIds = activeParticipantIdsRef.current;
        const filtered: Agent[] = personasCacheRef.current
          .filter(p => {
            const nameMatch = p._lowerName.includes(lowerFilter);
            const underscoreMatch = p._lowerUnderscoreName.includes(lowerFilter);
            return nameMatch || underscoreMatch;
          })
          .map(p => ({
            id: p.id,
            name: p._underscoreName, // Display with underscores for mention input
            status: currentParticipantIds.includes(p.id) ? 'running' as const : 'idle' as const,
            description: `${p.role} - ${p.background}`,
            isActive: currentParticipantIds.includes(p.id),
          }));
        setFilteredAgents(filtered);
        setShowAgentSuggestions(filtered.length > 0);
        setSelectedAgentIndex(0);
      } else {
        setShowAgentSuggestions(false);
      }
    }, 50); // 50ms debounce for responsive suggestions

    return () => clearTimeout(timer);
    // Performance: personas and activeParticipantIds are accessed via refs to avoid unnecessary re-runs
  }, [activeTabInput, customCommands]);

  // Get thread content as text for action commands
  const getThreadAsText = useCallback((): string => {
    if (!activeTabId) return '';
    const activeTab = tabs.find(t => t.id === activeTabId);
    if (!activeTab) return '';

    return activeTab.messages
      .map((msg) => {
        const time = msg.timestamp.toLocaleString();
        return `[${time}] ${msg.author} (${msg.type}):\n${msg.text}\n`;
      })
      .join('\n---\n\n');
  }, [activeTabId, tabs]);

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
    // Note: conversationMode is now managed by Store, removed setConversationMode
    // Note: talkStyle is now managed by Store, removed setTalkStyle
    setInput: (value) => {
      if (activeTabId) {
        updateTabInput(activeTabId, value);
      }
    },
    refreshPersonas: loadPersonas,
    refreshSessions,
    onRequestSandboxExit: (sandboxState) => {
      setPendingSandboxExit(sandboxState);
      setSandboxExitModalOpened(true);
    },
    onSandboxEntered: (sandboxState) => {
      setCurrentSandboxState(sandboxState);
    },
    getThreadAsText,
  });

  useEffect(() => {
    handleSlashCommandRef.current = handleSlashCommand;
  }, [handleSlashCommand]);

  const processInput = useCallback(
    async (rawInput: string, attachedFiles: File[] = []) => {
      if (!rawInput.trim() && attachedFiles.length === 0) {
        return;
      }

      const currentFiles = [...attachedFiles];

      const mentions = extractMentions(rawInput);
      if (mentions.length > 0) {
        console.log('[MENTION EVENT] Agents mentioned:', mentions.map(m => m.mentionText));
      }

      // SlashCommandの処理（分離済み）
      const parsed = parseCommand(rawInput);
      let backendInput = rawInput;
      let suppressUserEcho = false;

      if (parsed.isCommand && parsed.command) {
        const commandResult = await handleSlashCommand(rawInput);

        // SlashCommandの処理が完了（フロントエンドでのみ処理）
        if (commandResult.nextInput === null) {
          return;
        }

        backendInput = commandResult.nextInput;
        suppressUserEcho = commandResult.suppressUserMessage ?? false;
      }

      if (parsed.isCommand && parsed.command && !backendInput.trim()) {
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

      // アクティブなタブのAI思考状態を設定
      if (activeTabId) {
        setTabThinking(activeTabId, true, 'AI Assistant');
      }

      try {
        // Upload files to workspace and get paths
        const filePaths: string[] = [];
        const attachedFileData: import('./types/message').AttachedFile[] = [];
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

              // Get file preview data for images
              try {
                const previewData = await invoke<{
                  name: string;
                  path: string;
                  mimeType: string;
                  size: number;
                  data: string;
                }>("get_file_preview_data", {
                  filePath: uploadedFile.path,
                });

                attachedFileData.push({
                  name: previewData.name,
                  path: previewData.path,
                  mimeType: previewData.mimeType,
                  size: previewData.size,
                  data: previewData.data,
                });
              } catch (previewError) {
                console.error('[FILE] Failed to get preview data:', file.name, previewError);
                // Still add basic file info even if preview fails
                attachedFileData.push({
                  name: file.name,
                  path: uploadedFile.path,
                  mimeType: file.type || 'application/octet-stream',
                  size: file.size,
                });
              }
            } catch (uploadError) {
              console.error('[FILE] Failed to upload file:', file.name, uploadError);
              addMessage('error', 'System', `Failed to upload file ${file.name}: ${uploadError}`);
            }
          }
        }

        // Add user message with attachments after upload completes
        if (!suppressUserEcho) {
          addMessage('user', userNickname, messageText, attachedFileData.length > 0 ? attachedFileData : undefined);
        }

        // Normalize mentions before sending to backend (_ → space)
        // Example: "@Ayaka_Nakamura" → "@Ayaka Nakamura"
        const normalizedInput = normalizeMentionsInText(backendInput);

        const sessionEvent: SessionEvent = {
          type: 'user_input',
          content: normalizedInput,
          attachments: filePaths.length > 0 ? filePaths : undefined,
        };

        const result = await invoke<InteractionResult>('publish_session_event', {
          event: sessionEvent,
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
        // Update only the current session to refresh MessageCount in SessionList
        if (currentSessionId) {
          await updateSession(currentSessionId);
        }
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
      updateSession,
      setTabThinking,
      activeTabId,
      setStatus,
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

  // コマンドを選択（メモ化）
  const selectCommand = useCallback((command: CommandDefinition) => {
    if (!activeTabId) return;
    updateTabInput(activeTabId, `/${command.name} `);
    setShowSuggestions(false);
    textareaRef.current?.focus();
  }, [activeTabId, updateTabInput]);

  // エージェントを選択（メモ化）
  const selectAgent = useCallback((agent: Agent) => {
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
  }, [activeTabId, getActiveTab, updateTabInput]);

  // StatusBarからPersonaをメンション挿入（メモ化）
  const handleMentionPersona = useCallback((personaId: string) => {
    if (!activeTabId) return;
    const persona = personas.find(p => p.id === personaId);
    if (!persona) return;

    const activeTab = getActiveTab();
    if (!activeTab) return;

    const mentionName = persona.name.replace(/ /g, '_');
    const currentInput = activeTab.input;
    const newInput = currentInput + (currentInput.endsWith(' ') || currentInput === '' ? '' : ' ') + `@${mentionName} `;
    updateTabInput(activeTabId, newInput);
    textareaRef.current?.focus();
  }, [activeTabId, personas, getActiveTab, updateTabInput]);

  // StatusBarからPersonaを即時呼び出し（メモ化）
  const handleInvokePersona = useCallback((personaId: string) => {
    if (!activeTabId) return;
    const persona = personas.find(p => p.id === personaId);
    if (!persona) return;

    const mentionName = persona.name.replace(/ /g, '_');
    const message = `@${mentionName}`;

    // 直接processInputを呼び出して投稿
    void processInput(message);
  }, [activeTabId, personas, processInput]);

  // 入力変更ハンドラ（メモ化）
  const handleInputChange = useCallback((value: string) => {
    if (activeTabId) {
      updateTabInput(activeTabId, value);
    }
  }, [activeTabId, updateTabInput]);

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
        // Persist system message to backend
        handleAndPersistSystemMessage(
          conversationMessage(`📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`, 'info', undefined, 'system'),
          addMessage,
          invoke
        );
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
        // Persist system message to backend
        handleAndPersistSystemMessage(
          conversationMessage(`📎 Attached ${files.length} file(s): ${files.map(f => f.name).join(', ')}`, 'info', undefined, 'system'),
          addMessage,
          invoke
        );
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

  // クリップボードからペーストしてWorkspaceに保存し、セッションにアタッチするハンドラー
  const handlePasteAndAttach = async () => {
    if (!workspace || !activeTabId) {
      notifications.show({
        title: 'Error',
        message: 'No workspace or active session',
        color: 'red',
      });
      return;
    }

    try {
      // 1. クリップボード読み取り
      const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
      const clipboardText = await readText();

      if (!clipboardText || clipboardText.trim() === '') {
        notifications.show({
          title: 'Clipboard Empty',
          message: 'No text content in clipboard',
          color: 'yellow',
        });
        return;
      }

      // 2. ファイル名生成（タイムスタンプ付き）
      const now = new Date();
      const timestamp = now.toISOString().replace(/[:.]/g, '-').split('T')[0] + '_' +
                        now.toTimeString().split(' ')[0].replace(/:/g, '-');
      const filename = `clipboard_${timestamp}.txt`;

      // 3. テキストをバイト列に変換
      const encoder = new TextEncoder();
      const bytes = encoder.encode(clipboardText);
      const fileData = Array.from(bytes);

      // 4. Workspace にアップロード
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: null,
        messageTimestamp: null,
      });

      // 5. セッションにアタッチ
      const file = new File([new Uint8Array(bytes)], filename, { type: 'text/plain' });
      addAttachedFileToTab(activeTabId, file);

      // 6. 成功通知
      notifications.show({
        title: 'Pasted & Attached',
        message: `Created "${filename}" and attached to session`,
        color: 'green',
      });
    } catch (error) {
      console.error('[handlePasteAndAttach] Error:', error);
      notifications.show({
        title: 'Paste Failed',
        message: String(error),
        color: 'red',
      });
    }
  };

  // ワークスペースファイルから新規セッションを作成するハンドラー
  const handleNewSessionWithFile = async (file: File) => {
    if (!workspace) {
      addMessage('error', 'System', 'No workspace selected');
      return;
    }

    try {
      // 1. Create new session
      const { invoke } = await import('@tauri-apps/api/core');
      const newSession = await invoke<Session>('create_session', {
        workspaceId: workspace.id,
      });

      console.log('[handleNewSessionWithFile] Created new session:', newSession.id);

      // 2. Refresh sessions list to include the new session
      await refreshSessions();

      // 3. Get full session data (needed for openTab)
      const fullSession = await switchSession(newSession.id);
      const restoredMessages = convertSessionToMessages(fullSession, userNickname);

      // 4. Open tab
      const tabId = await openTab(fullSession, restoredMessages, workspace.id); // Phase 2: openTab is now async
      console.log('[handleNewSessionWithFile] Opened tab:', tabId);

      // 5. Attach file to the newly created tab
      addAttachedFileToTab(tabId, file);
      console.log('[handleNewSessionWithFile] Attached file:', file.name);

      // 6. Show notification
      notifications.show({
        title: 'New Session with File',
        message: `Created session with ${file.name}`,
        color: 'blue',
        icon: '📎',
      });
    } catch (error) {
      console.error('Failed to create session with file:', error);
      addMessage('error', 'System', `Failed to create new session: ${error}`);
    }
  };

  // ワークスペースファイルからセッションに移動するハンドラー
  const handleGoToSessionFromFile = (sessionId: string, messageTimestamp?: string) => {
    const session = sessions.find(s => s.id === sessionId);
    if (session) {
      handleSessionSelect(session);

      // If messageTimestamp is provided, scroll to that message after session loads
      if (messageTimestamp) {
        // Retry mechanism to wait for DOM to be ready
        const scrollToMessage = (attempt: number = 0) => {
          const tab = tabs.find(t => t.sessionId === sessionId);

          if (tab) {
            // Find the message with matching timestamp (compare only up to milliseconds)
            const targetTimestamp = messageTimestamp.substring(0, 23); // "2025-11-11T04:58:34.760"
            const targetMessage = tab.messages.find(m => {
              const msgTimestamp = m.timestamp.toISOString().substring(0, 23);
              return msgTimestamp === targetTimestamp;
            });

            if (targetMessage) {
              // Search for message element by timestamp prefix (since message IDs include random suffix)
              const allMessageElements = document.querySelectorAll('[id^="message-"]');

              // Find element whose ID contains the target timestamp
              let messageElement: Element | null = null;
              for (const element of allMessageElements) {
                if (element.id.includes(targetTimestamp)) {
                  messageElement = element;
                  break;
                }
              }

              if (messageElement) {
                messageElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
              } else if (attempt < 10) {
                // Retry after 200ms if element not found yet (max 10 attempts = 2 seconds)
                setTimeout(() => scrollToMessage(attempt + 1), 200);
              }
            }
          } else if (attempt < 10) {
            // Tab not ready yet, retry
            setTimeout(() => scrollToMessage(attempt + 1), 200);
          }
        };

        // Start attempting after initial delay
        setTimeout(() => scrollToMessage(0), 300);
      }
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

      // Phase 4: No need to refresh workspace manually - event-driven via workspace:update

      // Add system message to chat history and persist to session
      await handleAndPersistSystemMessage(
        conversationMessage(
          `Message saved to workspace: ${filename}`,
          'success',
          '💾'
        ),
        addMessage,
        invoke
      );

      // Toast notification for immediate feedback
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

  // タスクをワークスペースに保存するハンドラー
  const handleSaveTaskToWorkspace = async (task: Task) => {
    try {
      // ファイル名を生成（タイムスタンプ + タスクタイトル）
      const timestamp = new Date(task.createdAt).toISOString().replace(/[:.]/g, '-');
      const sanitizedTitle = task.title.replace(/[^a-zA-Z0-9_-]/g, '_').slice(0, 50);
      const filename = `task_${timestamp}_${sanitizedTitle}.md`;

      // タスク内容をMarkdown形式で整形
      let content = `# Task: ${task.title}\n\n`;
      content += `**Status:** ${task.status}\n`;
      content += `**Created:** ${new Date(task.createdAt).toLocaleString()}\n`;
      content += `**Updated:** ${new Date(task.updatedAt).toLocaleString()}\n`;
      if (task.completedAt) {
        content += `**Completed:** ${new Date(task.completedAt).toLocaleString()}\n`;
      }
      content += `**Steps Executed:** ${task.stepsExecuted}\n`;
      content += `**Steps Skipped:** ${task.stepsSkipped}\n\n`;

      content += `## Description\n\n${task.description}\n\n`;

      if (task.result) {
        content += `## Result\n\n${task.result}\n\n`;
      }

      if (task.error) {
        content += `## Error\n\n${task.error}\n\n`;
      }

      if (task.executionDetails?.context) {
        content += `## Execution Context\n\n`;
        for (const [key, value] of Object.entries(task.executionDetails.context)) {
          content += `### ${key}\n\n`;
          if (typeof value === 'string') {
            content += `\`\`\`\n${value}\n\`\`\`\n\n`;
          } else {
            content += `\`\`\`json\n${JSON.stringify(value, null, 2)}\n\`\`\`\n\n`;
          }
        }
      }

      // メッセージテキストをバイト配列に変換
      const encoder = new TextEncoder();
      const data = encoder.encode(content);
      const fileData = Array.from(data);

      // ワークスペースIDを取得
      const workspace = await invoke<{ id: string }>('get_current_workspace');

      // ワークスペースに保存
      await invoke('upload_file_from_bytes', {
        workspaceId: workspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: task.sessionId,
        messageTimestamp: task.createdAt,
      });

      // Phase 4: No need to refresh workspace manually - event-driven via workspace:update

      // Toast notification
      notifications.show({
        title: 'Task saved',
        message: `${filename}`,
        color: 'green',
        icon: '💾',
      });
    } catch (err) {
      console.error('Failed to save task to workspace:', err);
      notifications.show({
        title: 'Failed to save task',
        message: String(err),
        color: 'red',
      });
    }
  };

  // セッションをワークスペースに保存するハンドラー
  const handleSaveSessionToWorkspace = async (session: Session) => {
    try {
      // Fetch latest session data from backend (includes personaHistories with all messages)
      const latestSession = await invoke<Session | null>('get_session', { sessionId: session.id });
      if (!latestSession) {
        throw new Error(`Session not found: ${session.id}`);
      }

      // Markdownを生成
      const { filename, content, latestMessageTimestamp } = exportSessionToMarkdown(
        latestSession,
        userProfile?.nickname ?? 'You'
      );

      // メッセージテキストをバイト配列に変換
      const encoder = new TextEncoder();
      const data = encoder.encode(content);
      const fileData = Array.from(data);

      // ワークスペースIDを取得
      const currentWorkspace = await invoke<{ id: string }>('get_current_workspace');

      // ワークスペースに保存
      await invoke('upload_file_from_bytes', {
        workspaceId: currentWorkspace.id,
        filename: filename,
        fileData: fileData,
        sessionId: session.id,
        messageTimestamp: latestMessageTimestamp,
      });

      // Toast notification
      notifications.show({
        title: 'Session saved',
        message: `${filename}`,
        color: 'green',
        icon: '💾',
      });
    } catch (err) {
      console.error('Failed to save session to workspace:', err);
      notifications.show({
        title: 'Failed to save session',
        message: String(err),
        color: 'red',
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
    // Delete task from taskStore
    try {
      await deleteTaskFromStore(taskId);
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
        workspaceId: session.workspaceId.substring(0, 8),
        currentWorkspace: workspace?.id.substring(0, 8),
      });

      // 1. Workspace切り替え（必要なら）
      if (session.workspaceId !== workspace?.id) {
        console.log('[App] Switching workspace for session...');
        await switchWorkspaceBackend(session.id, session.workspaceId);
        // ↑ 'workspace-switched' イベント発火 → 既存リスナーで全体同期
      }

      // 2. セッションを切り替え（バックエンドで履歴付きSessionDataを取得）
      const fullSession = await switchSession(session.id);
      console.log('[App] switchSession result:', {
        sessionId: fullSession.id.substring(0, 8),
        hasPersonaHistories: !!fullSession.personaHistories,
        personaHistoriesKeys: fullSession.personaHistories ? Object.keys(fullSession.personaHistories) : [],
        hasSystemMessages: !!fullSession.systemMessages,
        systemMessagesCount: fullSession.systemMessages?.length || 0,
      });

      // 3. メッセージ履歴を復元
      const restoredMessages = convertSessionToMessages(fullSession, userNickname);
      console.log('[App] convertSessionToMessages result:', {
        messagesCount: restoredMessages.length,
        firstMessage: restoredMessages[0] ? {
          id: restoredMessages[0].id,
          type: restoredMessages[0].type,
          author: restoredMessages[0].author,
        } : null,
      });

      // 4. タブを開く
      void openTab(fullSession, restoredMessages, session.workspaceId); // Phase 2: openTab is now async

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
      // タブを先に閉じる（Session削除前にタブを閉じることで"Unknown Session"表示を防ぐ）
      const tab = tabs.find(t => t.sessionId === sessionId);
      if (tab) {
        await closeTab(tab.id);
      }

      // Sessionを削除
      await deleteSession(sessionId);

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

      // Phase 3: renameSession が SessionContext を更新するため、
      // computed tabs は自動的に更新される。updateTabTitle は冗長だが互換性のために保持
      const tab = tabs.find(t => t.sessionId === sessionId);
      if (tab) {
        void updateTabTitle(tab.id, newTitle); // Phase 3: updateTabTitle is now async
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
        .filter(s => s.isFavorite && !s.isArchived)
        .sort((a, b) => {
          if (a.sortOrder !== undefined && b.sortOrder !== undefined) {
            return a.sortOrder - b.sortOrder;
          }
          if (a.sortOrder !== undefined) return -1;
          if (b.sortOrder !== undefined) return 1;
          return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
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
      // Use current workspace ID if available, otherwise fallback to default (handled by SessionContext)
      await createSession(workspace?.id);
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

  const handleConversationModeChange = async (mode: string) => {
    // Delegate to Store (which handles service layer)
    await updateConversationMode(mode, addMessage);
  };

  const handleTalkStyleChange = async (value: string | null) => {
    const style = value || null;

    // Delegate to Store (which handles service layer)
    await updateTalkStyle(style, addMessage);
  };


  const handleStrategyChange = async (strategy: string) => {
    // Delegate to Store (which handles service layer)
    await updateExecutionStrategy(strategy, addMessage);
  };

  const handleContextModeChange = async (mode: 'rich' | 'clean') => {
    // Delegate to Store (which handles service layer)
    await updateContextMode(mode, addMessage);
  };

  const handleToggleParticipant = async (personaId: string, isChecked: boolean) => {
    // Get persona name from personaStore
    const persona = personas.find(p => p.id === personaId);
    if (!persona) {
      console.error('[App] Persona not found:', personaId);
      return;
    }

    // Delegate to Store
    await toggleParticipant(personaId, isChecked, persona.name, addMessage);
  };

  const handleApplyPreset = async (presetId: string) => {
    try {
      // Apply preset via backend
      await invoke('apply_dialogue_preset', { presetId });

      // Find the preset to show success message
      const preset = dialoguePresets.find(p => p.id === presetId);
      if (preset) {
        // Note: conversationMode, talkStyle, executionStrategy are managed by Store
        // Reload settings to reflect preset changes
        await loadSessionSettings(currentSessionId || '');

        await handleAndPersistSystemMessage(
          conversationMessage(`プリセット「${preset.name}」を適用しました`, 'success'),
          addMessage,
          invoke
        );
      }
    } catch (error) {
      console.error(error);
      await handleAndPersistSystemMessage(
        conversationMessage(`Failed to apply preset: ${error}`, 'error'),
        addMessage,
        invoke
      );
    }
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
      // Search by both mention text and search name (with _ → space conversion)
      // Example: "@Ayaka_Nakamura" matches persona "Ayaka Nakamura"
      const persona = personas.find(p =>
        p.name === mention.mentionText || p.name === mention.searchName
      );

      if (persona && !activeParticipantIds.includes(persona.id)) {
        try {
          await invoke('add_participant', { personaId: persona.id });
          addMessage('system', 'System', `${persona.name} が参加しました`);
          // Refresh participants list to update active participant IDs
          await refreshActiveParticipants();
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
      padding={0}
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
          onSaveSessionToWorkspace={handleSaveSessionToWorkspace}
          tasks={tasks}
          taskProgress={taskProgress}
          onTaskToggle={handleTaskToggle}
          onTaskDelete={handleTaskDelete}
          onRefreshTasks={refreshTasks}
          onSaveTaskToWorkspace={handleSaveTaskToWorkspace}
          onAttachFile={handleAttachFileFromWorkspace}
          includeWorkspaceInPrompt={includeWorkspaceInPrompt}
          onToggleIncludeWorkspaceInPrompt={setIncludeWorkspaceInPrompt}
          onGoToSession={handleGoToSessionFromFile}
          onNewSessionWithFile={handleNewSessionWithFile}
          onMessage={addMessage}
          onSlashCommandsUpdated={refreshCustomCommands}
          onRunSlashCommand={handleRunSlashCommand}
          onConversationModeChange={handleConversationModeChange}
          onTalkStyleChange={handleTalkStyleChange}
          onStrategyChange={handleStrategyChange}
          onToggleParticipant={handleToggleParticipant}
          personas={personas}
          activeParticipantIds={activeParticipantIds}
          executionStrategy={executionStrategy}
          conversationMode={conversationMode}
          talkStyle={talkStyle}
          onRefreshPersonas={loadPersonas}
          onRefreshSessions={refreshSessions}
        />
      </AppShell.Navbar>

      {/* メインコンテンツ */}
      <AppShell.Main>
        <Container size="md" h="100vh" p="md" style={{ display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <Stack style={{ flex: 1, minHeight: 0 }} gap="md">
            <Paper
              p="sm"
              radius="md"
              style={{
                backgroundColor: debugSettings?.enableLlmDebug
                  ? 'var(--mantine-color-orange-1)'
                  : undefined,
                borderLeft: debugSettings?.enableLlmDebug
                  ? '4px solid var(--mantine-color-orange-6)'
                  : undefined,
              }}
            >
              <Group gap="sm" justify="space-between">
                <Group gap="sm">
                  <Burger opened={navbarOpened} onClick={toggleNavbar} size="sm" />
                  <Text size="xl" fw={700}>ORCS</Text>
                  {debugSettings?.enableLlmDebug && (
                    <Badge size="sm" variant="filled" color="orange">
                      DEBUG MODE
                    </Badge>
                  )}
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

                {/* Session Info: Removed - redundant with TabName */}

                {/* Settings Menu */}
                <SettingsMenu
                  onSelectSession={handleSessionSelect}
                />
              </Group>
            </Group>
            </Paper>

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
            // 全タブを表示（現在WSを末尾に、orderで並べる）
            const currentWsId = workspace?.id;
            const sortedTabs = [...tabs].sort((a, b) => {
              const aIsCurrent = a.workspaceId === currentWsId;
              const bIsCurrent = b.workspaceId === currentWsId;
              // 現在WSを末尾に（右端に配置）
              if (aIsCurrent && !bIsCurrent) return 1;
              if (!aIsCurrent && bIsCurrent) return -1;
              // それ以外はorderフィールドで並べる（ドラッグ&ドロップの順序を保持）
              return a.order - b.order;
            });

            // Workspace名取得用ヘルパー
            const getWorkspaceName = (wsId: string): string => {
              return allWorkspaces.find(w => w.id === wsId)?.name || 'Unknown';
            };
            const getWorkspaceBadge = (wsId: string): string => {
              const name = getWorkspaceName(wsId);
              return name.slice(0, 3).toUpperCase();
            };

            // タブホバー時にセッション内容のプレビューを生成
            const handleTabHover = (tab: typeof sortedTabs[0]) => {
              if (tabPreviewCache[tab.id]) return;

              const session = sessions.find(s => s.id === tab.sessionId);
              if (!session) return;

              const allMessages = getAllMessages(session);
              const nonSystemMessages = allMessages.filter(msg => msg.role !== 'System');
              if (nonSystemMessages.length === 0) return;

              const firstMessage = nonSystemMessages[0];
              const preview = firstMessage.content.slice(0, 50).trim();

              setTabPreviewCache(prev => ({
                ...prev,
                [tab.id]: preview,
              }));
            };

            return (
              <Tabs
                value={activeTabId}
                onChange={async (value) => {
                  if (!value) return;

                  const tab = tabs.find(t => t.id === value);
                  if (!tab) return;

                  // 1. タブを切り替え
                  switchToTab(value);

                  // 2. バックエンドのセッションも切り替え
                  try {
                    await switchSession(tab.sessionId);
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
                    try {
                      await switchWorkspaceBackend(tab.sessionId, tab.workspaceId);
                    } catch (err) {
                      console.error('[App] Failed to switch workspace:', err);
                      notifications.show({
                        title: 'Workspace Switch Failed',
                        message: String(err),
                        color: 'red',
                      });
                    }
                  }
                }}
                style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}
              >
                <Tabs.List style={{ overflowX: 'auto', flexWrap: 'nowrap', display: 'flex', alignItems: 'center', gap: '4px' }}>
                  {sortedTabs.map((tab) => {
                    const isOtherWorkspace = tab.workspaceId !== currentWsId;
                    const wsName = getWorkspaceName(tab.workspaceId);
                    const wsBadge = getWorkspaceBadge(tab.workspaceId);

                    return (
                    <Tooltip
                      key={tab.id}
                      label={tabPreviewCache[tab.id] || `${wsName} / ${tab.title}`}
                      withArrow
                      position="bottom"
                      multiline
                      w={220}
                    >
                    <Tabs.Tab
                      value={tab.id}
                      data-tab-id={tab.id}
                      onMouseEnter={() => handleTabHover(tab)}
                      style={{
                        minWidth: isOtherWorkspace ? '150px' : '120px',
                        maxWidth: '220px',
                        flexShrink: 0,
                      }}
                      leftSection={
                        isOtherWorkspace ? (
                          <Badge size="xs" variant="light" color="gray" style={{ minWidth: '35px', padding: '2px 5px', fontSize: '9px', textAlign: 'center' }}>
                            {wsBadge}
                          </Badge>
                        ) : tab.isDirty ? '●' : undefined
                      }
                        rightSection={
                          sortedTabs.length > 1 ? (
                            <Box
                              component="span"
                              style={{
                                display: 'inline-flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                width: '18px',
                                height: '18px',
                                borderRadius: '4px',
                                color: '#868e96',
                                cursor: 'pointer',
                                transition: 'background-color 0.1s, color 0.1s',
                              }}
                              onMouseEnter={(e) => {
                                e.currentTarget.style.backgroundColor = '#dee2e6';
                                e.currentTarget.style.color = '#212529';
                              }}
                              onMouseLeave={(e) => {
                                e.currentTarget.style.backgroundColor = 'transparent';
                                e.currentTarget.style.color = '#868e96';
                              }}
                              onClick={async (e) => {
                                e.stopPropagation();

                                // 未保存の場合は確認
                                if (tab.isDirty) {
                                  setClosingTabId(tab.id);
                                  return;
                                }

                                // 1. 閉じるタブの情報を取得
                                const closingTab = tabs.find(t => t.id === tab.id);
                                if (!closingTab) return;

                                // 2. ActiveSessionのタブを閉じる場合
                                const isClosingActiveSession = closingTab.sessionId === currentSessionId;

                                // 3. ActiveSessionだった場合、次のSessionを選択
                                if (isClosingActiveSession && workspace) {
                                  // 4a. 現在のWorkspace内の残りSession取得
                                  const remainingSessions = sessions.filter(
                                    s => s.workspaceId === workspace.id && s.id !== closingTab.sessionId
                                  );

                                  if (remainingSessions.length > 0) {
                                    // 4b. 更新日時が直近のSessionを選択
                                    const sortedSessions = [...remainingSessions].sort(
                                      (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
                                    );
                                    const nextSession = sortedSessions[0];

                                    try {
                                      // 4c. Backend Session切り替え
                                      await switchSession(nextSession.id);

                                      // 4d. タブを開く
                                      const messages = convertSessionToMessages(nextSession, userNickname);
                                      void openTab(nextSession, messages, workspace.id); // Phase 2: openTab is now async

                                      // 4f. 古いタブを閉じる（次のSessionに切り替え後）
                                      await closeTab(tab.id);
                                    } catch (err) {
                                      console.error('[App] Failed to switch to next session:', err);
                                    }
                                  } else {
                                    // 4e. Workspace内にSessionがない場合、新規作成
                                    try {
                                      await createSession(workspace?.id);
                                    } catch (err) {
                                      console.error('[App] Failed to create new session:', err);
                                    }
                                  }
                                } else {
                                  // 非ActiveSessionのTab Closeの場合、単純に閉じる
                                  await closeTab(tab.id);
                                }
                              }}
                            >
                              ✕
                            </Box>
                          ) : undefined
                        }
                    >
                      <Text truncate style={{ maxWidth: '100%' }}>
                        {tab.title}
                      </Text>
                    </Tabs.Tab>
                    </Tooltip>
                    );
                  })}

                  {/* 新規セッション追加ボタン */}
                  <Tooltip label="New Session" withArrow>
                    <ActionIcon
                      variant="subtle"
                      color="blue"
                      size="md"
                      onClick={async () => {
                        await createSession(workspace?.id);
                      }}
                      style={{ marginLeft: '8px' }}
                    >
                      <IconPlus size={16} />
                    </ActionIcon>
                  </Tooltip>
                </Tabs.List>

                {sortedTabs.map((tab) => (
                  <Tabs.Panel key={tab.id} value={tab.id} style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
                    <ChatPanel
                      tab={tab}
                      isActive={activeTabId === tab.id}
                      currentSessionId={currentSessionId}
                      status={status}
                      userNickname={userNickname}
              gitInfo={gitInfo}
              autoMode={autoMode}
              conversationMode={conversationMode}
              talkStyle={talkStyle}
              executionStrategy={executionStrategy}
              contextMode={contextMode}
              sandboxState={activeTabId === tab.id ? currentSandboxState : null}
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
                      onInputChange={handleInputChange}
                      onKeyDown={handleKeyDown}
                      onFileSelect={handleFileSelect}
                      onRemoveFile={removeAttachedFile}
                      onDragOver={handleDragOver}
                      onDragLeave={handleDragLeave}
                      onDrop={handleDrop}
                      onSaveMessageToWorkspace={handleSaveMessageToWorkspace}
                      onExecuteAsTask={handleExecuteAsTask}
                      onAutoModeChange={setAutoMode}
                      onTalkStyleChange={handleTalkStyleChange}
                      onExecutionStrategyChange={handleStrategyChange}
                      onConversationModeChange={handleConversationModeChange}
                      onContextModeChange={handleContextModeChange}
                      onToggleParticipant={handleToggleParticipant}
                      dialoguePresets={dialoguePresets}
                      onApplyPreset={handleApplyPreset}
                      onMentionPersona={handleMentionPersona}
                      onInvokePersona={handleInvokePersona}
                      onSelectCommand={selectCommand}
                      onSelectAgent={selectAgent}
                      onHoverSuggestion={setSelectedSuggestionIndex}
                      onSaveSessionToWorkspace={() => {
                        const session = sessions.find(s => s.id === tab.sessionId);
                        if (session) {
                          void handleSaveSessionToWorkspace(session);
                        }
                      }}
                      onPasteAndAttach={handlePasteAndAttach}
                    />
                  </Tabs.Panel>
                ))}
              </Tabs>
            );
          })()}
          </Stack>
        </Container>
      </AppShell.Main>

      {/* Close tab confirmation modal */}
      <Modal
        opened={!!closingTabId}
        onClose={handleCancelCloseTab}
        title="Close Tab"
        centered
        size="sm"
      >
        <Stack gap="md">
          <Text size="sm">
            "{tabs.find(t => t.id === closingTabId)?.title}" has unsaved changes. Close anyway?
          </Text>
          <Text size="xs" c="dimmed">
            Any unsaved changes will be lost.
          </Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={handleCancelCloseTab}>
              Cancel
            </Button>
            <Button color="red" onClick={handleConfirmCloseTab}>
              Close
            </Button>
          </Group>
        </Stack>
      </Modal>

      {/* Sandbox exit confirmation modal */}
      <Modal
        opened={sandboxExitModalOpened}
        onClose={handleCancelSandboxExit}
        title="Exit Sandbox Mode"
        centered
        size="md"
      >
        <Stack gap="md">
          <Text size="sm">
            You are currently in sandbox mode.
          </Text>
          {pendingSandboxExit && (
            <Paper p="sm" bg="dark.6" style={{ fontFamily: 'monospace', fontSize: '0.85rem' }}>
              <Text size="xs" c="gray.4">Branch:</Text>
              <Text size="sm" c="white">{pendingSandboxExit.sandbox_branch}</Text>
              <Text size="xs" c="gray.4" mt="xs">Worktree:</Text>
              <Text size="sm" c="white" style={{ wordBreak: 'break-all' }}>
                {pendingSandboxExit.worktree_path}
              </Text>
            </Paper>
          )}
          <Text size="sm" fw={500}>
            What would you like to do with your changes?
          </Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={handleCancelSandboxExit}>
              Cancel
            </Button>
            <Button color="red" onClick={handleSandboxExitDiscard}>
              Discard Changes
            </Button>
            <Button color="blue" onClick={handleSandboxExitMerge}>
              Merge Changes
            </Button>
          </Group>
        </Stack>
      </Modal>
    </AppShell>
  );
}

export default App;
