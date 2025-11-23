import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react';
import type { Session } from '../types/session';
import type { Message } from '../types/message';

/**
 * タブの情報（Aggregated State）
 * - 各タブが独立した状態を持つ
 * - SessionContextと同じく、薄いState層としてModel（Backend）とView（UI）の境界を明確化
 */
export interface SessionTab {
  // セッション情報
  id: string; // タブのユニークID (タブ作成時に生成)
  sessionId: string; // 対応するセッションID
  workspaceId: string; // このタブが属するWorkspace ID
  title: string; // タブタイトル

  // メッセージ関連
  messages: Message[]; // タブ固有のメッセージ履歴

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
  lastAccessedAt: number; // 最終アクセス時刻 (タブ切り替え時に更新)
}

export interface TabContextValue {
  tabs: SessionTab[];
  activeTabId: string | null;
  
  // タブ操作
  openTab: (session: Session, messages: Message[], workspaceId: string, switchToTab?: boolean) => string; // 新規タブを開く。既に開いている場合はフォーカス
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
  const [tabs, setTabs] = useState<SessionTab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);

  /**
   * 新規タブを開く（既に開いている場合はフォーカス）
   */
  const openTab = useCallback((session: Session, messages: Message[], workspaceId: string, switchToTab: boolean = true): string => {
    let tabId: string = '';
    let isExistingTab = false;

    setTabs((prev) => {
      // 既に同じセッションのタブが開いているか確認
      const existingTab = prev.find((tab) => tab.sessionId === session.id);

      if (existingTab) {
        // 既存タブを更新
        tabId = existingTab.id;
        isExistingTab = true;
        return prev.map((tab) =>
          tab.id === existingTab.id
            ? { ...tab, messages, title: session.title, lastAccessedAt: Date.now() }
            : tab
        );
      }

      // 新規タブを作成（デフォルト値で初期化）
      tabId = `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
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
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, isDirty } : tab))
    );
  }, []);

  /**
   * タブの入力テキストを更新
   */
  const updateTabInput = useCallback((tabId: string, input: string) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, input } : tab))
    );
  }, []);

  /**
   * タブの添付ファイルを更新
   */
  const updateTabAttachedFiles = useCallback((tabId: string, files: File[]) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, attachedFiles: files } : tab))
    );
  }, []);

  /**
   * タブに添付ファイルを追加
   */
  const addAttachedFileToTab = useCallback((tabId: string, file: File) => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, attachedFiles: [...tab.attachedFiles, file] } : tab
      )
    );
  }, []);

  /**
   * タブから添付ファイルを削除
   */
  const removeAttachedFileFromTab = useCallback((tabId: string, index: number) => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId
          ? { ...tab, attachedFiles: tab.attachedFiles.filter((_, i) => i !== index) }
          : tab
      )
    );
  }, []);

  /**
   * タブのドラッグ状態を更新
   */
  const setTabDragging = useCallback((tabId: string, isDragging: boolean) => {
    setTabs((prev) =>
      prev.map((tab) => (tab.id === tabId ? { ...tab, isDragging } : tab))
    );
  }, []);

  /**
   * タブのAI思考状態を更新
   */
  const setTabThinking = useCallback((tabId: string, isThinking: boolean, personaName: string = 'AI') => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, isAiThinking: isThinking, thinkingPersona: personaName } : tab
      )
    );
  }, []);

  /**
   * タブのAutoMode状態を更新
   */
  const setTabAutoMode = useCallback((tabId: string, autoMode: boolean) => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, autoMode } : tab
      )
    );
  }, []);

  /**
   * タブのAutoChat iteration状態を更新
   */
  const setTabAutoChatIteration = useCallback((tabId: string, iteration: number | null) => {
    setTabs((prev) =>
      prev.map((tab) =>
        tab.id === tabId ? { ...tab, autoChatIteration: iteration } : tab
      )
    );
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

