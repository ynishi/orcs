# Tab Rendering Optimization

## Overview

ORCS Desktopのタブシステムにおけるレンダリング最適化の設計ドキュメント。

複数タブを開いた際のパフォーマンス低下を、条件付きレンダリングによって解決する実装パターンについて説明する。

## Problem

### 症状
- 複数のタブ（セッション）を開いた状態で、入力フィールドへの入力が重くなる
- タブが増えるほど、UI全体の反応速度が低下する

### 原因
初期実装では、すべてのタブ（アクティブ・非アクティブ問わず）が常に完全にレンダリングされていた：

```tsx
{visibleTabs.map((tab) => (
  <Tabs.Panel key={tab.id} value={tab.id}>
    <ChatPanel tab={tab} {...props} />
  </Tabs.Panel>
))}
```

Mantine UIの`Tabs.Panel`は、非アクティブなタブを`display: none`で隠すだけで、以下の処理はすべてのタブで実行される：

1. **メッセージリスト全体のレンダリング**（各メッセージコンポーネントの評価）
2. 入力フォームの再評価
3. 各種useEffectフックの実行
4. 状態同期処理

特に、大量のメッセージを含むタブが複数あると、入力のたびに全タブのメッセージリストが再評価され、著しいパフォーマンス低下を引き起こす。

## Solution

### 条件付きレンダリング

アクティブなタブのみ、メッセージリストを完全にレンダリングする：

```tsx
{isActive ? (
  // Active tab: Render full message list
  <MessageList
    messages={tab.messages}
    onSaveMessageToWorkspace={onSaveMessageToWorkspace}
    onExecuteAsTask={onExecuteAsTask}
    onCreateSlashCommand={handleCreateSlashCommand}
    workspaceRootPath={workspace?.rootPath}
  />
) : (
  // Inactive tab: Lightweight placeholder
  <Box p="md" c="dimmed" ta="center">
    <Text size="sm">{tab.messages.length} messages (tab inactive)</Text>
  </Box>
)}
```

### 実装箇所

1. **ChatPanel.tsx** (`orcs-desktop/src/components/chat/ChatPanel.tsx`)
   - `isActive: boolean` プロパティを追加
   - メッセージリストの条件付きレンダリング

2. **App.tsx** (`orcs-desktop/src/App.tsx`)
   - `ChatPanel`に`isActive={activeTabId === tab.id}`を渡す

## Design Rationale

### なぜ全タブをDOMに保持するのか

完全にアンマウント（DOM削除）せず、DOMに保持したまま条件付きレンダリングする理由：

#### 1. バックエンドイベントの受信継続

バックエンドから送られる`dialogue-turn`イベント（ストリーミングメッセージ）は、**すべてのセッション（タブ）**に対して発生する：

```tsx
// App.tsx: dialogue-turn リスナー
listen<StreamingDialogueTurn>('dialogue-turn', (event) => {
  const turn = event.payload;
  const targetTab = getTabBySessionId(turn.session_id);

  if (!targetTab) return; // タブが存在しなければ無視

  // タブの状態を更新（アクティブ/非アクティブ問わず）
  addMessageToTab(targetTab.id, newMessage);
});
```

もしタブをアンマウントすると、以下の問題が発生する：
- バックエンドからのイベントを受信できない
- メッセージ履歴が更新されない
- AutoMode実行中のタブをバックグラウンドで監視できない

#### 2. AutoModeなどのバックグラウンド実行

AutoMode（自律的な連続対話）は、非アクティブなタブでも実行される可能性がある：

```tsx
// ユーザーが別のタブに切り替えても、AutoModeは継続
// バックエンドが dialogue-turn イベントを送信し続ける
// → 非アクティブタブでも状態更新が必要
```

#### 3. タブ状態の保持

各タブは以下の状態を持ち、タブ切り替え時もこれらを保持する必要がある：

```typescript
interface SessionTab {
  messages: Message[];        // メッセージ履歴
  input: string;              // 入力中のテキスト
  attachedFiles: File[];      // 添付ファイル
  isAiThinking: boolean;      // AI思考中フラグ
  autoMode: boolean;          // AutoModeフラグ
  // ... その他の状態
}
```

アンマウントすると、これらの状態を別途保存・復元する仕組みが必要になり、実装が複雑化する。

### 状態管理とレンダリングの分離

この設計の核心は、**状態管理とレンダリングを分離**することにある：

| レイヤー | 対象 | 動作 |
|---------|------|------|
| **TabContext** | すべてのタブ | 常に状態を管理・更新 |
| **ChatPanel** | アクティブタブのみ | 重いコンポーネントをレンダリング |

```
┌─────────────────────────────────────┐
│ TabContext (State Management)      │ ← すべてのタブの状態を保持
│  - tabs: SessionTab[]              │ ← バックエンドイベントで更新
│  - activeTabId: string             │
└─────────────────────────────────────┘
                 ↓
    ┌────────────┴────────────┐
    │                         │
┌───┴────┐              ┌────┴────┐
│ Tab A  │ (active)     │ Tab B   │ (inactive)
│ ━━━━━━ │              │ ─ ─ ─ ─ │
│ Full   │              │ Light   │
│ Render │              │ Render  │
└────────┘              └─────────┘
```

## Implementation Details

### ChatPanel.tsx

```tsx
export function ChatPanel({
  tab,
  isActive,  // ← 追加
  // ... other props
}: ChatPanelProps) {
  // タブの状態更新（全タブで実行）
  useEffect(() => {
    // AutoChat設定の読み込みなど
  }, [tab.sessionId]);

  return (
    <Stack>
      <ScrollArea>
        {isActive ? (
          // 重いメッセージリスト（アクティブタブのみ）
          <MessageList messages={tab.messages} {...handlers} />
        ) : (
          // 軽量プレースホルダー（非アクティブタブ）
          <Box><Text>{tab.messages.length} messages</Text></Box>
        )}
      </ScrollArea>

      {/* 入力フォームは全タブでレンダリング（状態保持のため） */}
      <InputForm value={tab.input} {...handlers} />
    </Stack>
  );
}
```

### App.tsx

```tsx
{visibleTabs.map((tab) => (
  <Tabs.Panel key={tab.id} value={tab.id}>
    <ChatPanel
      tab={tab}
      isActive={activeTabId === tab.id}  // ← 追加
      {...props}
    />
  </Tabs.Panel>
))}
```

## Trade-offs

### メモリ vs CPU

| アプローチ | メモリ使用量 | CPU使用量 | 実装複雑度 |
|-----------|------------|-----------|----------|
| **完全アンマウント** | 少ない | 多い（再マウント時） | 高い（状態保存が必要） |
| **条件付きレンダリング** | 多い（DOMツリー保持） | 少ない（差分更新のみ） | 低い |
| **仮想化 (react-window)** | 中程度 | 少ない | 中程度 |

### 選択理由

本実装では**条件付きレンダリング**を採用：

**利点**:
- ✅ バックエンドイベント受信が継続
- ✅ AutoMode等のバックグラウンド実行をサポート
- ✅ タブ状態が自然に保持される
- ✅ 実装がシンプル

**欠点**:
- ⚠️ 大量のタブを開くとメモリ使用量が増加
- ⚠️ 非アクティブタブのuseEffectは実行される（ただし軽量）

### メモリ使用量の考察

通常のユースケース（5〜10タブ程度）では、メモリ増加は許容範囲：

- 各タブのDOMツリー（入力フォームのみ）: 〜50KB
- メッセージ履歴データ: 100メッセージで〜500KB
- 合計: 10タブで〜5MB程度

→ 現代のデスクトップ環境では問題なし

## Future Improvements

パフォーマンスやメモリ使用量がさらに問題になった場合の改善案：

### 1. タブのLazy Loading

一定数（例: 5タブ）を超えたら、古いタブをアンマウント：

```tsx
const ACTIVE_TAB_LIMIT = 5;
const shouldRender = isActive || isRecentlyAccessed(tab, ACTIVE_TAB_LIMIT);
```

### 2. メッセージ仮想化 (react-window)

大量のメッセージがある場合、表示領域のみレンダリング：

```tsx
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={messages.length}
  itemSize={100}
>
  {({ index, style }) => (
    <MessageItem message={messages[index]} style={style} />
  )}
</FixedSizeList>
```

### 3. バックグラウンドセッションタイプ

UI不要のバックグラウンド実行専用セッション：

```typescript
interface SessionTab {
  type: 'interactive' | 'background';
  // background タイプは UI を持たない
}
```

### 4. Web Worker活用

メッセージ処理をメインスレッドから分離：

```tsx
// Worker でメッセージをパース・整形
// → UI スレッドはレンダリングのみ
```

## References

- **Implementation PR**: `feature/optimize-tab-rendering`
- **Related Files**:
  - `orcs-desktop/src/App.tsx` (L2105-2149)
  - `orcs-desktop/src/components/chat/ChatPanel.tsx` (L398-412)
  - `orcs-desktop/src/context/TabContext.tsx`

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 2025-11-15 | Claude Code | Initial implementation and documentation |
