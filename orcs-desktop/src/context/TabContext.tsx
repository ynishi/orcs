import { createContext, useCallback, useContext, useEffect, useMemo, useState, useRef, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
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
  isNonInteractiveCommand: boolean; // 非対話的コマンド実行中かどうか（ホバーメニュー等から）

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
    isNonInteractiveCommand: false,
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
  order: number; // タブの表示順序

  // From Session (joined by sessionId)
  title: string; // タブタイトル
  messages: Message[]; // タブ固有のメッセージ履歴

  // From TabUIState (frontend-only)
  input: string; // 入力中のテキスト
  attachedFiles: File[]; // 添付ファイル
  isDragging: boolean; // ドラッグ中かどうか
  isAiThinking: boolean; // AI思考中かどうか
  thinkingPersona: string; // 思考中のペルソナ名
  isNonInteractiveCommand: boolean; // 非対話的コマンド実行中かどうか
  autoMode: boolean; // AutoChatモードが有効かどうか
  autoChatIteration: number | null; // 現在のAutoChat iteration番号 (null = 未実行)
  isDirty: boolean; // 未保存データがあるか
}

/**
 * Performance: 入力状態を専用 Context に分離
 * activeTabInput が変わっても TabContext の consumer は再レンダリングされない
 */
export interface TabInputContextValue {
  activeTabInput: string;
}

export interface TabContextValue {
  tabs: SessionTab[];
  activeTabId: string | null;

  // タブ操作 (Phase 2: Backend-First)
  openTab: (session: Session, messages: Message[], workspaceId: string, switchToTab?: boolean) => Promise<string>; // 新規タブを開く。既に開いている場合はフォーカス
  closeTab: (tabId: string) => Promise<void>; // タブを閉じる
  switchTab: (tabId: string) => Promise<void>; // タブを切り替える
  switchWorkspace: (workspaceId: string) => Promise<void>; // Workspace切り替え時にタブを切り替える

  // メッセージ関連
  updateTabMessages: (tabId: string, messages: Message[]) => void; // タブのメッセージを更新
  addMessageToTab: (tabId: string, message: Message) => void; // タブにメッセージを追加

  // タブメタデータ
  updateTabTitle: (tabId: string, title: string) => Promise<void>; // タブのタイトルを更新 (Phase 3: async)
  setTabDirty: (tabId: string, isDirty: boolean) => void; // タブのdirtyフラグを更新

  // 入力フォーム状態
  updateTabInput: (tabId: string, input: string) => void; // タブの入力テキストを更新
  updateTabAttachedFiles: (tabId: string, files: File[]) => void; // タブの添付ファイルを更新
  addAttachedFileToTab: (tabId: string, file: File) => void; // タブに添付ファイルを追加
  removeAttachedFileFromTab: (tabId: string, index: number) => void; // タブから添付ファイルを削除

  // UI状態
  setTabDragging: (tabId: string, isDragging: boolean) => void; // タブのドラッグ状態を更新
  setTabThinking: (tabId: string, isThinking: boolean, personaName?: string, isNonInteractive?: boolean) => void; // タブのAI思考状態を更新

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
  closeWorkspaceTabs: (workspaceId: string) => Promise<void>; // 指定ワークスペースのタブを全て閉じる
}

const TabInputContext = createContext<TabInputContextValue | undefined>(undefined);
const TabContext = createContext<TabContextValue | undefined>(undefined);

interface TabProviderProps {
  children: ReactNode;
  onTabSwitched?: (tabId: string, workspaceId: string) => void;
}

export function TabProvider({ children, onTabSwitched }: TabProviderProps) {
  // Phase 2: Backend (AppState) を SSOT として取得
  const { appState } = useAppStateStore();
  const { sessions, renameSession } = useSessionContext();

  // Phase 2: フロントエンド専用のUI状態を Map で管理
  const [tabUIStates, setTabUIStates] = useState<Map<string, TabUIState>>(new Map());

  // Performance: input を Ref で分離（tabs の再計算を防ぐ）
  // state ではなく Ref を使うことで、入力変更が tabs の useMemo をトリガーしない
  const tabInputsRef = useRef<Map<string, string>>(new Map());
  // activeTabInput のみを state で管理（入力フィールドの再レンダリング用）
  const [activeTabInputState, setActiveTabInputState] = useState<string>('');

  // Phase 4: セッションメッセージをローカルで管理（リアルタイム更新用）
  // key: sessionId, value: Message[]
  // Backend永続化は saveCurrentSession 時に行う
  const [sessionMessages, setSessionMessages] = useState<Map<string, Message[]>>(new Map());

  // Phase 4: Session messages のキャッシュ（無限ループ防止）
  // key: sessionId, value: Message[]
  const sessionMessagesCache = useMemo(() => {
    const cache = new Map<string, Message[]>();
    sessions.forEach((session) => {
      cache.set(session.id, convertSessionToMessages(session, 'You'));
    });
    return cache;
  }, [sessions]);

  // Phase 2: AppState.openTabs と Sessions と TabUIStates から SessionTab を動的生成
  // Phase 4: messages は sessionMessages から取得（優先）、なければキャッシュから
  // Phase 2.4 (V1.6+): Backend保存されたUI状態を復元
  // これが tabs の SSOT となる（Backend-First Pattern）
  const tabs = useMemo<SessionTab[]>(() => {
    if (!appState) return [];

    return appState.openTabs.map((openTab) => {
      // SessionsからSessionを取得
      const session = sessions.find((s) => s.id === openTab.sessionId);

      // TabUIStateを取得
      const localUIState = tabUIStates.get(openTab.id);
      const defaultUIState = getDefaultTabUIState();

      // Phase 2.4 (V1.6+): ローカルUIStateとBackend UI状態をマージ
      // 優先順位: ローカル > Backend > デフォルト
      // Note: ローカルが未初期化の場合は Backend から復元（アプリ再起動対応）
      const mergedUIState: TabUIState = {
        // Performance: input は tabInputsRef から取得（Ref なので依存関係に含まれない）
        // これにより入力変更が tabs の再計算をトリガーしない
        input: tabInputsRef.current.get(openTab.id) ?? openTab.input ?? '',
        attachedFiles: localUIState?.attachedFiles ?? defaultUIState.attachedFiles,
        isDragging: localUIState?.isDragging ?? defaultUIState.isDragging,
        isAiThinking: localUIState?.isAiThinking ?? defaultUIState.isAiThinking,
        thinkingPersona: localUIState?.thinkingPersona ?? defaultUIState.thinkingPersona,
        isNonInteractiveCommand: localUIState?.isNonInteractiveCommand ?? defaultUIState.isNonInteractiveCommand,
        autoMode: openTab.autoMode ?? localUIState?.autoMode ?? defaultUIState.autoMode,
        autoChatIteration: openTab.autoChatIteration ?? localUIState?.autoChatIteration ?? defaultUIState.autoChatIteration,
        isDirty: openTab.isDirty ?? localUIState?.isDirty ?? defaultUIState.isDirty,
      };

      // Phase 4: sessionMessages から取得（優先）、なければキャッシュから
      const messages = sessionMessages.get(openTab.sessionId)
        ?? sessionMessagesCache.get(openTab.sessionId)
        ?? [];

      // SessionTabを構築
      return {
        // From Backend (OpenTab)
        id: openTab.id,
        sessionId: openTab.sessionId,
        workspaceId: openTab.workspaceId,
        lastAccessedAt: openTab.lastAccessedAt,
        order: openTab.order,

        // From Session (joined by sessionId)
        title: session?.title ?? 'Unknown Session',
        messages,

        // From TabUIState (merged from Backend + local)
        ...mergedUIState,
      };
    });
  }, [appState, sessions, tabUIStates, sessionMessages, sessionMessagesCache]);

  // Phase 2: activeTabId は AppState から取得（Backend SSOT）
  const activeTabId = appState?.activeTabId ?? null;

  // Performance: openTabs を Ref で保持（useEffect の依存配列から除外するため）
  const openTabsRef = useRef(appState?.openTabs);
  useEffect(() => {
    openTabsRef.current = appState?.openTabs;
  }, [appState?.openTabs]);

  // Performance: activeTabId 変更時に activeTabInputState を同期
  // tabInputsRef にない場合は Backend (openTabs) から復元
  useEffect(() => {
    if (activeTabId) {
      let input = tabInputsRef.current.get(activeTabId);
      if (input === undefined) {
        // Backend から復元（初回アクセス時）
        const openTab = openTabsRef.current?.find(t => t.id === activeTabId);
        input = openTab?.input ?? '';
        tabInputsRef.current.set(activeTabId, input);
      }
      setActiveTabInputState(input);
    } else {
      setActiveTabInputState('');
    }
  }, [activeTabId]);

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
    // V1.6+: Backend に保存された input を復元（アプリ再起動時のデグレ修正）
    // V1.7+: Backend に保存された attachedFilePaths からファイルを復元
    const initializeUIState = async () => {
      const currentUIState = tabUIStates.get(tabId);
      if (currentUIState) {
        // 既に初期化済み
        return;
      }

      // Backend から保存された状態を取得
      const backendTab = useAppStateStore.getState().appState?.openTabs.find((t) => t.id === tabId);
      const restoredInput = backendTab?.input ?? '';
      const attachedFilePaths = backendTab?.attachedFilePaths ?? [];

      console.log('[TabContext] Restoring state from backend:', {
        tabId,
        inputLength: restoredInput.length,
        attachedFilePathsCount: attachedFilePaths.length,
      });

      // 添付ファイルパスから File オブジェクトを復元
      const attachedFiles: File[] = [];
      for (const filePath of attachedFilePaths) {
        try {
          // ファイル内容を読み込み
          const fileData = await invoke<number[]>('read_workspace_file', { filePath });
          const uint8Array = new Uint8Array(fileData);

          // ファイル名を抽出
          const fileName = filePath.split('/').pop() || 'unknown';

          // MIME タイプを推定（拡張子から）
          const ext = fileName.split('.').pop()?.toLowerCase() || '';
          const mimeTypes: Record<string, string> = {
            'txt': 'text/plain',
            'md': 'text/markdown',
            'json': 'application/json',
            'js': 'text/javascript',
            'ts': 'text/typescript',
            'tsx': 'text/typescript',
            'jsx': 'text/javascript',
            'html': 'text/html',
            'css': 'text/css',
            'png': 'image/png',
            'jpg': 'image/jpeg',
            'jpeg': 'image/jpeg',
            'gif': 'image/gif',
            'svg': 'image/svg+xml',
            'pdf': 'application/pdf',
          };
          const mimeType = mimeTypes[ext] || 'application/octet-stream';

          // Blob から File を作成
          const blob = new Blob([uint8Array], { type: mimeType });
          const file = new File([blob], fileName, { type: mimeType });
          attachedFiles.push(file);

          console.log('[TabContext] Restored attached file:', { fileName, size: file.size });
        } catch (error) {
          console.error('[TabContext] Failed to restore attached file:', { filePath, error });
        }
      }

      setTabUIStates((prev) => {
        const newMap = new Map(prev);
        newMap.set(tabId, {
          ...getDefaultTabUIState(),
          input: restoredInput,
          attachedFiles,
        });
        return newMap;
      });
    };

    // 非同期で初期化（UI をブロックしない）
    initializeUIState().catch((error) => {
      console.error('[TabContext] Failed to initialize UI state:', error);
    });

    // Note: Backend が app-state:update イベントを発火し、activeTabId も自動更新される
    // switchToTab パラメータは現在無視される（Backend は常にタブをアクティブにする）

    return tabId;
  }, [tabUIStates]);

  // Phase 3: initializeTabUIState 関数を削除
  // Backend SSOT により、タブは自動的にレンダリングされるため不要

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
   * Phase 4: sessionMessages Map を更新（ローカル管理）
   */
  const updateTabMessages = useCallback((tabId: string, messages: Message[]) => {
    console.log('[TabContext] updateTabMessages:', { tabId, messagesCount: messages.length });

    // Phase 4: appState から sessionId を取得（tabs 依存を避けるため）
    const openTab = appState?.openTabs.find((t) => t.id === tabId);
    if (!openTab) {
      console.warn('[TabContext] Tab not found for updateTabMessages:', tabId);
      return;
    }

    // Phase 4: sessionMessages Map を更新
    setSessionMessages((prev) => {
      const newMap = new Map(prev);
      newMap.set(openTab.sessionId, messages);
      return newMap;
    });
  }, [appState]);

  /**
   * タブにメッセージを追加
   * Phase 4: sessionMessages Map にメッセージを追加（ローカル管理）
   */
  const addMessageToTab = useCallback((tabId: string, message: Message) => {
    console.log('[TabContext] addMessageToTab:', { tabId, messageId: message.id });

    // Phase 4: sessionMessages Map にメッセージを追加
    // appState から sessionId を取得（tabs 依存を避けるため）
    const openTab = appState?.openTabs.find((t) => t.id === tabId);
    if (!openTab) {
      console.warn('[TabContext] Tab not found for addMessageToTab:', tabId);
      return;
    }

    setSessionMessages((prev) => {
      const currentMessages = prev.get(openTab.sessionId) ?? [];
      // 既にメッセージがない場合は Session から取得
      if (currentMessages.length === 0) {
        const session = sessions.find((s) => s.id === openTab.sessionId);
        const sessionMessages = session ? convertSessionToMessages(session, 'You') : [];
        const newMap = new Map(prev);
        newMap.set(openTab.sessionId, [...sessionMessages, message]);
        return newMap;
      }

      const newMap = new Map(prev);
      newMap.set(openTab.sessionId, [...currentMessages, message]);
      return newMap;
    });
  }, [appState, sessions]);

  /**
   * タブのタイトルを更新
   * Phase 3: Backend Session の renameSession に委譲
   */
  const updateTabTitle = useCallback(async (tabId: string, title: string) => {
    console.log('[TabContext] updateTabTitle:', { tabId, title });

    // Get sessionId from tab
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) {
      console.warn('[TabContext] Tab not found for updateTabTitle:', tabId);
      return;
    }

    try {
      // Phase 3: Backend Session を更新（SessionContext経由）
      await renameSession(tab.sessionId, title);
      console.log('[TabContext] Title updated via backend session');
      // Note: SessionContext が sessions を更新し、computed tabs が自動的に更新される
    } catch (error) {
      console.error('[TabContext] Failed to update tab title:', error);
      throw error;
    }
  }, [tabs, renameSession]);

  /**
   * タブのdirtyフラグを更新
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   * Phase 2.3 (V1.6+): Backend同期 (immediate)
   */
  const setTabDirty = useCallback((tabId: string, isDirty: boolean) => {
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, isDirty });
      return newMap;
    });

    // Immediate Backend sync (not frequently updated)
    useAppStateStore.getState().updateTabUIState(tabId, { isDirty }).catch((error) => {
      console.error('[TabContext] Failed to sync isDirty to backend:', error);
    });
  }, []);

  /**
   * タブの入力テキストを更新
   * Performance: tabInputsRef (Ref) を更新し、activeTabInputState のみを state 更新
   * これにより tabs の再計算をトリガーせずに入力を即座に反映
   * Phase 2.2 (V1.6+): Backend同期 (debounce 500ms)
   */
  const updateTabInputTimerRef = useRef<Map<string, NodeJS.Timeout>>(new Map());

  const updateTabInput = useCallback((tabId: string, input: string) => {
    // Performance: Ref を更新（state 更新ではないので tabs の再計算をトリガーしない）
    tabInputsRef.current.set(tabId, input);

    // activeTabInput state のみを更新（アクティブタブの場合のみ）
    // Fix: 他タブの更新時に activeTabInputState が誤って上書きされるバグを修正
    if (tabId === activeTabId) {
      setActiveTabInputState(input);
    }

    // Debounced Backend sync (500ms)
    const timers = updateTabInputTimerRef.current;
    const existingTimer = timers.get(tabId);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    const timer = setTimeout(() => {
      console.log('[TabContext] Syncing input to backend (debounced):', { tabId, inputLength: input.length });
      useAppStateStore.getState().updateTabUIState(tabId, { input }).catch((error) => {
        console.error('[TabContext] Failed to sync input to backend:', error);
      });
      timers.delete(tabId);
    }, 500);

    timers.set(tabId, timer);
  }, [activeTabId]);

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
   * @param isNonInteractive - 非対話的コマンド（ホバーメニュー等）からの実行かどうか
   */
  const setTabThinking = useCallback((tabId: string, isThinking: boolean, personaName: string = 'AI', isNonInteractive: boolean = false) => {
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, {
        ...current,
        isAiThinking: isThinking,
        thinkingPersona: personaName,
        isNonInteractiveCommand: isNonInteractive
      });
      return newMap;
    });
  }, []);

  /**
   * タブのAutoMode状態を更新
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   * Phase 2.3 (V1.6+): Backend同期 (immediate)
   */
  const setTabAutoMode = useCallback((tabId: string, autoMode: boolean) => {
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, autoMode });
      return newMap;
    });

    // Immediate Backend sync (not frequently updated)
    useAppStateStore.getState().updateTabUIState(tabId, { autoMode }).catch((error) => {
      console.error('[TabContext] Failed to sync autoMode to backend:', error);
    });
  }, []);

  /**
   * タブのAutoChat iteration状態を更新
   * Phase 2: tabUIStates のみを更新（tabs は computed）
   * Phase 2.3 (V1.6+): Backend同期 (immediate)
   */
  const setTabAutoChatIteration = useCallback((tabId: string, iteration: number | null) => {
    setTabUIStates((prev) => {
      const current = prev.get(tabId);
      if (!current) return prev;
      const newMap = new Map(prev);
      newMap.set(tabId, { ...current, autoChatIteration: iteration });
      return newMap;
    });

    // Immediate Backend sync (not frequently updated)
    useAppStateStore.getState().updateTabUIState(tabId, { autoChatIteration: iteration }).catch((error) => {
      console.error('[TabContext] Failed to sync autoChatIteration to backend:', error);
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
   * 指定ワークスペースのタブを全て閉じる
   * Phase 2: ワークスペースIDでフィルタしてから個別にcloseTabを呼ぶ
   */
  const closeWorkspaceTabs = useCallback(async (workspaceId: string) => {
    console.log('[TabContext] closeWorkspaceTabs called:', { workspaceId });
    // Phase 2: 指定ワークスペースのタブを取得して個別に閉じる
    const workspaceTabs = tabs.filter(tab => tab.workspaceId === workspaceId);
    const tabIds = workspaceTabs.map(tab => tab.id);

    console.log('[TabContext] Closing tabs:', {
      workspaceId,
      tabCount: tabIds.length,
      tabIds
    });

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

  // Performance: 入力状態を専用 Context に分離
  // activeTabInputState が変わっても TabContext の consumer は再レンダリングされない
  // Performance: 入力値のみの Context（読み取り専用）
  // App.tsx はこの Context を subscribe しない → キー入力で App.tsx が再レンダリングされない
  const inputValue = useMemo<TabInputContextValue>(
    () => ({ activeTabInput: activeTabInputState }),
    [activeTabInputState]
  );

  const value = useMemo<TabContextValue>(
    () => ({
      tabs,
      activeTabId,

      // タブ操作
      openTab,
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
      closeWorkspaceTabs,
    }),
    [
      tabs,
      activeTabId,
      openTab,
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
      closeWorkspaceTabs,
    ]
  );

  return (
    <TabContext.Provider value={value}>
      <TabInputContext.Provider value={inputValue}>
        {children}
      </TabInputContext.Provider>
    </TabContext.Provider>
  );
}

export function useTabContext(): TabContextValue {
  const context = useContext(TabContext);
  if (!context) {
    throw new Error('useTabContext must be used within a TabProvider');
  }
  return context;
}

/**
 * Performance: 入力状態のみを購読する専用 hook
 * キー入力で変わるのはこの Context のみ → TabContext の consumer は再レンダリングされない
 */
export function useTabInput(): TabInputContextValue {
  const context = useContext(TabInputContext);
  if (!context) {
    throw new Error('useTabInput must be used within a TabProvider');
  }
  return context;
}

