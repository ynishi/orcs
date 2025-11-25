import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';
import type { Session } from '../types/session';
import type { Message } from '../types/message';
import { useAppStateStore } from '../stores/appStateStore';
import { convertSessionToMessages } from '../types/session';
import { useSessionContext } from './SessionContext';

/**
 * フロントエンド専用のタブUI状態
 * - Backend (OpenTab) には永続化されない一時的なUI状態
 * - タブごとに独立して管理される
 */
export interface TabUIState {
  // 入力フォーム状態
  input: string; // 入力中のテキスト
  attachedFiles: File[]; // 添付ファイル

  // UI状態
  isDragging: boolean; // ドラッグ中かどうか
  isAiThinking: boolean; // AI思考中かどうか
  thinkingPersona: string; // 思考中のペルソナ名

  // AutoChat状態
  autoMode: boolean; // AutoChatモードが有効かどうか
  autoChatIteration: number | null; // 現在のAutoChat iteration番号 (null = 未実行)

  // メタデータ
  isDirty: boolean; // 未保存データがあるか
}

/**
 * デフォルトのTabUIStateを返す
 */
function getDefaultTabUIState(): TabUIState {
  return {
    input: '',
    attachedFiles: [],
    isDragging: false,
    isAiThinking: false,
    thinkingPersona: 'AI',
    autoMode: false,
    autoChatIteration: null,
    isDirty: false,
  };
}

/**
 * タブの情報（Aggregated State）
 * - Backend (OpenTab) + Session + TabUIState の統合ビュー
 * - SessionContextと同じく、薄いState層としてModel（Backend）とView（UI）の境界を明確化
 */
export interface SessionTab {
  // From Backend (OpenTab)
  id: string; // タブのユニークID
  sessionId: string; // 対応するセッションID
  workspaceId: string; // このタブが属するWorkspace ID
  lastAccessedAt: number; // 最終アクセス時刻

  // From Session (joined by sessionId)
  title: string; // タブタイトル
  messages: Message[]; // タブ固有のメッセージ履歴

  // From TabUIState (frontend-only)
  input: string; // 入力中のテキスト
  attachedFiles: File[]; // 添付ファイル
  isDragging: boolean; // ドラッグ中かどうか
  isAiThinking: boolean; // AI思考中かどうか
  thinkingPersona: string; // 思考中のペルソナ名
  autoMode: boolean; // AutoChatモードが有効かどうか
  autoChatIteration: number | null; // 現在のAutoChat iteration番号 (null = 未実行)
  isDirty: boolean; // 未保存データがあるか
}

export interface TabContextValue {
  tabs: SessionTab[];
  activeTabId: string | null;
  
  // タブ操作
  openTab: (session: Session, messages: Message[], workspaceId: string, switchToTab?: boolean) => string; // 新規タブを開く。既に開いている場合はフォーカス
  initializeTabUIState: (tabId: string, sessionId: string, workspaceId: string, title: string, messages: Message[]) => void; // 既存のタブIDでタブUI状態を初期化（Backend復元用）
  closeTab: (tabId: string) => void; // タブを閉じる
  switchTab: (tabId: string) => void; // タブを切り替える
  switchWorkspace: (workspaceId: string) => void; // Workspace切り替え時にタブを切り替える
  
  // メッセージ関連
  updateTabMessages: (tabId: string, messages: Message[]) => void; // タブのメッセージを更新
  addMessageToTab: (tabId: string, message: Message) => void; // タブにメッセージを追加
  
  // タブメタデータ
  updateTabTitle: (tabId: string, title: string) => void; // タブのタイトルを更新
  setTabDirty: (tabId: string, isDirty: boolean) => void; // タブのdirtyフラグを更新
  
  // 入力フォーム状態
  updateTabInput: (tabId: string, input: string) => void; // タブの入力テキストを更新
  updateTabAttachedFiles: (tabId: string, files: File[]) => void; // タブの添付ファイルを更新
  addAttachedFileToTab: (tabId: string, file: File) => void; // タブに添付ファイルを追加
  removeAttachedFileFromTab: (tabId: string, index: number) => void; // タブから添付ファイルを削除
  
  // UI状態
  setTabDragging: (tabId: string, isDragging: boolean) => void; // タブのドラッグ状態を更新
  setTabThinking: (tabId: string, isThinking: boolean, personaName?: string) => void; // タブのAI思考状態を更新

  // AutoChat状態
  setTabAutoMode: (tabId: string, autoMode: boolean) => void; // タブのAutoMode状態を更新
  setTabAutoChatIteration: (tabId: string, iteration: number | null) => void; // タブのAutoChat iteration状態を更新

  // ヘルパー
  getTab: (tabId: string) => SessionTab | undefined; // タブを取得
  getTabBySessionId: (sessionId: string) => SessionTab | undefined; // セッションIDからタブを取得
  getActiveTab: () => SessionTab | undefined; // アクティブなタブを取得
  getVisibleTabs: (workspaceId: string) => SessionTab[]; // 指定されたWorkspaceのタブのみを取得
  reorderTabs: (fromIndex: number, toIndex: number) => void; // タブの順序を変更
  closeAllTabs: () => void; // 全タブを閉じる
}

const TabContext = createContext<TabContextValue | undefined>(undefined);

interface TabProviderProps {
  children: ReactNode;
  onTabSwitched?: (tabId: string, workspaceId: string) => void;
}

export function TabProvider({ children, onTabSwitched }: TabProviderProps) {
  // Phase 2: Backend (AppState) を SSOT として取得
  const { appState } = useAppStateStore();
  const { sessions } = useSessionContext();

  // Phase 1: 既存の tabs state を維持しつつ、新しい tabUIStates を並行導入
  const [tabs, setTabs] = useState<SessionTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  // Phase 1: フロントエンド専用のUI状態を Map で管理
  const [tabUIStates, setTabUIStates] = useState<Map<string, TabUIState>>(new Map());

  // Phase 2: AppState.openTabs と Sessions と TabUIStates から SessionTab を動的生成
  const computedTabs = useMemo<SessionTab[]>(() => {
    if (!appState) return [];

    return appState.openTabs.map((openTab) => {
      // SessionsからSessionを取得
      const session = sessions.find((s) => s.id === openTab.sessionId);

      // TabUIStateを取得（なければデフォルト）
      const uiState = tabUIStates.get(openTab.id) ?? getDefaultTabUIState();

      // SessionTabを構築
      return {
        // From Backend (OpenTab)
        id: openTab.id,
        sessionId: openTab.sessionId,
        workspaceId: openTab.workspaceId,
        lastAccessedAt: openTab.lastAccessedAt,

        // From Session (joined by sessionId)
        title: session?.title ?? 'Unknown Session',
        messages: session ? convertSessionToMessages(session, 'You') : [],

        // From TabUIState (frontend-only)
        ...uiState,
      };
    });
  }, [appState, sessions, tabUIStates]);

  // Phase 1: tabUIStatesが未使用警告を回避（Phase 2で使用予定）
  if (tabUIStates.size < 0) {
    console.log('Phase 1: tabUIStates is being managed but not yet used for rendering');
  }

  /**
   * 新規タブを開く（既に開いている場合はフォーカス）
   */
  const openTab = useCallback((session: Session, messages: Message[], workspaceId: string, switchToTab: boolean = true): string => {
    console.log('[TabContext] openTab called:', {
      sessionId: session.id.substring(0, 8),
      messagesCount: messages.length,
      workspaceId: workspaceId.substring(0, 8),
    });

    let tabId: string = '';
    let isExistingTab = false;

    setTabs((prev) => {
      // 既に同じセッションのタブが開いているか確認
      const existingTab = prev.find((tab) => tab.sessionId === session.id);

      if (existingTab) {
        // 既存タブを更新
        console.log('[TabContext] Updating existing tab:', {
          tabId: existingTab.id.substring(existingTab.id.length - 8),
          oldMessagesCount: existingTab.messages.length,
          newMessagesCount: messages.length,
        });
        tabId = existingTab.id;
        isExistingTab = true;
        return prev.map((tab) =>
          tab.id === existingTab.id
            ? { ...tab, messages, title: session.title, lastAccessedAt: Date.now() }
            : tab
        );
      }

      // 新規タブを作成（デフォルト値で初期化）
      tabId = `tab-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
      console.log('[TabContext] Creating new tab:', {
        tabId: tabId.substring(tabId.length - 8),
        messagesCount: messages.length,
      });
      const newTab: SessionTab = {
        // セッション情報
        id: tabId,
        sessionId: session.id,
        workspaceId: workspaceId,
        title: session.title,

        // メッセージ関連
        messages,

        // 入力フォーム状態
        input: '',
        attachedFiles: [],

        // UI状態
        isDragging: false,
        isAiThinking: false,
        thinkingPersona: 'AI',

        // AutoChat状態
        autoMode: false,
        autoChatIteration: null,

        // メタデータ
        isDirty: false,
        lastAccessedAt: Date.now(),
      };

      return [...prev, newTab];
    });

    // Phase 1: 新規タブのUIStateを初期化
    if (!isExistingTab) {
      setTabUIStates((prev) => {
        const newMap = new Map(prev);
        newMap.set(tabId, getDefaultTabUIState());
        return newMap;
      });
    }

    // setTabs()の外で setActiveTabId() を呼ぶことで、確実に更新を反映
    // 既存タブの場合もswitchToTabがtrueならアクティブにする
    if (switchToTab) {
      setActiveTabId(tabId);
      if (isExistingTab) {
        console.log('[TabContext] Switched to existing tab:', tabId);
      }
    }

    return tabId;
  }, []);

  /**
   * 既存のタブIDでタブUI状態を初期化（Backend復元用）
   * - openTabと異なり、新しいIDを生成せずバックエンドから取得したIDをそのまま使用
   * - 主にアプリ起動時のタブ復元で使用
   */
  const initializeTabUIState = useCallback((
    tabId: string,
    sessionId: string,
    workspaceId: string,
    title: string,
    messages: Message[]
  ) => {
    console.log('[TabContext] initializeTabUIState called:', {
      tabId: tabId.substring(0, 8),
      sessionId: sessionId.substring(0, 8),
      workspaceId: workspaceId.substring(0, 8),
      title,
      messagesCount: messages.length,
    });

    setTabs((prev) => {
      // 既に同じIDのタブが存在する場合は更新
      const existingTab = prev.find((tab) => tab.id === tabId);

      if (existingTab) {
        console.log('[TabContext] Updating existing tab UI state:', tabId.substring(0, 8));
        return prev.map((tab) =>
          tab.id === tabId
            ? { ...tab, sessionId, workspaceId, title, messages, lastAccessedAt: Date.now() }
            : tab
        );
      }

      // 新規タブとして作成（デフォルト値で初期化）
      console.log('[TabContext] Creating new tab with preset ID:', tabId.substring(0, 8));
      const newTab: SessionTab = {
        // セッション情報
        id: tabId,
        sessionId,
        workspaceId,
        title,

        // メッセージ関連
        messages,

        // 入力フォーム状態
        input: '',
        attachedFiles: [],

        // UI状態
        isDragging: false,
        isAiThinking: false,
        thinkingPersona: 'AI',

        // AutoChat状態
        autoMode: false,
        autoChatIteration: null,

        // メタデータ
        isDirty: false,
        lastAccessedAt: Date.now(),
      };

      return [...prev, newTab];
    });

    // Phase 1: UIStateを初期化（既存タブでなければ）
    setTabUIStates((prev) => {
      if (!prev.has(tabId)) {
        const newMap = new Map(prev);
        newMap.set(tabId, getDefaultTabUIState());
        return newMap;
      }
      return prev;
    });
  }, []);

  /**
   * タブを閉じる
   */
  const closeTab = useCallback((tabId: string) => {
    console.log('[TabContext] closeTab called:', { tabId });

    // 削除前にタブ情報を取得
    const currentTabs = tabs;
    const targetIndex = currentTabs.findIndex((tab) => tab.id === tabId);

    if (targetIndex === -1) {
      console.log('[TabContext] closeTab: tab not found');
      return;
    }

    console.log('[TabContext] closeTab: targetIndex =', targetIndex, 'total tabs:', currentTabs.length);

    // タブを削除
    setTabs((prev) => {
      const newTabs = prev.filter((tab) => tab.id !== tabId);
      console.log('[TabContext] closeTab: newTabs count =', newTabs.length);
      return newTabs;
    });

    // Phase 1: UIStateもクリーンアップ
    setTabUIStates((prev) => {
      const newMap = new Map(prev);
      newMap.delete(tabId);
      return newMap;
    });

    // アクティブタブを閉じた場合、次のタブにフォーカス
    if (activeTabId === tabId) {
      const newTabs = currentTabs.filter((tab) => tab.id !== tabId);
      if (newTabs.length > 0) {
        const nextIndex = Math.min(targetIndex, newTabs.length - 1);
        const nextActiveId = newTabs[nextIndex].id;
        console.log('[TabContext] closeTab: updating activeTabId to', nextActiveId.substring(0, 8), 'at index', nextIndex);
        setActiveTabId(nextActiveId);
      } else {
        console.log('[TabContext] closeTab: no tabs left, setting activeTabId to null');
        setActiveTabId(null);
      }
    }
  }, [tabs, activeTabId]);

  /**
   * タブを切り替える
   */
  const switchTab = useCallback((tabId: string) => {
    let targetWorkspaceId: string | undefined;

    setTabs((prev) =>
      prev.map((tab) => {
        if (tab.id === tabId) {
          targetWorkspaceId = tab.workspaceId;
          return { ...tab, lastAccessedAt: Date.now() };
        }
        return tab;
      })
    );

    setActiveTabId(tabId);

    // コールバック実行
    if (targetWorkspaceId && onTabSwitched) {
      onTabSwitched(tabId, targetWorkspaceId);
    }
  }, [onTabSwitched]);

  /**
   * タブのメッセージを更新
   */
  const updateTabMessages = useCallback((tabId: string, messages: Message[]) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, messages } : tab))
    );
  }, []);

  /**
   * タブにメッセージを追加
   */
  const addMessageToTab = useCallback((tabId: string, message: Message) => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, messages: [...tab.messages, message] } : tab
      )
    );
  }, []);

  /**
   * タブのタイトルを更新
   */
  const updateTabTitle = useCallback((tabId: string, title: string) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, title } : tab))
    );
  }, []);

  /**
   * タブのdirtyフラグを更新
   */
  const setTabDirty = useCallback((tabId: string, isDirty: boolean) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, isDirty } : tab))
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, isDirty });
      return newMap;
    });
  }, []);

  /**
   * タブの入力テキストを更新
   */
  const updateTabInput = useCallback((tabId: string, input: string) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, input } : tab))
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, input });
      return newMap;
    });
  }, []);

  /**
   * タブの添付ファイルを更新
   */
  const updateTabAttachedFiles = useCallback((tabId: string, files: File[]) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, attachedFiles: files } : tab))
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, attachedFiles: files });
      return newMap;
    });
  }, []);

  /**
   * タブに添付ファイルを追加
   */
  const addAttachedFileToTab = useCallback((tabId: string, file: File) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, attachedFiles: [...tab.attachedFiles, file] } : tab
      )
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, attachedFiles: [...current.attachedFiles, file] });
      return newMap;
    });
  }, []);

  /**
   * タブから添付ファイルを削除
   */
  const removeAttachedFileFromTab = useCallback((tabId: string, index: number) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId
          ? { ...tab, attachedFiles: tab.attachedFiles.filter((_, i) => i !== index) }
          : tab
      )
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, attachedFiles: current.attachedFiles.filter((_, i) => i !== index) });
      return newMap;
    });
  }, []);

  /**
   * タブのドラッグ状態を更新
   */
  const setTabDragging = useCallback((tabId: string, isDragging: boolean) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, isDragging } : tab))
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, isDragging });
      return newMap;
    });
  }, []);

  /**
   * タブのAI思考状態を更新
   */
  const setTabThinking = useCallback((tabId: string, isThinking: boolean, personaName: string = 'AI') => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, isAiThinking: isThinking, thinkingPersona: personaName } : tab
      )
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, isAiThinking: isThinking, thinkingPersona: personaName });
      return newMap;
    });
  }, []);

  /**
   * タブのAutoMode状態を更新
   */
  const setTabAutoMode = useCallback((tabId: string, autoMode: boolean) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, autoMode } : tab
      )
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, autoMode });
      return newMap;
    });
  }, []);

  /**
   * タブのAutoChat iteration状態を更新
   */
  const setTabAutoChatIteration = useCallback((tabId: string, iteration: number | null) => {
    // Phase 1: 既存のsetTabsと並行してtabUIStatesも更新
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, autoChatIteration: iteration } : tab
      )
    );
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, autoChatIteration: iteration });
      return newMap;
    });
  }, []);

  /**
   * タブを取得
   */
  const getTab = useCallback((tabId: string): SessionTab | undefined => {
    return tabs.find((tab) => tab.id === tabId);
  }, [tabs]);

  /**
   * セッションIDからタブを取得
   */
  const getTabBySessionId = useCallback((sessionId: string): SessionTab | undefined => {
    return tabs.find((tab) => tab.sessionId === sessionId);
  }, [tabs]);

  /**
   * アクティブなタブを取得
   */
  const getActiveTab = useCallback((): SessionTab | undefined => {
    if (!activeTabId) return undefined;
    return tabs.find((tab) => tab.id === activeTabId);
  }, [tabs, activeTabId]);

  /**
   * タブの順序を変更
   */
  const reorderTabs = useCallback((fromIndex: number, toIndex: number) => {
    setTabs((prev) => {
      if (fromIndex < 0 || fromIndex >= prev.length || toIndex < 0 || toIndex >= prev.length) {
        return prev;
      }

      const newTabs = [...prev];
      const [movedTab] = newTabs.splice(fromIndex, 1);
      newTabs.splice(toIndex, 0, movedTab);
      return newTabs;
    });
  }, []);

  /**
   * 全タブを閉じる
   */
  const closeAllTabs = useCallback(() => {
    setTabs([]);
    setActiveTabId(null);
    // Phase 1: 全UIStateもクリア
    setTabUIStates(new Map());
  }, []);

  /**
   * Workspace切り替え時にタブを切り替える
   * - 新しいWorkspaceのタブがあれば、最後にアクセスしたタブにフォーカス
   * - なければアクティブタブをnullにする（Appで新しいセッションを開く）
   */
  const switchWorkspace = useCallback((workspaceId: string) => {
    const workspaceTabs = tabs.filter((tab) => tab.workspaceId === workspaceId);
    
    if (workspaceTabs.length > 0) {
      // 最後にアクセスしたタブを探す
      const sortedTabs = [...workspaceTabs].sort((a, b) => b.lastAccessedAt - a.lastAccessedAt);
      setActiveTabId(sortedTabs[0].id);
    } else {
      // このWorkspaceのタブがない場合はnullにする
      setActiveTabId(null);
    }
  }, [tabs]);

  /**
   * 指定されたWorkspaceのタブのみを取得
   */
  const getVisibleTabs = useCallback((workspaceId: string): SessionTab[] => {
    return tabs.filter((tab) => tab.workspaceId === workspaceId);
  }, [tabs]);

  const value = useMemo<TabContextValue>(
    () => ({
      tabs,
      activeTabId,

      // タブ操作
      openTab,
      initializeTabUIState,
      closeTab,
      switchTab,
      switchWorkspace,
      
      // メッセージ関連
      updateTabMessages,
      addMessageToTab,
      
      // タブメタデータ
      updateTabTitle,
      setTabDirty,
      
      // 入力フォーム状態
      updateTabInput,
      updateTabAttachedFiles,
      addAttachedFileToTab,
      removeAttachedFileFromTab,
      
      // UI状態
      setTabDragging,
      setTabThinking,

      // AutoChat状態
      setTabAutoMode,
      setTabAutoChatIteration,

      // ヘルパー
      getTab,
      getTabBySessionId,
      getActiveTab,
      getVisibleTabs,
      reorderTabs,
      closeAllTabs,
    }),
    [
      tabs,
      activeTabId,
      openTab,
      initializeTabUIState,
      closeTab,
      switchTab,
      switchWorkspace,
      updateTabMessages,
      addMessageToTab,
      updateTabTitle,
      setTabDirty,
      updateTabInput,
      updateTabAttachedFiles,
      addAttachedFileToTab,
      removeAttachedFileFromTab,
      setTabDragging,
      setTabThinking,
      setTabAutoMode,
      setTabAutoChatIteration,
      getTab,
      getTabBySessionId,
      getActiveTab,
      getVisibleTabs,
      reorderTabs,
      closeAllTabs,
    ]
  );

  return <TabContext.Provider value={value}>{children}</TabContext.Provider>;
}

export function useTabContext(): TabContextValue {
  const context = useContext(TabContext);
  if (!context) {
    throw new Error('useTabContext must be used within a TabProvider');
  }
  return context;
}

