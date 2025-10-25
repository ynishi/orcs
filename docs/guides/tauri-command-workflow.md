# Tauriコマンド追加ワークフロー

## 概要

新しいTauriコマンドを追加する際の手順を、今回の`rename_session`を例に解説。

## 🎯 基本フロー

### 1. ビジネスロジックの実装（SessionManager）

**場所**: `crates/orcs-core/src/session_manager.rs`

```rust
/// Renames a session by updating its title.
pub async fn rename_session(&self, session_id: &str, new_title: String) -> Result<()> {
    let mut session = self.repository.find_by_id(session_id).await?
        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

    session.title = new_title;
    session.updated_at = chrono::Utc::now().to_rfc3339();

    self.repository.save(&session).await?;
    Ok(())
}
```

**ポイント**:
- Repository経由でデータアクセス
- エラーハンドリング（Result型）
- タイムスタンプ更新

### 2. Tauriコマンド関数の追加

**場所**: `orcs-desktop/src-tauri/src/main.rs`

```rust
/// Renames a session
#[tauri::command]
async fn rename_session(
    session_id: String,
    new_title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.session_manager
        .rename_session(&session_id, new_title)
        .await
        .map_err(|e| e.to_string())
}
```

**ポイント**:
- `#[tauri::command]` マクロ必須
- 引数は自動でJSONからデシリアライズ
- エラーは`String`に変換（Tauri IPC制約）
- `State<'_, AppState>`でアプリケーション状態にアクセス

### 3. invoke_handlerに登録

**場所**: `orcs-desktop/src-tauri/src/main.rs` のmain関数内

```rust
.invoke_handler(tauri::generate_handler![
    create_session,
    list_sessions,
    switch_session,
    delete_session,
    rename_session,  // ← 追加
    save_current_session,
    // ... その他
])
```

**注意**: この登録を忘れると実行時エラー

### 4. TypeScript Hooksの実装

**場所**: `src/hooks/useSessions.ts`

```typescript
const renameSession = async (sessionId: string, newTitle: string): Promise<void> => {
  try {
    await invoke('rename_session', { sessionId, newTitle });
    // ローカルStateを更新
    setSessions(prev =>
      prev.map(s => s.id === sessionId ? { ...s, title: newTitle } : s)
    );
  } catch (err) {
    console.error('Failed to rename session:', err);
    throw new Error(`Failed to rename session: ${err}`);
  }
};
```

**ポイント**:
- `invoke('コマンド名', { 引数オブジェクト })`
- バックエンド更新後、ローカルStateも更新
- エラーハンドリング

### 5. UIコンポーネントから呼び出し

**場所**: `src/components/sessions/SessionList.tsx`

```typescript
const handleSaveEdit = (sessionId: string) => {
  if (editingTitle.trim()) {
    onSessionRename?.(sessionId, editingTitle.trim());
  }
  setEditingSessionId(null);
};
```

## 📋 チェックリスト

新しいTauriコマンド追加時：

### バックエンド（Rust）
- [ ] SessionManagerにメソッド実装
  - [ ] Repository経由でデータアクセス
  - [ ] エラーハンドリング（Result型）
  - [ ] 必要に応じてタイムスタンプ更新
- [ ] Tauriコマンド関数を追加
  - [ ] `#[tauri::command]` マクロ
  - [ ] 適切な引数型（String, bool, etc）
  - [ ] `State<'_, AppState>` で状態アクセス
  - [ ] エラーを`String`に変換
- [ ] `invoke_handler![]` に登録
- [ ] `cargo check` で確認

### フロントエンド（TypeScript）
- [ ] Hooksに関数追加
  - [ ] `invoke()` でコマンド呼び出し
  - [ ] ローカルState更新
  - [ ] エラーハンドリング
- [ ] UIコンポーネントから呼び出し
- [ ] `npx tsc --noEmit` で型チェック

### 動作確認
- [ ] Dev環境で実際に動作確認
- [ ] エラーケースも確認（存在しないIDなど）

## 🔍 よくある間違い

### 1. invoke_handlerへの登録忘れ
**症状**: `command rename_session not found` エラー

**修正**:
```rust
.invoke_handler(tauri::generate_handler![
    rename_session,  // ← 追加
    // ...
])
```

### 2. 引数名の不一致
**症状**: 引数が`undefined`

**TypeScript**:
```typescript
invoke('rename_session', { sessionId, newTitle })  // camelCase
```

**Rust**:
```rust
async fn rename_session(
    session_id: String,    // snake_case
    new_title: String,     // snake_case
    // ...
)
```

Tauriが自動的にcamelCase ↔ snake_case変換してくれるので問題なし。

### 3. ローカルState更新忘れ
**症状**: UIが更新されない（リロードすると表示される）

**修正**:
```typescript
await invoke('rename_session', { sessionId, newTitle });
// ← ここでローカルStateを更新
setSessions(prev =>
  prev.map(s => s.id === sessionId ? { ...s, title: newTitle } : s)
);
```

### 4. エラーハンドリング不足
**症状**: エラー時にUIがフリーズ

**修正**:
```typescript
try {
  await invoke('rename_session', { sessionId, newTitle });
} catch (err) {
  console.error('Failed:', err);
  throw err;  // または適切なエラー表示
}
```

## 🚀 パフォーマンス最適化

### 楽観的更新（Optimistic Update）

先にUIを更新し、失敗したら戻す：

```typescript
const renameSession = async (sessionId: string, newTitle: string) => {
  // 先にUI更新
  const prevSessions = sessions;
  setSessions(prev =>
    prev.map(s => s.id === sessionId ? { ...s, title: newTitle } : s)
  );

  try {
    await invoke('rename_session', { sessionId, newTitle });
  } catch (err) {
    // 失敗したら元に戻す
    setSessions(prevSessions);
    throw err;
  }
};
```

## 📚 参考

- [Tauri公式: Commands](https://tauri.app/v1/guides/features/command)
- [Schema変更チェックリスト](./schema-change-checklist.md)

## 🧪 テスト

### Rustユニットテスト
```rust
#[tokio::test]
async fn test_rename_session() {
    let repository = Arc::new(MockSessionRepository::new());
    let manager = SessionManager::new(repository);

    // テストコード
}
```

### E2Eテスト（将来的に）
```typescript
// Playwright等で
await page.click('[data-testid="rename-session"]');
await page.fill('input', 'New Title');
await page.press('input', 'Enter');
expect(await page.textContent('.session-title')).toBe('New Title');
```
