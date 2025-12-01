# ContextMode 設計書

## 概要

ContextModeは、AIエージェントに提供されるシステムContextの量を制御する機能。
デフォルトの「Rich」モードでは全てのContext拡張が有効だが、「Clean」モードではExpertise（Persona背景）のみを保持し、シンプルな対話を実現する。

## 目的

- 余計なContext指示による「癖」を排除
- Expertiseベースのプレーンな対話を可能に
- ユーザーが必要に応じて明示的にファイル添付やWorkspace参照を行える柔軟性を維持

## ContextMode定義

| Mode | 説明 |
|------|------|
| **Rich** | デフォルト。全てのContext拡張が有効 |
| **Clean** | Expertiseのみ。システム拡張は無効 |

## Context拡張の分類

### Rich Mode で有効、Clean Mode で無効

| Context | 説明 | 適用箇所 |
|---------|------|----------|
| SlashCommand Prompt Extension | AIにスラッシュコマンド一覧・使用方法を提供 | `session.rs:build_slash_command_prompt` |
| TalkStyle Instructions | ブレスト/カジュアル/意思決定等の対話スタイル指示 | `InteractionManager` |
| ConversationMode Instructions | Normal/Concise/Brief/Discussion等の指示 | `InteractionManager` |
| ExecutionStrategy Hints | Broadcast/Sequential/Mentioned等のヒント | `InteractionManager` |
| Participant Context | 参加者一覧・役割情報 | `InteractionManager` |

### 両Mode で有効（保持）

| Context | 説明 | 理由 |
|---------|------|------|
| Expertise (Persona Background) | ペルソナの専門性・背景情報 | Cleanでも専門性は維持したい |
| User Attachments | ユーザーが明示的に添付したファイル | ユーザーの意図的な提供 |
| Workspace Files (on demand) | ユーザーがコマンドで参照したファイル | ユーザーの意図的な参照 |
| Conversation History | 直近の会話履歴 | 対話の継続性に必要 |

## データモデル

### ContextMode enum

```rust
// orcs-core/src/session/model.rs

/// Context mode for controlling AI context injection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, SchemaBridge)]
#[serde(rename_all = "snake_case")]
pub enum ContextMode {
    /// Full context: all system extensions enabled
    #[default]
    Rich,
    /// Clean context: expertise only, no system extensions
    Clean,
}
```

### Session Domain Model

```rust
// orcs-core/src/session/model.rs
pub struct Session {
    // ... existing fields ...

    /// Context mode for AI interactions
    #[serde(default)]
    pub context_mode: ContextMode,
}
```

### DTO Layer (Version Bump Required)

**重要**: Session DTOにフィールド追加するため、version-migrateでバージョンを上げる

現在の最新バージョン: **4.2.0** → 新バージョン: **4.3.0**

```rust
// orcs-infrastructure/src/dto/session.rs

/// Session DTO - Version 4.3.0
/// Added: context_mode field for controlling AI context injection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[versioned(version = "4.3.0")]
pub struct SessionV4_3_0 {
    // ... existing fields from V4_2_0 ...

    /// Context mode for AI interactions (v4.3.0+)
    /// Default: Rich (full context)
    #[serde(default)]
    pub context_mode: ContextMode,
}

// Migration from V4_2_0 to V4_3_0
impl From<SessionV4_2_0> for SessionV4_3_0 {
    fn from(old: SessionV4_2_0) -> Self {
        Self {
            // ... copy all existing fields ...
            context_mode: ContextMode::Rich, // default for existing sessions
        }
    }
}
```

### Migration Strategy

- `#[serde(default)]` により、既存のJSONファイルは自動的に `Rich` モードとして読み込まれる
- version-migrate の自動マイグレーション機能で V4_2_0 → V4_3_0 変換

## Frontend 型定義

```typescript
// types/session.ts
export type ContextMode = 'rich' | 'clean';

export interface Session {
  // ... existing fields ...
  contextMode?: ContextMode;
}
```

## API 設計

### Tauri Commands

```rust
/// Sets the context mode for the active session
#[tauri::command]
pub async fn set_context_mode(
    mode: String,  // "rich" | "clean"
    state: State<'_, AppState>,
) -> Result<(), String>

/// Gets the current context mode for the active session
#[tauri::command]
pub async fn get_context_mode(
    state: State<'_, AppState>,
) -> Result<String, String>
```

## InteractionManager 変更

```rust
impl InteractionManager {
    /// Sets the context mode
    pub async fn set_context_mode(&self, mode: ContextMode) {
        *self.context_mode.write().await = mode;
    }

    /// Gets the current context mode
    pub async fn get_context_mode(&self) -> ContextMode {
        self.context_mode.read().await.clone()
    }

    /// Check if prompt extensions should be applied
    fn should_apply_prompt_extension(&self, context_mode: &ContextMode) -> bool {
        matches!(context_mode, ContextMode::Rich)
    }
}
```

## 影響箇所

### 1. session.rs (handle_input)

```rust
// Before calling manager.handle_input_with_streaming
let context_mode = manager.get_context_mode().await;

if matches!(context_mode, ContextMode::Rich) {
    let slash_commands = state.slash_command_repository.list_commands().await...;
    let prompt_extension = build_slash_command_prompt(&slash_commands);
    manager.set_prompt_extension(prompt_extension).await;
} else {
    // Clean mode: no prompt extension
    manager.set_prompt_extension(None).await;
}
```

### 2. InteractionManager (dialogue building)

TalkStyle, ConversationMode の適用時に ContextMode をチェック：

```rust
// In build_dialogue or similar
if matches!(self.context_mode, ContextMode::Rich) {
    // Apply TalkStyle instructions
    // Apply ConversationMode instructions
    // Apply ExecutionStrategy hints
}
// Expertise is always applied
```

## UI 設計

### StatusBar 表示

```
[Rich ▼] | Participants: 3 | Mode: Normal | ...
```

または

```
[📚 Rich ▼] → クリックでドロップダウン
  ├─ 📚 Rich (Full Context)
  └─ 🧹 Clean (Expertise Only)
```

### コンポーネント

```typescript
// components/chat/ContextModeSelector.tsx
interface ContextModeSelectorProps {
  value: ContextMode;
  onChange: (mode: ContextMode) => void;
}
```

## 実装順序

1. **Phase 1: Domain Model**
   - [ ] `ContextMode` enum を `orcs-core/src/session/model.rs` に追加
   - [ ] `Session` に `context_mode` フィールド追加

2. **Phase 2: DTO Layer (Version Bump)**
   - [ ] `SessionV4_3_0` を `orcs-infrastructure/src/dto/session.rs` に追加
   - [ ] `From<SessionV4_2_0> for SessionV4_3_0` マイグレーション実装
   - [ ] 最新バージョンエイリアス更新
   - [ ] Domain ↔ DTO 変換関数更新

3. **Phase 3: InteractionManager**
   - [ ] `InteractionManager` に context_mode 状態追加
   - [ ] `to_session` / `from_session` で context_mode を含める
   - [ ] Tauri commands 追加 (`set_context_mode`, `get_context_mode`)

4. **Phase 4: Context制御ロジック**
   - [ ] `handle_input` で ContextMode に応じた prompt_extension 制御
   - [ ] `InteractionManager` で TalkStyle/ConversationMode 適用制御

5. **Phase 5: Frontend**
   - [ ] `ContextMode` 型定義追加
   - [ ] `ContextModeSelector` コンポーネント作成
   - [ ] `StatusBar` に統合
   - [ ] Backend API 呼び出し

6. **Phase 6: テスト・検証**
   - [ ] 既存セッション読み込みテスト（マイグレーション確認）
   - [ ] Clean Mode での動作確認（Context注入が無効化されること）
   - [ ] Rich Mode との切り替え確認

## 将来の拡張

- **Custom Mode**: 個別にContext項目のON/OFFを選択
- **Preset Modes**: 用途別プリセット（Coding, Discussion, Research等）
- **Per-Message Override**: 特定メッセージだけContextを変更
