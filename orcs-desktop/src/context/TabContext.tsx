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

  // タブ操作 (Phase 2: Backend-First)
  openTab: (session: Session, messages: Message[], workspaceId: string, switchToTab?: boolean) => Promise<string>; // 新規タブを開く。既に開いている場合はフォーカス
  initializeTabUIState: (tabId: string, sessionId: string, workspaceId: string, title: string, messages: Message[]) => void; // 既存のタブIDでタブUI状態を初期化（Backend復元用） - Phase 3で削除予定
  closeTab: (tabId: string) => Promise<void>; // タブを閉じる
  switchTab: (tabId: string) => Promise<void>; // タブを切り替える
  switchWorkspace: (workspaceId: string) => Promise<void>; // Workspace切り替え時にタブを切り替える
  
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
  reorderTabs: (fromIndex: number, toIndex: number) => Promise<void>; // タブの順序を変更
  closeAllTabs: () => Promise<void>; // 全タブを閉じる
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

  // Phase 2: フロントエンド専用のUI状態を Map で管理
  const [tabUIStates, setTabUIStates] = useState<Map<string, TabUIState>>(new Map());

  // Phase 2: AppState.openTabs と Sessions と TabUIStates から SessionTab を動的生成
  // これが tabs の SSOT となる（Backend-First Pattern）
  const tabs = useMemo<SessionTab[]>(() => {
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

  // Phase 2: activeTabId は AppState から取得（Backend SSOT）
  const activeTabId = appState?.activeTabId ?? null;

  /**
   * 新規タブを開く（既に開いている場合はフォーカス）
   * Phase 2: Backend (AppStateStore) に委譲
   */
  const openTab = useCallback(async (session: Session, messages: Message[], workspaceId: string, switchToTab: boolean = true): Promise<string> => {
    console.log('[TabContext] openTab called:', {
      sessionId: session.id.substring(0, 8),
      messagesCount: messages.length,
      workspaceId: workspaceId.substring(0, 8),
      switchToTab,
    });

    // Phase 2: Backend (AppStateStore) を呼び出してタブを開く
    // Backend が既存タブのチェックと新規作成を行う
    const tabId = await useAppStateStore.getState().openTab(session.id, workspaceId);

    console.log('[TabContext] Tab opened by backend:', tabId);

    // Phase 2: UIState を初期化（既に存在する場合はスキップ）
    setTabUIStates((prev) => {
      if (!prev.has(tabId)) {
        const newMap = new Map(prev);
        newMap.set(tabId, getDefaultTabUIState());
        return newMap;
      }
      return prev;
    });

    // Note: Backend が app-state:update イベントを発火し、activeTabId も自動更新される
    // switchToTab パラメータは現在無視される（Backend は常にタブをアクティブにする）

    return tabId;
  }, []);

  /**
   * 既存のタブIDでタブUI状態を初期化（Backend復元用）
   * Phase 2: Backend が tabs を管理するため、UIState のみを初期化
   * Phase 3: この関数は削除予定（Backend からの復元で自動的に UIState が初期化される）
   */
  const initializeTabUIState = useCallback((
    tabId: string,
    _sessionId: string,
    _workspaceId: string,
    _title: string,
    _messages: Message[]
  ) => {
    console.log('[TabContext] initializeTabUIState called (Phase 2: UIState only):', {
      tabId: tabId.substring(0, 8),
    });

    // Phase 2: UIStateを初期化（既存タブでなければ）
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
   * Phase 2: Backend (AppStateStore) に委譲
   */
  const closeTab = useCallback(async (tabId: string) => {
    console.log('[TabContext] closeTab called:', { tabId });

    // Phase 2: Backend (AppStateStore) を呼び出してタブを閉じる
    await useAppStateStore.getState().closeTab(tabId);

    // Phase 2: UIStateもクリーンアップ
    setTabUIStates((prev) => {
      const newMap = new Map(prev);
      newMap.delete(tabId);
      return newMap;
    });

    // Note: Backend が app-state:update イベントを発火し、activeTabId も自動更新される
    console.log('[TabContext] Tab closed by backend:', tabId);
  }, []);

  /**
   * タブを切り替える
   * Phase 2: Backend (AppStateStore) に委譲
   */
  const switchTab = useCallback(async (tabId: string) => {
    console.log('[TabContext] switchTab called:', { tabId });

    // Phase 2: Backend (AppStateStore) を呼び出してタブを切り替える
    await useAppStateStore.getState().setActiveTab(tabId);

    // コールバック実行（workspaceId を tabs から取得）
    const targetTab = tabs.find(t => t.id === tabId);
    if (targetTab && onTabSwitched) {
      onTabSwitched(tabId, targetTab.workspaceId);
    }

    // Note: Backend が app-state:update イベントを発火し、activeTabId とタブの lastAccessedAt が自動更新される
    console.log('[TabContext] Tab switched by backend:', tabId);
  }, [tabs, onTabSwitched]);

  /**
   * タブのメッセージを更新
   * Phase 2: Backend (Session) の更新が必要 - 現在は No-op
   * TODO Phase 3: Backend の Session を更新する実装に変更
   */
  const updateTabMessages = useCallback((_tabId: string, _messages: Message[]) => {
    console.log('[TabContext] updateTabMessages: Phase 2 - No-op (Backend Session update required)');
    // Phase 2: tabs は computed なので直接更新できない
    // Phase 3 で Backend の Session を更新する実装に変更する
  }, []);

  /**
   * タブにメッセージを追加
   * Phase 2: Backend (Session) の更新が必要 - 現在は No-op
   * TODO Phase 3: Backend の Session を更新する実装に変更
   */
  const addMessageToTab = useCallback((_tabId: string, _message: Message) => {
    console.log('[TabContext] addMessageToTab: Phase 2 - No-op (Backend Session update required)');
    // Phase 2: tabs は computed なので直接更新できない
    // Phase 3 で Backend の Session を更新する実装に変更する
  }, []);

  /**
   * タブのタイトルを更新
   * Phase 2: Backend (Session) の更新が必要 - 現在は No-op
   * TODO Phase 3: Backend の Session を更新する実装に変更
   */
  const updateTabTitle = useCallback((_tabId: string, _title: string) => {
    console.log('[TabContext] updateTabTitle: Phase 2 - No-op (Backend Session update required)');
    // Phase 2: tabs は computed なので直接更新できない
    // Phase 3 で Backend の Session を更新する実装に変更する
  }, []);

  /**
   * タブのdirtyフラグを更新
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const setTabDirty = useCallback((tabId: string, isDirty: boolean) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const updateTabInput = useCallback((tabId: string, input: string) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const updateTabAttachedFiles = useCallback((tabId: string, files: File[]) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const addAttachedFileToTab = useCallback((tabId: string, file: File) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const removeAttachedFileFromTab = useCallback((tabId: string, index: number) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const setTabDragging = useCallback((tabId: string, isDragging: boolean) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const setTabThinking = useCallback((tabId: string, isThinking: boolean, personaName: string = 'AI') => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const setTabAutoMode = useCallback((tabId: string, autoMode: boolean) => {
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
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   */
  const setTabAutoChatIteration = useCallback((tabId: string, iteration: number | null) => {
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
   * Phase 2: Backend (AppStateStore) に委譲
   */
  const reorderTabs = useCallback(async (fromIndex: number, toIndex: number) => {
    console.log('[TabContext] reorderTabs called:', { fromIndex, toIndex });
    await useAppStateStore.getState().reorderTabs(fromIndex, toIndex);
    // Note: Backend が app-state:update イベントを発火し、tabs の順序が自動更新される
  }, []);

  /**
   * 全タブを閉じる
   * Phase 2: Backend の全タブを閉じるAPIがないため、個別にcloseTabを呼ぶ
   */
  const closeAllTabs = useCallback(async () => {
    console.log('[TabContext] closeAllTabs called');
    // Phase 2: 全タブを個別に閉じる
    const tabIds = tabs.map(tab => tab.id);
    for (const tabId of tabIds) {
      await closeTab(tabId);
    }
    // UIState は closeTab で個別にクリアされる
  }, [tabs, closeTab]);

  /**
   * Workspace切り替え時にタブを切り替える
   * - 新しいWorkspaceのタブがあれば、最後にアクセスしたタブにフォーカス
   * - なければアクティブタブはBackendがnullに設定する
   * Phase 2: Backend (AppStateStore) の setActiveTab を使用
   */
  const switchWorkspace = useCallback(async (workspaceId: string) => {
    const workspaceTabs = tabs.filter((tab) => tab.workspaceId === workspaceId);

    if (workspaceTabs.length > 0) {
      // 最後にアクセスしたタブを探す
      const sortedTabs = [...workspaceTabs].sort((a, b) => b.lastAccessedAt - a.lastAccessedAt);
      await useAppStateStore.getState().setActiveTab(sortedTabs[0].id);
    }
    // Note: タブがない場合、activeTabId は Backend が管理しているため、ここでは何もしない
    // App側で新しいタブを開くロジックが必要
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

