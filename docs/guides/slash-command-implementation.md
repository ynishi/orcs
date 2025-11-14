# SlashCommand実装ガイド

## 概要

ORCSにおけるSlashCommand（`/command`形式）の実装方法を解説。Built-inコマンドとCustomコマンドの2種類があり、HumanとAgentの両方から呼び出される可能性を考慮する必要があります。

## 🎯 コマンドの種類

### 1. Built-in Commands
- フロントエンド（TypeScript）で直接ハンドリング
- 例: `/help`, `/expert`, `/create-persona`, `/workspace`
- 複雑なUI連携や状態管理が必要な場合に使用

### 2. Custom Commands
- `.orcs/commands/`にMarkdownで定義
- タイプ: `prompt`, `shell`, `task`
- バックエンドで展開・実行される
- 注: `entity`タイプは削除されました（Built-inコマンドで実装する必要があります）

## 📋 Built-in Command実装フロー

### ステップ1: コマンド定義の追加

**場所**: `orcs-desktop/src/types/command.ts`

```typescript
{
  name: 'create-persona',
  icon: '👤',
  description: 'Create a new persona from JSON definition (UUID auto-generated)',
  usage: '/create-persona <json>',
  examples: [
    '/create-persona {"name": "Rust Expert", "role": "Senior Rust Developer", ...}',
  ],
  argsDescription: 'JSON with required fields: name, role, background, communication_style, backend',
}
```

**ポイント**:
- `name`: コマンド名（スラッシュなし）
- `icon`: UI表示用アイコン
- `description`: コマンドの説明
- `usage`: 使用方法の例
- `examples`: 具体的な使用例
- `argsDescription`: 引数の詳細説明

### ステップ2: Rustバックエンド実装（必要な場合）

**場所**: `orcs-desktop/src-tauri/src/commands/[module].rs`

```rust
/// Creates a new persona from a CreatePersonaRequest
#[tauri::command] // cSpell:ignore tauri
pub async fn create_persona(
    request: orcs_core::persona::CreatePersonaRequest,
    state: State<'_, AppState>,
) -> Result<Persona, String> {
    // Validate request
    request.validate()?;

    // Convert to Persona (UUID auto-generated)
    let persona = request.into_persona();

    // Save to repository
    let mut all_personas = state
        .persona_repository
        .get_all()
        .await
        .map_err(|e| format!("Failed to load personas: {}", e))?;

    all_personas.push(persona.clone());

    state
        .persona_repository
        .save_all(&all_personas)
        .await
        .map_err(|e| format!("Failed to save persona: {}", e))?;

    Ok(persona)
}
```

**ポイント**:
- `#[tauri::command]`マクロ必須 (cSpell:ignore tauri)
- 引数は自動でデシリアライズされる
- `State<'_, AppState>`でアプリケーション状態にアクセス
- エラーは`Result<T, String>`で返す
- 非同期処理は`async`キーワードで

### ステップ3: コマンドハンドラーの登録

**場所**: `orcs-desktop/src-tauri/src/commands/mod.rs`

```rust
// cSpell:ignore tauri
pub fn handlers() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // ... other commands
        personas::create_persona,
        // ... more commands
    ]
}
```

### ステップ4: フロントエンドハンドラー実装

**場所**: `orcs-desktop/src/hooks/useSlashCommands.ts`

```typescript
case 'create-persona':
  if (parsed.args && parsed.args.length > 0) {
    const jsonString = parsed.args.join(' ');
    try {
      // Parse and validate JSON
      const personaRequest = JSON.parse(jsonString);

      // Show loading notification
      notifications.show({
        title: 'Creating Persona',
        message: `Creating persona: ${personaRequest.name || 'Unknown'}...`,
        color: 'blue',
        autoClose: false,
        id: 'persona-creation',
      });

      // Invoke Rust backend
      const persona = await invoke<import('../types/agent').PersonaConfig>('create_persona', {
        request: personaRequest,
      });

      notifications.hide('persona-creation');

      // Display success message in conversation
      const successMessage = `Persona created: ${persona.name} ${persona.icon || '👤'}\nRole: ${persona.role}`;

      await handleAndPersistSystemMessage(
        conversationMessage(successMessage, 'info', '✅'),
        addMessage,
        invoke
      );

      // Refresh UI state
      await refreshPersonas();
      await refreshSessions();
    } catch (error) {
      console.error('Failed to create persona:', error);
      notifications.hide('persona-creation');

      const errorMessage = error instanceof SyntaxError
        ? `Invalid JSON format: ${error.message}`
        : `Failed to create persona: ${error}`;

      await handleAndPersistSystemMessage(
        conversationMessage(errorMessage, 'error', '❌'),
        addMessage,
        invoke
      );
    }
  } else {
    // Show usage help
    await handleAndPersistSystemMessage(
      conversationMessage(
        'Usage: /create-persona <json>\nExample: /create-persona {"name": "Expert", ...}',
        'error'
      ),
      addMessage,
      invoke
    );
  }
  await saveCurrentSession();
  break;
```

**実装の必須要素**:

#### 1. コマンドログの永続化
```typescript
// NOTE: This is critical for UI parity across session reloads
await handleAndPersistSystemMessage(
  commandMessage(commandLabel),
  addMessage,
  invoke
);
```

#### 2. ローディング状態の表示
```typescript
notifications.show({
  title: 'Creating Persona',
  message: 'Creating...',
  color: 'blue',
  autoClose: false,
  id: 'unique-id',
});

// ... process ...

notifications.hide('unique-id');
```

#### 3. 結果の会話への表示
```typescript
await handleAndPersistSystemMessage(
  conversationMessage(resultMessage, 'info', '✅'),
  addMessage,
  invoke
);
```

#### 4. 状態の更新
```typescript
// 関連するデータを再読み込み
await refreshPersonas();
await refreshSessions();
```

#### 5. エラーハンドリング
```typescript
try {
  // ... main logic ...
} catch (error) {
  console.error('Failed to execute:', error);
  notifications.hide('loading-id');

  await handleAndPersistSystemMessage(
    conversationMessage(`Failed: ${error}`, 'error', '❌'),
    addMessage,
    invoke
  );
}
```

#### 6. セッション保存
```typescript
await saveCurrentSession();
```

## 🤖 Human/Agent呼び出しの考慮事項

### Agentからのコマンド発行

Agentがコマンドを発行する場合、以下の形式を使用：

```xml
<Slash>
  <Name>create-persona</Name>
  <Args>{"name": "CodeReviewer", "role": "Senior Code Review Specialist", ...}</Args>
</Slash>
```

**検出メカニズム** (`App.tsx:300-326`):

```typescript
// Agent responses can themselves issue SlashCommands
if (
  !isSystemMessage &&
  turn.session_id === currentSessionIdRef.current &&
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
          console.error('[STREAM] Failed to execute agent slash command:', error);
        }
      }
    })();
  }
}
```

**実行時の違い**:

```typescript
const { source = 'user', actorName, autoSubmit = false } = options;

const provenanceActor = source === 'agent' ? `${actorName ?? 'Agent'}'s ` : '';
const commandLabel = source === 'agent'
  ? `${actorName ?? 'Agent'} issued ${rawInput}`
  : rawInput;
```

- **source**: `'user'` or `'agent'` - コマンドの発行元
- **actorName**: Agent名（Agent発行の場合のみ）
- **autoSubmit**: `true`の場合、結果を自動的に会話に投稿

### Agent用のコマンド設計ガイドライン

1. **JSONフォーマット**: 引数がJSON形式の場合、パースエラーを適切にハンドリング
2. **通知**: Agentからの実行でも通知を表示（ユーザーが状況を理解できる）
3. **会話への表示**: 実行結果を必ず会話に表示
4. **冪等性**: 同じコマンドを複数回実行しても安全な設計
5. **エラーメッセージ**: 明確で実行可能な修正提案を含める

## 🔧 Custom Command作成フロー

### 1. Promptタイプ

**場所**: `.orcs/commands/[command-name].md`

```markdown
---
type: prompt
description: "Code review with specific focus areas"
---

Please review the following code with focus on:
- Security vulnerabilities
- Performance bottlenecks
- Best practices

{{args}}
```

**用途**: AIにプロンプトとして送信するテンプレート

### 2. Shellタイプ

**場所**: `.orcs/commands/[command-name].md`

```markdown
---
type: shell
description: "Run tests with coverage"
working_dir: "{{workspace_root}}"
---

npm run test:coverage
```

**用途**: シェルコマンドの実行と出力表示

### 3. Taskタイプ

**場所**: `.orcs/commands/[command-name].md`

```markdown
---
type: task
description: "Execute background task"
---

Task description here with {{args}}
```

**用途**: バックグラウンドでの長時間実行タスク

### 変数展開

- `{{args}}`: コマンドライン引数
- `{{workspace_root}}`: 現在のワークスペースルート
- その他カスタム変数

## 🚨 現在のアーキテクチャ課題と今後のリファクタリング

### 問題点

1. **処理の分散**
   - コマンド定義: `types/command.ts`
   - バックエンド実装: `src-tauri/src/commands/`
   - ハンドラー登録: `commands/mod.rs`
   - フロントエンド処理: `hooks/useSlashCommands.ts`
   - 検出ロジック: `App.tsx`
   - パースロジック: `utils/commandParser.ts`

2. **重複したロジック**
   - Built-inとCustomで似た処理が重複
   - エラーハンドリングパターンが統一されていない
   - 通知表示のボイラープレート

3. **拡張性の問題**
   - 新しいコマンド追加時に複数箇所を修正
   - Agent/Human両対応のテストが困難
   - コマンド間の依存関係が不明確

### リファクタリング提案

#### Phase 1: コマンドレジストリの統一

```typescript
// commands/registry.ts
interface CommandHandler {
  name: string;
  description: string;
  validate: (args: string[]) => ValidationResult;
  execute: (args: string[], options: ExecuteOptions) => Promise<CommandResult>;
  onSuccess?: (result: any) => Promise<void>;
  onError?: (error: Error) => Promise<void>;
}

const commandRegistry = new Map<string, CommandHandler>();
```

#### Phase 2: 宣言的なコマンド定義

```typescript
// commands/definitions/create-persona.ts
export const createPersonaCommand: CommandHandler = {
  name: 'create-persona',
  description: 'Create a new persona from JSON definition',

  validate: (args) => {
    if (args.length === 0) return { valid: false, error: 'JSON required' };
    try {
      const data = JSON.parse(args.join(' '));
      return validatePersonaSchema(data);
    } catch (e) {
      return { valid: false, error: 'Invalid JSON' };
    }
  },

  execute: async (args, { invoke, notifications }) => {
    const request = JSON.parse(args.join(' '));

    notifications.show({ id: 'create-persona', message: 'Creating...' });

    try {
      const persona = await invoke('create_persona', { request });
      return { success: true, data: persona };
    } finally {
      notifications.hide('create-persona');
    }
  },

  onSuccess: async (persona) => {
    await refreshPersonas();
    await refreshSessions();
  },
};
```

#### Phase 3: 統一されたコマンド実行パイプライン

```typescript
// commands/executor.ts
export async function executeCommand(
  commandName: string,
  args: string[],
  options: ExecuteOptions
): Promise<void> {
  const handler = commandRegistry.get(commandName);
  if (!handler) throw new Error(`Unknown command: ${commandName}`);

  // 1. Log command
  await logCommand(commandName, args, options);

  // 2. Validate
  const validation = handler.validate(args);
  if (!validation.valid) {
    await showError(validation.error);
    return;
  }

  // 3. Execute
  try {
    const result = await handler.execute(args, options);

    // 4. Handle success
    await showSuccess(result);
    if (handler.onSuccess) {
      await handler.onSuccess(result.data);
    }
  } catch (error) {
    // 5. Handle error
    await showError(error);
    if (handler.onError) {
      await handler.onError(error);
    }
  }

  // 6. Save session
  await saveCurrentSession();
}
```

#### Phase 4: Agent/Human統一インターフェース

```typescript
// commands/invocation.ts
export interface CommandInvocation {
  command: string;
  args: string[];
  source: 'human' | 'agent';
  actorName?: string;
  autoSubmit?: boolean;
}

export async function handleInvocation(invocation: CommandInvocation): Promise<void> {
  const { command, args, source, actorName, autoSubmit } = invocation;

  // Unified execution with source-aware behavior
  const options: ExecuteOptions = {
    source,
    actorName,
    autoSubmit,
    // ... other common options
  };

  await executeCommand(command, args, options);
}
```

### リファクタリングの優先度

1. **High**: コマンドレジストリの統一（新規コマンド追加の簡素化）
2. **Medium**: エラーハンドリング・通知の標準化
3. **Low**: Custom commandとBuilt-in commandの統合

## 📚 参考実装

### 成功例: `/expert`コマンド
- シンプルな引数処理
- 適切な通知表示
- 結果の会話表示
- 状態更新

### 参考: `/create-persona`コマンド
- JSON引数のパース
- バックエンド連携
- エラーハンドリング
- Agent/Human両対応

## ✅ 実装チェックリスト

Built-in Commandを追加する際のチェックリスト：

- [ ] `types/command.ts`にコマンド定義を追加
- [ ] Rustバックエンドに`#[tauri::command]`実装（必要な場合）
- [ ] `commands/mod.rs`にハンドラーを登録（必要な場合）
- [ ] `useSlashCommands.ts`にcase文を追加
- [ ] コマンドログを永続化（`handleAndPersistSystemMessage`）
- [ ] ローディング通知を実装
- [ ] 結果を会話に表示
- [ ] 関連状態を更新（refresh関数）
- [ ] エラーハンドリングを実装
- [ ] セッション保存を実行
- [ ] Agent発行時の動作を確認
- [ ] 引数なしの場合のヘルプ表示
- [ ] TypeScriptコンパイルエラーがないか確認
- [ ] 実際に動作テスト（Human/Agent両方）

## 🔍 デバッグのポイント

### コマンドが検出されない場合

1. **Agentのレスポンス確認**: `App.tsx:306`のconsole.logで検出状況を確認
2. **XML形式の確認**: `<Slash><Name>...</Name></Slash>`形式になっているか
3. **セッションID一致**: `turn.session_id === currentSessionIdRef.current`

### コマンドが実行されない場合

1. **switch文の確認**: case文が追加されているか
2. **コマンド名の一致**: 定義とcase文のnameが一致しているか
3. **Built-in判定**: `isValidCommand`がtrueを返すか確認

### バックエンドエラー

1. **Rustコマンド登録**: `mod.rs`のhandlers()に含まれているか
2. **引数の型**: TypeScriptからRustへの型変換が正しいか
3. **エラーログ**: Tauriのコンソール出力を確認 (cSpell:ignore Tauri)

---

**最終更新**: 2025-11-15
**関連ドキュメント**:
- `tauri-command-workflow.md` - Tauriコマンド実装の基本 (cSpell:ignore tauri)
- `ARCHITECTURE.md` - システム全体のアーキテクチャ
