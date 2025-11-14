# ORCS AI Integration Module

UIへのAIエージェント機能統合ライブラリ

## 🎯 コンセプト

このモジュールは、ORCSアプリケーション内のあらゆる入力可能な箇所に、AIエージェントのインテリジェンスをシームレスに統合します。

- **1クリックでAI機能**: ✨アイコンから即座に生成・修正
- **コンテキスト認識**: 入力箇所に最適化されたプロンプト
- **ヘッドレスUI設計**: ロジックとUIを完全分離
- **プロバイダー差し替え可能**: Gemini, Claude, OpenAI等に対応

## 📦 アーキテクチャ

```
src/ai/
├── core/                    # ヘッドレスロジック（UI非依存）
│   ├── hooks/
│   │   └── useAIRegister.ts # 核心的なフック
│   ├── context/
│   │   └── AIContext.tsx    # グローバル状態管理
│   └── types/
│       └── ai.ts            # 型定義
│
├── providers/               # AIプロバイダー実装
│   └── GeminiProvider.ts    # Gemini (Tauri実装)
│
└── components/              # UIコンポーネント
    └── AITextField.tsx      # AI統合テキストフィールド
```

### 3層構造

1. **Core Layer** (`ai/core/`)
   - Tauri等の特定実装に依存しない
   - 純粋なReact hooks
   - 他のプロジェクトでも再利用可能

2. **Provider Layer** (`ai/providers/`)
   - `IAIProvider`インターフェースを実装
   - Tauri版: `GeminiProvider`
   - Web版: `WebGeminiProvider`（将来）

3. **UI Layer** (`ai/components/`)
   - Mantine UIコンポーネント
   - デフォルトUIを提供しつつカスタマイズ可能

## 🚀 使い方

### 1. セットアップ

`main.tsx`で`AIProvider`をセットアップ（既に設定済み）:

\`\`\`tsx
import { AIProvider, GeminiProvider } from './ai';

ReactDOM.createRoot(document.getElementById("root")).render(
  <AIProvider provider={new GeminiProvider()}>
    <App />
  </AIProvider>
);
\`\`\`

### 2. AITextFieldを使う（最も簡単）

\`\`\`tsx
import { AITextField } from '@/ai';

function MyComponent() {
  const [bio, setBio] = useState('');

  return (
    <AITextField
      value={bio}
      onChange={setBio}
      context={{
        scope: 'UserProfile.Bio',
        type: 'long_text',
        maxLength: 500,
      }}
      placeholder="自己紹介を入力..."
      label="自己紹介"
    />
  );
}
\`\`\`

### 3. useAIRegisterで独自UIを構築

\`\`\`tsx
import { useAIRegister } from '@/ai';

function CustomInput() {
  const [value, setValue] = useState('');

  const ai = useAIRegister({
    context: { scope: 'CustomInput', type: 'string' },
    getValue: () => value,
    setValue: (newValue) => setValue(newValue),
  });

  return (
    <div>
      <input value={value} onChange={(e) => setValue(e.target.value)} />

      {/* ✨ AIトリガー */}
      <button {...ai.triggerProps}>AI</button>

      {/* AIメニュー */}
      {ai.menuProps.isOpen && (
        <div>
          <button onClick={() => ai.actions.generate()}>💫 生成</button>
          <button onClick={() => ai.actions.refine()}>🖌️ 修正</button>
          <button onClick={ai.actions.undo} disabled={!ai.state.canUndo}>
            ← 元に戻す
          </button>
        </div>
      )}
    </div>
  );
}
\`\`\`

## 🔧 主要API

### `useAIRegister(options)`

**入力**:
- `context`: AIコンテキスト情報
- `getValue`: 現在の値を取得する関数
- `setValue`: 値を設定する関数
- `enabled`: 有効/無効（オプション）

**戻り値**:
- `triggerProps`: トリガーボタン用プロパティ
- `menuProps`: メニュー用プロパティ
- `actions`: AI操作（generate, refine, undo等）
- `state`: 現在の状態（isLoading, history等）

### `AIContextInfo`

\`\`\`typescript
interface AIContextInfo {
  scope: string;                    // "UserProfile.Bio"
  type: 'string' | 'long_text' | 'markdown' | 'code';
  maxLength?: number;
  metadata?: Record<string, any>;
}
\`\`\`

## 🏷️ 方向性指定

AIには「方向性」を指定して生成・修正できます:

- **フォーマルに**: ビジネス文書向け
- **簡潔に**: 短く要約
- **専門的に**: 技術的詳細を含む
- **友好的に**: 親しみやすい表現
- **カジュアルに**: リラックスした雰囲気
- **詳しく**: 詳細な説明

\`\`\`tsx
// 方向性を指定して生成
ai.actions.generate('フォーマルに');
ai.actions.refine('簡潔に');
\`\`\`

## 🔌 バックエンド実装（TODO）

現在、`GeminiProvider`は以下のTauriコマンドを呼び出します:

\`\`\`rust
// crates/orcs-interaction/src/commands/ai.rs (TODO: 実装)

#[tauri::command]
pub async fn ai_generate(
    prompt: String,
    context: AIContext,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Gemini APIを呼び出し
    // ...
}

#[tauri::command]
pub async fn ai_refine(
    prompt: String,
    current_text: String,
    context: AIContext,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Gemini APIを呼び出し
    // ...
}
\`\`\`

## 📝 次のステップ

### Phase 1: MVP完成 ✅
- [x] 型定義
- [x] `useAIRegister`フック
- [x] `AIContext`
- [x] `GeminiProvider`（Tauri実装）
- [x] `AITextField`コンポーネント
- [x] `main.tsx`に統合

### Phase 2: バックエンド実装（次）
- [ ] Rustバックエンド: `ai_generate` コマンド
- [ ] Rustバックエンド: `ai_refine` コマンド
- [ ] Gemini API連携

### Phase 3: UI拡張
- [ ] 履歴モーダル（🗒️）
- [ ] チャットパネル（💬）
- [ ] エラートースト表示
- [ ] ローディング改善

### Phase 4: 統合
- [ ] MessageItemへの統合
- [ ] SessionRenameへの統合
- [ ] SlashCommandEditorへの統合

### Phase 5: ライブラリ化（将来）
- [ ] `packages/ai-core` に切り出し
- [ ] Web版Provider実装
- [ ] NPMパッケージ化

## 🎨 設計の優れた点

1. **Tauri非依存**: `IAIProvider`インターフェースで抽象化
2. **ヘッドレスUI**: ロジックとUIを完全分離
3. **型安全**: TypeScriptで完全に型付け
4. **既存パターンと整合**: SessionContext等と同じ設計
5. **拡張性**: 新しいプロバイダーを簡単に追加可能

## 🧪 テスト方法（現時点）

1. `npm run dev`でアプリを起動
2. コンソールで `window.__TEST_AI__`を確認（開発モード）
3. AITextFieldコンポーネントを任意の場所に配置してテスト

## 📚 参考資料

- [UI設計ドキュメント](../workspace/ui.md)
- [Tanstack Query](https://tanstack.com/query)（ヘッドレスUIの参考）
- [Mantine UI](https://mantine.dev/)（UIフレームワーク）
