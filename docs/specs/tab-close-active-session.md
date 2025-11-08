# Tab Close & Active Session Management 仕様

## 概要

タブのクローズ操作とActiveSession（現在選択中のSession）の自動切り替え機能の仕様。

ActiveSessionのタブを閉じた際、自動的に次のSessionに切り替わり、左ペインのSessionList選択状態と右ペインのタブUIが同期する。

---

## 背景と課題

### 発見された問題

1. **Tab Close時にタブが再オープンされる**
   - CloseButtonをクリックしてもタブが消えない
   - `loadActiveSessionMessages` useEffectの依存配列に`tabs`が含まれていたため、タブ削除時に再実行され、タブが再度開かれていた

2. **SessionList選択時にタブが選択状態にならない**
   - 左ペインでSessionを選択してタブは開くが、タブに下線が表示されない（UnSelect状態）
   - `openTab()`内で`setActiveTabId()`が`setTabs()`のコールバック内で呼ばれており、React更新タイミングの問題で確実に反映されなかった

3. **ActiveSessionのタブをCloseしても選択状態が外れるだけ**
   - タブは閉じるが、左ペインのSessionList選択状態（青背景）が残る
   - 次のSessionに自動切り替えされない

4. **Tab Close時に閉じたタブが残る**
   - `switchSession()` → `loadActiveSessionMessages` useEffect → `openTab()` の競合により、タブが重複作成される

5. **Tab Close時に間違ったタブにフォーカス**
   - `closeTab()` が削除前の配列で次のタブを選択するため、閉じるSessionのタブにフォーカスしてしまう
   - その結果、閉じたSessionのタブが再オープンされる

---

## 設計判断

### 判断1: ActiveSession自動切り替えの実装場所

**選択肢:**
- **Option A**: TabContext内に実装
- **Option B**: App.tsx CloseButtonハンドラーに実装 ✅ 採用

**判断理由:**
- **責務の分離:** TabContextはタブUIの管理のみ、Sessionとの連携はApp.tsx
- **既存パターンの踏襲:** SessionContext.deleteSession()の既存ロジックと類似
- **柔軟性:** App.tsx側で細かい制御が可能

### 判断2: 次のActiveSession選択アルゴリズム

**選択肢:**
- **Option A**: インデックスベース（削除したSessionの位置に近いものを選択）
- **Option B**: 更新日時ベース（最も直近に更新されたSession） ✅ 採用

**判断理由:**
- **ユーザー期待値:** 最近使ったSessionに戻りたい
- **SessionListソート順との一貫性:** SessionListは更新日時でソートされている（SessionList.tsx L44-46）
- **直感的:** 時系列順の方が予測しやすい

### 判断3: Edge Case - Workspace内最後のSessionのTab Close

**選択肢:**
- **Option A**: ActiveSession = null（タブなし状態を許容）
- **Option B**: 新規Session自動作成 ✅ 採用

**判断理由:**
- **Default Workspace仕様との整合性:** すべてのSessionは必ずWorkspaceに紐づく（`workspace_id: String`）
- **既存動作の踏襲:** SessionContext.deleteSession()でも新規作成している（L100-101）
- **UX:** 空の状態を作らず、常に何かしらのSessionが選択されている方が使いやすい

### 判断4: Tab Close処理の順序

**選択肢:**
- **Option A**: `closeTab()` → `switchSession()` → `openTab()`
- **Option B**: `switchSession()` → `openTab()` → `closeTab()` ✅ 採用

**判断理由:**
- **競合回避:** 次のSessionに切り替えてタブを開いた後に古いタブを閉じることで、`closeTab()`が間違ったタブにフォーカスすることを防ぐ
- **確実性:** 正しいタブがフォーカスされた状態を保証してから古いタブを削除
- **自動フォーカス無効化:** 閉じるタブは既にActiveではないため、`closeTab()`内の自動フォーカスロジックが実行されない

---

## 実装仕様

### 1. Tab Close時の処理フロー

#### ActiveSessionのTab Close

```
[1] CloseButton onClick
[2] 閉じるタブがActiveSessionか判定
    - closingTab.sessionId === currentSessionId
[3] Workspace内の残りSession取得
    - sessions.filter(s => s.workspace_id === workspace.id && s.id !== closingTab.sessionId)
[4] 更新日時が直近のSessionを選択
    - sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
[5] Backend Session切り替え
    - await switchSession(nextSession.id)
[6] 次のSessionのTabを開く
    - const newTabId = openTab(nextSession, messages, workspace.id, true)
    - openTab()は既存タブがあれば更新してフォーカス、なければ新規作成
[7] 古いタブを閉じる
    - closeTab(closingTab.id)
```

#### 非ActiveSessionのTab Close

```
[1] CloseButton onClick
[2] 閉じるタブがActiveSessionでないことを確認
    - closingTab.sessionId !== currentSessionId
[3] 単純にタブを閉じる
    - closeTab(tab.id)
[4] closeTab()内で次のタブにフォーカス（TabContext L164-170）
```

### 2. 関連する修正

#### 修正1: loadActiveSessionMessages useEffectの依存配列最適化

**ファイル:** `orcs-desktop/src/App.tsx` L318-361

**変更:**
- 依存配列から `tabs` を削除
- `tabs.find()` → `getTabBySessionId()` に変更

**Before:**
```typescript
useEffect(() => {
  const existingTab = tabs.find(tab => tab.sessionId === currentSessionId);
  if (!existingTab && workspace) {
    openTab(activeSession, restoredMessages, workspace.id, true);
  }
}, [currentSessionId, sessions, sessionsLoading, userNickname, personas, tabs, openTab]);
```

**After:**
```typescript
useEffect(() => {
  const existingTab = getTabBySessionId(currentSessionId);
  if (!existingTab && workspace) {
    openTab(activeSession, restoredMessages, workspace.id, true);
  }
}, [currentSessionId, sessions, sessionsLoading, userNickname, personas, workspace, openTab, getTabBySessionId]);
```

#### 修正2: openTab()のState更新順序

**ファイル:** `orcs-desktop/src/context/TabContext.tsx` L85-137

**変更:**
- `setActiveTabId()` を `setTabs()` の外に移動

**Before:**
```typescript
setTabs((prev) => {
  if (existingTab) {
    tabId = existingTab.id;
    if (switchToTab) {
      setActiveTabId(tabId);  // ← setTabs()内
    }
    return prev.map(...);
  }
  // ...
});
```

**After:**
```typescript
setTabs((prev) => {
  if (existingTab) {
    tabId = existingTab.id;
    return prev.map(...);
  }
  // ...
});

// setTabs()の外で setActiveTabId() を呼ぶ
if (switchToTab) {
  setActiveTabId(tabId!);
}
```

#### 修正3: CloseButton処理順序の変更

**ファイル:** `orcs-desktop/src/App.tsx` L1418-1497

**変更:**
- `closeTab()` を最後に呼ぶように変更
- ActiveSessionと非ActiveSessionで分岐

**実装コード:**
```typescript
<CloseButton
  onClick={async (e) => {
    e.stopPropagation();

    // 未保存確認
    if (tab.isDirty) {
      if (!window.confirm(`"${tab.title}" has unsaved changes. Close anyway?`)) {
        return;
      }
    }

    // 閉じるタブの情報を取得
    const closingTab = tabs.find(t => t.id === tab.id);
    if (!closingTab) return;

    // ActiveSessionのタブを閉じる場合
    const isClosingActiveSession = closingTab.sessionId === currentSessionId;

    if (isClosingActiveSession && workspace) {
      // 1. Workspace内の残りSession取得
      const remainingSessions = sessions.filter(
        s => s.workspace_id === workspace.id && s.id !== closingTab.sessionId
      );

      if (remainingSessions.length > 0) {
        // 2. 更新日時が直近のSessionを選択
        const sortedSessions = [...remainingSessions].sort(
          (a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
        );
        const nextSession = sortedSessions[0];

        try {
          // 3. Backend Session切り替え
          await switchSession(nextSession.id);

          // 4. 次のSessionのTabを開く
          const messages = convertSessionToMessages(nextSession, userNickname);
          const newTabId = openTab(nextSession, messages, workspace.id, true);

          // 5. 古いタブを閉じる
          closeTab(tab.id);
        } catch (err) {
          console.error('[App] Failed to switch to next session:', err);
        }
      } else {
        // Workspace内にSessionがない場合、新規作成
        await createSession();
      }
    } else {
      // 非ActiveSessionのTab Closeの場合、単純に閉じる
      closeTab(tab.id);
    }
  }}
/>
```

---

## Edge Case仕様

### Case 1: 非ActiveSessionのTab Close

**状態:**
- tabs: `[Session A (Active), Session B, Session C]`
- currentSessionId: `Session A`

**操作:** Session BのタブのCloseButtonをクリック

**期待動作:**
- Session Bのタブが閉じる
- Session AがActiveのまま
- 左ペインのSessionList選択状態は変わらない

**実装:** `closeTab(tab.id)` のみ実行

### Case 2: ActiveSessionのTab Close（残りSessionあり）

**状態:**
- tabs: `[Session A (Active), Session B, Session C]`
- currentSessionId: `Session A`
- Session B.updated_at: 2025-01-08 12:00
- Session C.updated_at: 2025-01-08 11:00

**操作:** Session AのタブのCloseButtonをクリック

**期待動作:**
1. Session Aのタブが閉じる
2. Session B（更新日時が最新）に自動切り替え
3. Session Bのタブが開く（既に開いていればフォーカス）
4. 左ペインのSessionList選択状態がSession Bに変わる（青背景）

**実装:**
```
remainingSessions = [Session B, Session C]
sortedSessions = [Session B, Session C]  // updated_atでソート
nextSession = Session B
switchSession(Session B)
openTab(Session B, ...)
closeTab(Session A)
```

### Case 3: Workspace内最後のSessionのTab Close

**状態:**
- tabs: `[Session A (Active)]`
- currentSessionId: `Session A`
- Workspace内のSession数: 1

**操作:** Session AのタブのCloseButtonをクリック

**期待動作:**
1. Session Aのタブが閉じる
2. 新規Session自動作成
3. 新しいSessionがActiveSessionになる
4. 新しいSessionのタブが開く

**実装:**
```
remainingSessions = []
createSession()
// createSession()内で新しいSessionがActiveSessionになる
// loadActiveSessionMessages useEffectが新しいSessionのタブを開く
```

### Case 4: 複数Workspaceにまたがるタブ（異なるWorkspace間）

**状態:**
- tabs: `[Session A (Workspace 1, Active), Session B (Workspace 2)]`
- currentSessionId: `Session A`
- currentWorkspace: `Workspace 1`

**操作:** Session AのタブのCloseButtonをクリック

**期待動作:**
1. Session Aのタブが閉じる
2. Workspace 1内の残りSessionを検索 → 0件
3. 新規Session自動作成（Workspace 1に作成）
4. Session Bは別Workspaceなので、選択されない

**実装:**
```
remainingSessions = sessions.filter(
  s => s.workspace_id === workspace.id  // Workspace 1
    && s.id !== closingTab.sessionId
)
// remainingSessions = [] （Session Bは別Workspace）
createSession()
```

---

## 型システムによる保証

### Session.workspace_id の必須化（既存）

**型定義:**
```typescript
// TypeScript
export interface Session {
  workspace_id: string;  // Required
}

// Rust
pub struct Session {
    pub workspace_id: String,
}
```

**効果:**
- すべてのSessionは必ずWorkspaceに紐づく
- Workspace内のSession取得が型安全

### SessionTab.workspaceId の必須化（既存）

**型定義:**
```typescript
export interface SessionTab {
  workspaceId: string;  // Required
}
```

**効果:**
- Tab切り替え時にWorkspace情報を特定可能
- `getVisibleTabs(workspaceId)` で型安全にフィルター

---

## 検証項目

- ✅ 非ActiveSessionのTab Closeで何も起きない（タブのみ閉じる）
- ✅ ActiveSessionのTab Closeで次のSessionに切り替わる
- ✅ 次のSessionは更新日時が直近のもの
- ✅ 切り替え後、そのSessionのTabが開く（または既存Tabにフォーカス）
- ✅ 閉じたタブが消える
- ✅ 閉じたSessionのタブが再オープンされない
- ✅ SessionListの選択状態が正しく更新される（青背景）
- ✅ タブが重複しない
- ✅ Workspace内最後のSessionのTab Closeで新規Session作成
- ✅ 無限ループなし
- ✅ TypeScript compilation成功

---

## 将来の拡張性

### 1. Tab Close時のカスタマイズ

現在は「更新日時が直近のSession」に切り替えるが、将来的に以下のような選択肢を提供可能:

- **ユーザー設定:** 「最近使った順」「作成日時順」「アルファベット順」など
- **スマート選択:** 頻繁に切り替えているSession、関連するSession（同じWorkspace内）など

### 2. Tab Close時のコールバック

TabProvider.onTabClosedコールバックを追加することで、以下が可能:

```typescript
<TabProvider onTabClosed={(tabId, sessionId) => {
  // Analytics送信
  trackTabClose(tabId, sessionId);

  // Plugin通知
  pluginManager.emit('tab:closed', tabId, sessionId);
}}>
```

### 3. Undo機能

閉じたタブを復元する機能:

- 閉じたタブの履歴を保持
- Ctrl+Shift+T で最後に閉じたタブを再オープン

---

## 参考資料

- **SDEプロトコル:** `workspace/SDE_tab_close.md` - 設計案の比較検討プロセス、調査結果
- **App State & Event Architecture:** `docs/specs/app-state-event-design.md` - Context Bridge パターン
- **Default Workspace仕様:** `docs/specs/default-workspace.md` - workspace_id必須化の背景

---

## 更新履歴

| 日付 | 変更内容 |
|------|----------|
| 2025-01-08 | 初版作成（Tab Close & Active Session Management機能実装） |
