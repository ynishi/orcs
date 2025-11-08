# App State & Event Architecture 仕様

## 概要

Reactアプリケーション（ORCS Desktop）における**State管理**と**コンポーネント間イベント連携**のアーキテクチャ設計。

Tab切り替え時にWorkspace情報を連動させる要件から、「Context Bridge パターン」を採用し、既存の`workspace-switched`イベントリスナーを活用する設計を確立した。

---

## 背景と課題

### 問題の発見

**タスク:** Tab（Session）を開いた際、そのSessionに紐づくWorkspace情報が左ペインに反映されない

**原因分析:**
1. **Tab切り替え時にWorkspace情報が更新されない**
   - `TabContext.switchTab()` がWorkspaceContextに何も通知していない
   - `tab.workspaceId` フィールドは存在するが活用されていない

2. **Session選択時に間違ったworkspaceIdを使用**
   - `handleSessionSelect()` が現在の `workspace.id` を使用
   - `session.workspace_id` を使っていない（型定義もOptionalだった）

3. **イベント連携パターンの不一致**
   - Workspace切り替え: `workspace-switched` イベントで全体同期 ✅
   - Tab切り替え: イベントなし、同期なし ❌
   - Session選択: 現在のWorkspaceに依存 ❌

---

## アーキテクチャ構造

### コンポーネント階層

```
App (State統合管理層)
 ├─ LeftMenu (Navbar)
 │   ├─ SessionList        - Session一覧表示
 │   ├─ WorkspacePanel     - Workspace情報表示
 │   └─ TasksList          - タスク一覧表示
 └─ RightPanel
     ├─ Top
     │   ├─ Header         - アプリタイトル、ユーザー情報
     │   └─ WorkspaceSwitcher - Workspace切り替えUI
     └─ View
         └─ Tabs           - 複数Session表示（タブUI）
```

### State管理の責務分離

| Context | 責務 | 管理するState |
|---------|------|---------------|
| **TabContext** | Tab（Session UI）の状態管理 | `tabs[]`, `activeTabId` |
| **WorkspaceContext** | Workspace情報の管理 | `workspace`, `allWorkspaces`, `files` |
| **SessionContext** | Session（会話履歴）の管理 | `sessions[]`, `currentSessionId` |

### イベント連携の要求

1. **LeftMenu → View**: Session選択 → Tabを開く
2. **View → LeftMenu**: Tab切り替え → Workspaceバッジ更新、Session強調表示、ファイルリスト更新
3. **Top → LeftMenu**: Workspace切り替え → Session一覧フィルター、タスク一覧更新
4. **View → Top**: Tab切り替え → Workspace表示更新

---

## 設計判断

### 判断1: イベント連携パターンの選択

**検討した設計案:**

#### A. App中央集権ルーティング
- すべてのイベントハンドラーをApp.tsxに集約
- 子コンポーネントは純粋な表示 + イベント発火のみ

**メリット:**
- シンプルで追跡しやすい
- 既存パターンの拡張
- デバッグ容易

**デメリット:**
- App肥大化
- 密結合

#### B. Event Emitter パターン
- カスタムイベントシステム導入
- 各コンポーネントがイベントをemit/listen

**メリット:**
- 完全な疎結合
- 拡張性が高い

**デメリット:**
- 複雑性が高い（追跡困難）
- オーバーエンジニアリング
- メモリリークのリスク

#### C. Context Bridge パターン ✅ 採用
- Contextを**State管理**と**イベント仲介**の両方に使う
- AppでContextをブリッジし、相互参照を管理

**メリット:**
- 既存パターンの自然な拡張
- 適度な疎結合
- 型安全（TypeScript）
- デバッグ容易
- Reactイディオム準拠

**デメリット:**
- 中規模の複雑性

### 判断2: 具体的な実装パターン

**選択肢:**
- **Option A**: TabProviderに`onTabSwitched`コールバックを注入し、App.tsxで受け取る
- **Option B**: `<Tabs onChange>`内で直接処理 ✅ 採用

**採用理由:**
- main.tsxでTabProviderが既に配置されており、Appからpropsを渡せない
- `<Tabs onChange>`内で直接処理する方がシンプル
- TabContext.onTabSwitchedは将来の拡張用に残す（現時点では未使用）

---

## 実装仕様

### 1. State要求の整理

#### 必須State（型システムで保証）

**Session.workspace_id: string**
- すべてのSessionは必ずWorkspaceに紐づく
- Default Workspace実装により、起動時に `~/orcs` が作成・保証される
- TypeScript型定義をRust側と同期（`workspace_id?: string` → `workspace_id: string`）

**TabContext.tabs[].workspaceId: string**
- 各TabはSessionを開いた時点のworkspaceIdを保持
- Tab切り替え時にWorkspace情報を特定するために使用

#### 派生State（計算可能）

```typescript
// 現在アクティブなTabのWorkspace ID
const activeWorkspaceId = tabs.find(t => t.id === activeTabId)?.workspaceId;

// 指定Workspaceに属するTab一覧
const visibleTabs = tabs.filter(t => t.workspaceId === currentWorkspaceId);
```

### 2. イベントフロー仕様

#### パターンA: Workspace切り替え（既存）

```
[トリガー] WorkspaceSwitcher UI

[フロー]
1. WorkspaceSwitcher → switchWorkspaceBackend(sessionId, workspaceId)
2. Backend: switch_workspace command 実行
3. Backend: 'workspace-switched' イベント発火
4. Frontend: workspace-switchedリスナー実行
   a. refreshWorkspace() - Workspace情報更新
   b. refreshSessions() - Session一覧更新
   c. switchWorkspaceTabs(workspaceId) - Tab切り替え
   d. openTab() / switchToTab() - 適切なTabにフォーカス
   e. setGitInfo() - Git情報更新

[保証]
- Workspace情報が全体で同期される
- 左ペイン（SessionList, WorkspacePanel, TasksList）が更新される
- 該当WorkspaceのTabが存在すればフォーカス、なければnull
```

#### パターンB: Tab切り替え（今回実装）

```
[トリガー] <Tabs> UIでTabクリック

[フロー]
1. <Tabs onChange> → tab情報を取得
2. switchToTab(tabId) - TabContext更新
3. switchSession(tab.sessionId) - Backend Session切り替え
4. 条件分岐: tab.workspaceId !== currentWorkspace?.id
   ├─ Yes: switchWorkspaceBackend(sessionId, workspaceId)
   │        ↓
   │   'workspace-switched' イベント発火
   │        ↓
   │   パターンAのフロー実行
   │
   └─ No: 何もしない（同一Workspace内の切り替え）

[保証]
- 同一Workspace内の切り替え: 不要なWorkspace更新なし
- 異なるWorkspace間の切り替え: 自動でWorkspace切り替え → 全体同期
- 無限ループなし（workspaceSwitchingRefによる排他制御）
```

#### パターンC: Session選択（今回実装）

```
[トリガー] 左ペインのSessionList → Session選択

[フロー]
1. handleSessionSelect(session)
2. 条件分岐: session.workspace_id !== currentWorkspace?.id
   ├─ Yes: switchWorkspaceBackend(session.id, session.workspace_id)
   │        ↓
   │   'workspace-switched' イベント発火 → パターンAのフロー
   │
   └─ No: スキップ
3. switchSession(session.id) - Session履歴取得
4. openTab(session, messages, session.workspace_id) - Tab開く

[保証]
- 異なるWorkspaceのSessionを選択 → Workspace自動切り替え
- Sessionのタブが開く
- 左ペインが同期
```

### 3. エッジケース仕様

#### Case 1: 同一Workspace内のTab切り替え

**状態:**
- workspace-1 にTab A, Tab B が存在
- 現在 Tab A がアクティブ

**操作:** Tab B をクリック

**期待動作:**
- `switchSession(Tab B.sessionId)` のみ実行
- `switchWorkspaceBackend()` 呼ばれない
- 左ペインのWorkspace情報は変わらない

**検証条件:**
```typescript
if (tab.workspaceId !== workspace?.id) {
  // 呼ばれない
}
```

#### Case 2: 異なるWorkspace間のTab切り替え

**状態:**
- workspace-1 にTab A
- workspace-2 にTab C
- 現在 Tab A (workspace-1) がアクティブ

**操作:** Tab C をクリック

**期待動作:**
1. `switchSession(Tab C.sessionId)` 実行
2. `switchWorkspaceBackend(sessionId, workspace-2)` 実行
3. `workspace-switched` イベント発火
4. 左ペイン全体が workspace-2 の情報に更新
5. SessionList, WorkspacePanel, TasksListがリフレッシュ

**検証条件:**
- `refreshWorkspace()` が呼ばれる
- `workspace.id` が `workspace-2` に変わる

#### Case 3: 無限ループの防止

**懸念:**
```
handleTabSwitched → switchWorkspaceBackend → workspace-switched
→ switchWorkspaceTabs → switchToTab → handleTabSwitched → ...
```

**防止メカニズム:**
1. **排他制御:** `workspaceSwitchingRef` による処理中フラグ
2. **冪等性:** `switchWorkspaceTabs()` は既存タブにフォーカスするだけ（新規Tab開かない）
3. **条件分岐:** `tab.workspaceId === workspace?.id` の場合は何もしない

**検証条件:**
```typescript
// workspace-switchedリスナー (App.tsx L461-536)
if (workspaceSwitchingRef.current) {
  console.log('[App] workspace-switched event ignored (refresh already in progress)');
  return;
}
workspaceSwitchingRef.current = true;
try {
  // ...処理
} finally {
  workspaceSwitchingRef.current = false;
}
```

#### Case 4: タブがない状態でのWorkspace切り替え

**状態:**
- workspace-3 に切り替え
- workspace-3 にはタブが1つもない

**操作:** WorkspaceSwitcher → workspace-3

**期待動作:**
- `switchWorkspaceTabs(workspace-3)` 実行
- `activeTabId` が `null` になる
- Tabエリアに「No session opened」メッセージ表示

**実装:**
```typescript
// TabContext.switchWorkspace() (L342-353)
const switchWorkspace = useCallback((workspaceId: string) => {
  const workspaceTabs = tabs.filter((tab) => tab.workspaceId === workspaceId);

  if (workspaceTabs.length > 0) {
    const sortedTabs = [...workspaceTabs].sort((a, b) => b.lastAccessedAt - a.lastAccessedAt);
    setActiveTabId(sortedTabs[0].id);
  } else {
    setActiveTabId(null);  // ← タブなし
  }
}, [tabs]);
```

---

## 型システムによる保証

### Session.workspace_id の必須化

**Before:**
```typescript
// TypeScript
export interface Session {
  workspace_id?: string; // Optional
}
```

**After:**
```typescript
// TypeScript
export interface Session {
  workspace_id: string; // Required
}

// Rust (既に必須)
pub struct Session {
    pub workspace_id: String,
}
```

**効果:**
- `session.workspace_id` へのアクセスが型安全に
- Optionalチェーン不要（`session.workspace_id?.` → `session.workspace_id`）
- Default Workspace実装により、すべてのSessionにworkspace_idが保証される

---

## 実装ファイル

### 修正ファイル（3ファイル）

1. **`orcs-desktop/src/types/session.ts`**
   - `workspace_id?: string` → `workspace_id: string`

2. **`orcs-desktop/src/context/TabContext.tsx`**
   - `TabProviderProps` に `onTabSwitched` コールバック追加（将来の拡張用）
   - `switchTab()` でコールバック実行（現時点では未使用）

3. **`orcs-desktop/src/App.tsx`**
   - `<Tabs onChange>` でWorkspace切り替えロジック追加
   - `handleSessionSelect()` で `session.workspace_id` 使用 + Workspace切り替え対応

### コード例

#### App.tsx: <Tabs onChange> の実装

```typescript
<Tabs
  value={activeTabId}
  onChange={async (value) => {
    if (!value) return;

    const tab = tabs.find(t => t.id === value);
    if (!tab) return;

    // 1. タブを切り替え
    switchToTab(value);

    // 2. バックエンドのセッションも切り替え
    try {
      await switchSession(tab.sessionId);
    } catch (err) {
      console.error('[App] Failed to switch backend session:', err);
      return;
    }

    // 3. Workspace切り替え（必要な場合のみ）
    if (tab.workspaceId !== workspace?.id) {
      try {
        await switchWorkspaceBackend(tab.sessionId, tab.workspaceId);
        // ↑ 'workspace-switched' イベント発火 → 既存リスナーで全体同期
      } catch (err) {
        console.error('[App] Failed to switch workspace:', err);
      }
    }
  }}
>
```

#### App.tsx: handleSessionSelect() の実装

```typescript
const handleSessionSelect = async (session: Session) => {
  try {
    // 1. Workspace切り替え（必要なら）
    if (session.workspace_id !== workspace?.id) {
      await switchWorkspaceBackend(session.id, session.workspace_id);
      // ↑ 'workspace-switched' イベント発火 → 既存リスナーで全体同期
    }

    // 2. Session切り替え（履歴取得）
    const fullSession = await switchSession(session.id);
    const restoredMessages = convertSessionToMessages(fullSession, userNickname);

    // 3. Tabを開く（session.workspace_idを使用）
    openTab(fullSession, restoredMessages, session.workspace_id);
  } catch (err) {
    console.error('[App] Failed to select session:', err);
  }
};
```

---

## 将来の拡張性

### TabProvider.onTabSwitchedの活用

現在は`<Tabs onChange>`内で直接処理しているが、将来的に以下のような拡張が考えられる：

**シナリオ1: 複数の購読者が必要になった場合**
```typescript
// App.tsx
const handleTabSwitched = useCallback(async (tabId: string, workspaceId: string) => {
  // Workspace切り替え
  if (workspaceId !== workspace?.id) {
    await switchWorkspaceBackend(sessionId, workspaceId);
  }

  // その他の処理（例: Analytics送信、ログ記録など）
  trackTabSwitch(tabId, workspaceId);
}, [workspace, switchWorkspaceBackend]);

// main.tsx または App.tsx
<TabProvider onTabSwitched={handleTabSwitched}>
  <App />
</TabProvider>
```

**シナリオ2: Plugin Systemの導入**
```typescript
// Plugin APIでTab切り替えイベントを購読
pluginManager.on('tab:switched', (tabId, workspaceId) => {
  // プラグイン処理
});

// TabContext内でプラグインにも通知
if (targetWorkspaceId && onTabSwitched) {
  onTabSwitched(tabId, targetWorkspaceId);
  pluginManager.emit('tab:switched', tabId, targetWorkspaceId);
}
```

### Event Emitterへの移行基準

以下の条件を満たした場合、Event Emitterパターンへの移行を検討：

1. **コンポーネント数が50+**
2. **イベント種類が20+**
3. **購読者が複数（3+）存在するイベントが多数**
4. **動的なイベント購読・解除が頻繁**

現時点では **YAGNI（You Aren't Gonna Need It）** 原則に従い、Context Bridgeパターンで十分。

---

## 検証項目

- ✅ 同一Workspace内のTab切り替えで不要なWorkspace更新なし
- ✅ 異なるWorkspace間のTab切り替えで全体同期
- ✅ Session選択時のWorkspace自動切り替え
- ✅ 無限ループなし（排他制御確認済み）
- ✅ タブがない状態でのWorkspace切り替え対応
- ✅ TypeScriptコンパイルエラーなし
- ✅ Session.workspace_id の型安全性

---

## 参考資料

- **SDEプロトコル:** `workspace/SDE_tab.md` - 設計案の比較検討プロセス
- **Default Workspace仕様:** `docs/specs/default-workspace.md` - workspace_id必須化の背景
- **関連Issue:** Tab-Workspace連動機能の実装

---

## 更新履歴

| 日付 | 変更内容 |
|------|----------|
| 2025-01-XX | 初版作成（Context Bridge パターン採用） |
