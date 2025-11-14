/**
 * AI Integration Core Types
 *
 * このファイルは、AI機能統合の核となる型定義を提供します。
 * Tauri等の特定の実装に依存しない、汎用的なインターフェースです。
 */

/**
 * AIコンテキスト情報
 *
 * AIに渡される入力箇所の文脈情報を表します。
 * これにより、AIは適切なプロンプトを生成できます。
 */
export interface AIContextInfo {
  /** スコープ（例: "UserProfile.Bio", "Message.Reply", "Session.Title"） */
  scope: string;

  /** 入力タイプ */
  type: 'string' | 'long_text' | 'markdown' | 'code';

  /** 最大文字数（オプション） */
  maxLength?: number;

  /** 関連データ（コンテキスト補完用） */
  metadata?: Record<string, any>;
}

/**
 * AI操作の種類
 */
export type AIOperation = 'generate' | 'refine';

/**
 * AI履歴エントリ
 */
export interface AIHistoryEntry {
  /** 生成/修正された値 */
  value: string;

  /** タイムスタンプ */
  timestamp: Date;

  /** 操作種別 */
  operation: AIOperation;

  /** 使用された方向性（オプション） */
  direction?: string;
}

/**
 * AI Provider インターフェース
 *
 * 実装に依存しない、AIサービスへの抽象化されたインターフェースです。
 * Gemini, Claude, OpenAI等、どのプロバイダーでもこのインターフェースを実装することで
 * 差し替え可能になります。
 */
export interface IAIProvider {
  /**
   * 新しいコンテンツを生成します
   *
   * @param context - AIコンテキスト情報
   * @param direction - 生成の方向性（例: "フォーマルに", "簡潔に"）
   * @returns 生成されたテキスト
   */
  generate(context: AIContextInfo, direction?: string): Promise<string>;

  /**
   * 既存のコンテンツを修正します
   *
   * @param currentText - 現在のテキスト
   * @param context - AIコンテキスト情報
   * @param direction - 修正の方向性（例: "フォーマルに", "簡潔に"）
   * @returns 修正されたテキスト
   */
  refine(currentText: string, context: AIContextInfo, direction?: string): Promise<string>;
}

/**
 * useAIRegister の入力オプション
 */
export interface AIRegisterOptions {
  /** AIに渡すコンテキスト */
  context: AIContextInfo;

  /** 現在の値を取得するコールバック */
  getValue: () => string;

  /** 値を設定するコールバック */
  setValue: (newValue: string) => void;

  /** 有効/無効（デフォルト: true） */
  enabled?: boolean;

  /** エラーハンドラ（オプション） */
  onError?: (error: Error) => void;
}

/**
 * useAIRegister の戻り値
 */
export interface AIRegisterResult {
  /** UIコンポーネント用プロパティ */
  triggerProps: {
    /** クリックハンドラ */
    onClick: () => void;

    /** アクティブ状態 */
    isActive: boolean;
  };

  /** メニュー用プロパティ */
  menuProps: {
    /** 開閉状態 */
    isOpen: boolean;

    /** 閉じるハンドラ */
    onClose: () => void;
  };

  /** ヘッドレスなアクション */
  actions: {
    /** 💫 生成 */
    generate: (direction?: string) => Promise<void>;

    /** 🖌️ 修正 */
    refine: (direction?: string) => Promise<void>;

    /** ← 元に戻す */
    undo: () => void;

    /** 🗒️ 履歴表示 */
    showHistory: () => void;

    /** 💬 チャットを開く */
    showChat: () => void;
  };

  /** 現在の状態 */
  state: {
    /** ローディング中 */
    isLoading: boolean;

    /** 履歴 */
    history: AIHistoryEntry[];

    /** Undo可能か */
    canUndo: boolean;

    /** エラー */
    error: Error | null;
  };
}
