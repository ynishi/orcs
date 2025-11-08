# Workspace Panel UI Design Specification

**Status**: ✅ Implemented (2025-11-08)
**Related**: `session-list-ui-v3.1.0.md`

---

## 概要

Workspace Panel UIの改善と、SessionListパターンへの統一。

**課題**:
- WorkspacePanel全体に固定高さ制限（400px）があり、画面下部に無駄な空白が発生
- FileListがSessionListと異なるUIパターンで統一感がない
- ファイルアイコンがコンテンツエリアに表示され、視覚的階層が不明確

**解決策**:
- WorkspacePanelの固定高さを削除し、flexレイアウトで画面全体に伸びるように変更
- FileListをSessionListと同じカード形式レイアウトに変更
- ファイルアイコンをヘッダー行に移動して視覚的階層を明確化

---

## 1. WorkspacePanelレイアウト仕様

### 1.1. 全体構造

```
┌─────────────────────────────────────┐
│ Workspace Files              🗂️ 💻 ➕ │ ← ヘッダー
│ ○ Include in prompt                 │ ← トグル
├─────────────────────────────────────┤
│ ┌─────────────────────────────────┐ │
│ │ FileList (Scrollable)           │ │ ← Flexで伸びる
│ │ ...                             │ │
│ │ ...                             │ │
│ └─────────────────────────────────┘ │
├─────────────────────────────────────┤
│ 2 items                             │ ← フッター
└─────────────────────────────────────┘
```

### 1.2. 実装詳細

**Stack全体**:
```tsx
<Stack gap="xs" style={{
  display: 'flex',
  flexDirection: 'column',
  height: '100%'
}}>
```

**変更前**:
```tsx
style={{ maxHeight: '400px' }}  // 固定高さ制限
```

**ScrollArea**:
```tsx
<ScrollArea style={{ flex: 1 }} px="sm">
```

**変更前**:
```tsx
<ScrollArea h={280} px="sm">  // 固定高さ
```

### 1.3. レイアウト効果

| 要素 | 変更前 | 変更後 | 効果 |
|-----|-------|-------|------|
| Stack | maxHeight: 400px | height: 100% | 親コンテナの高さいっぱいまで伸びる |
| ScrollArea | h={280} | flex: 1 | ヘッダー/フッター以外のスペースを全て使用 |

---

## 2. FileListレイアウト仕様

### 2.1. カード構造（SessionListパターン統一）

```
┌────────────────────────────────────────┐
│ 🦀 ⋮                                   │ ← TOPメニュー行
├────────────────────────────────────────┤
│ main.rs                                │ ← コンテンツエリア
│ 12.8 KB • Text • From chat             │
│ 1h ago                                 │
└────────────────────────────────────────┘
```

### 2.2. 実装詳細

**カード外観**:
```tsx
{
  borderRadius: '8px',
  border: '1px solid var(--mantine-color-gray-3)',
  backgroundColor: file.id === selectedFileId ? '#e7f5ff' : 'white',
  transition: 'all 0.15s ease',
  cursor: 'pointer',
  overflow: 'hidden',
}
```

**TOPメニュー行**:
```tsx
<Group
  gap="xs"
  px="md"
  py="xs"
  justify="space-between"
  style={{
    backgroundColor: file.id === selectedFileId ? '#d0ebff' : '#f8f9fa',
    borderBottom: '1px solid var(--mantine-color-gray-3)',
  }}
>
  {/* ファイルアイコン（左寄せ） */}
  <Text size="lg">{getFileIcon(file)}</Text>

  {/* コンテキストメニュー */}
  <Menu position="bottom-end" withinPortal>
    <Menu.Target>
      <ActionIcon><IconDotsVertical /></ActionIcon>
    </Menu.Target>
  </Menu>
</Group>
```

**変更前**:
```tsx
justify="flex-end"  // 右寄せのみ
// アイコンなし
```

**コンテンツエリア**:
```tsx
<Box p="md">
  <Box style={{ flex: 1, minWidth: 0 }}>
    {/* Primary: ファイル名 */}
    <Text size="sm" fw={600} truncate>
      {file.name}
    </Text>

    {/* Secondary: サイズ + タイプ + From chat Badge */}
    <Group gap="xs" mt={4}>
      <Text size="xs" c="dimmed">{formatFileSize(file.size)}</Text>
      <Text size="xs" c="dimmed">•</Text>
      <Text size="xs" c="dimmed">{getFileTypeCategory(file.mimeType)}</Text>
      {file.sessionId && (
        <>
          <Text size="xs" c="dimmed">•</Text>
          <Badge size="xs" variant="light" color="violet">
            From chat
          </Badge>
        </>
      )}
    </Group>

    {/* Tertiary: 相対時間 */}
    <Text size="xs" c="dimmed" mt={2}>
      {formatRelativeTime(file.uploadedAt)}
    </Text>
  </Box>
</Box>
```

**変更前**:
```tsx
<Group gap="sm" wrap="nowrap">
  <Text size="lg">{getFileIcon(file)}</Text>  // アイコン重複
  <Box style={{ flex: 1, minWidth: 0 }}>
    <Text size="sm" fw={500} truncate>{file.name}</Text>
    <Text size="xs" c="dimmed">{formatFileSize(file.size)}</Text>
  </Box>
</Group>
```

### 2.3. スペーシング（SessionListと統一）

| 要素 | 値 | 根拠 |
|-----|-----|------|
| アイテム間ギャップ | 4px | SessionListと同じ |
| 水平パディング（カード内） | 16px (md) | Material Design標準 |
| TOPメニュー垂直パディング | 8px (xs) | コンパクト |
| コンテンツ垂直パディング | 16px (md) | 読みやすさ |

---

## 3. ファイルアイコン仕様

### 3.1. アイコンマッピング

```tsx
const getFileIcon = (file: UploadedFile) => {
  const ext = file.name.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'rs': return '🦀';
    case 'ts':
    case 'tsx': return '📘';
    case 'js':
    case 'jsx': return '📜';
    case 'md': return '📝';
    case 'json': return '⚙️';
    case 'toml': return '📋';
    default: return '📄';
  }
};
```

### 3.2. 配置変更

**変更前**:
- コンテンツエリアの左側に表示
- ファイル名と同じ行

**変更後**:
- ヘッダー行の左側に表示
- メニューアイコンと対称配置
- コンテンツエリアからは削除（重複排除）

---

## 4. メニュー構成

### 4.1. メニュー内容

```tsx
<Menu position="bottom-end" withinPortal>
  {/* Go to conversation（sessionIdがある場合のみ） */}
  {file.sessionId && (
    <>
      <Menu.Item leftSection={<IconMessageCircle />} color="violet">
        Go to conversation
      </Menu.Item>
      <Menu.Divider />
    </>
  )}

  <Menu.Item leftSection={<IconMessage />}>
    Attach to chat
  </Menu.Item>

  <Menu.Item leftSection={<IconExternalLink />}>
    Open file
  </Menu.Item>

  <Menu.Item leftSection={<IconPencil />}>
    Rename
  </Menu.Item>

  <Menu.Divider />

  <Menu.Item leftSection={<IconTrash />} color="red">
    Delete
  </Menu.Item>
</Menu>
```

### 4.2. 変更点

**変更前**:
- ホバー時に5個のActionIconを表示
- SessionIdの有無で表示切り替え

**変更後**:
- ドットアイコンのMenuコンポーネント
- SessionListと同じパターン
- タッチデバイス対応

---

## 5. ファイル情報表示の改善

### 5.1. 追加情報

| 情報 | 実装 | 説明 |
|-----|------|------|
| ファイルタイプ | `getFileTypeCategory()` | Text, Image, PDF, Codeなど |
| 相対時間 | `formatRelativeTime()` | "just now", "1h ago", "3d ago" |
| From chat Badge | `file.sessionId` | チャットから添付されたファイル |

### 5.2. formatRelativeTime実装

```tsx
function formatRelativeTime(timestamp: number): string {
  const now = Date.now() / 1000;
  const diff = now - timestamp;
  const minutes = Math.floor(diff / 60);
  const hours = Math.floor(diff / 3600);
  const days = Math.floor(diff / 86400);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  if (days < 7) return `${days}d ago`;
  return new Date(timestamp * 1000).toLocaleDateString();
}
```

### 5.3. getFileTypeCategory実装

```tsx
function getFileTypeCategory(mimeType: string): string {
  if (mimeType.startsWith('text/')) return 'Text';
  if (mimeType.startsWith('image/')) return 'Image';
  if (mimeType.startsWith('video/')) return 'Video';
  if (mimeType.startsWith('audio/')) return 'Audio';
  if (mimeType.includes('pdf')) return 'PDF';
  if (mimeType.includes('json')) return 'JSON';
  if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('gz')) return 'Archive';
  if (mimeType.includes('javascript') || mimeType.includes('typescript')) return 'Code';
  return 'File';
}
```

---

## 6. SessionListとの統一性

### 6.1. 共通パターン

| 要素 | SessionList | FileList | 統一 |
|-----|-------------|----------|------|
| カード外観 | border + radius | border + radius | ✅ |
| TOPメニュー背景 | #f8f9fa / #d0ebff | #f8f9fa / #d0ebff | ✅ |
| 選択状態背景 | #e7f5ff | #e7f5ff | ✅ |
| メニューアイコン | ⋮ | ⋮ | ✅ |
| アイテム間ギャップ | 4px | 4px | ✅ |
| パディング | md(16px) | md(16px) | ✅ |

### 6.2. 差異

| 要素 | SessionList | FileList | 理由 |
|-----|-------------|----------|------|
| TOPメニュー左側 | ⭐ | 🦀 (ファイルアイコン) | Favoriteがファイルには不要 |
| Primary情報 | Session Title | File Name | 対象が異なる |
| Secondary情報 | Workspace + msgs + time | Size + Type + time | 表示する情報が異なる |

---

## 7. 実装ファイル

| ファイル | 変更内容 |
|---------|---------|
| `orcs-desktop/src/components/workspace/WorkspacePanel.tsx` | 固定高さ削除、flexレイアウト化 |
| `orcs-desktop/src/components/files/FileList.tsx` | カード形式レイアウト、ファイルアイコン移動、メニュー化 |

---

## 8. 検証結果

### 8.1. レイアウト
- ✅ WorkspacePanelが画面下部まで伸びる
- ✅ ScrollAreaが適切にスクロール可能
- ✅ 無駄な空白が解消

### 8.2. 視認性改善
- ✅ カード形式で情報が整理され、見やすくなった
- ✅ ファイルアイコンがヘッダーに配置され、視覚的階層が明確
- ✅ SessionListとの統一感が向上

### 8.3. 操作性
- ✅ メニューで全操作にアクセス可能
- ✅ タッチデバイス対応
- ✅ ファイル情報（タイプ、時間）が一目で確認可能

---

## 9. 将来拡張ポイント

### 9.1. ファイルプレビュー
```
┌────────────────────────────────────────┐
│ 🖼️ ⋮                                   │
├────────────────────────────────────────┤
│ screenshot.png                         │
│ [サムネイル画像]                        │
│ 2.3 MB • Image • From chat             │
└────────────────────────────────────────┘
```

### 9.2. ファイルタグ
```
┌────────────────────────────────────────┐
│ 📄 [design] [v2] ⋮                     │ ← タグ表示
├────────────────────────────────────────┤
│ spec.md                                │
│ ...                                    │
└────────────────────────────────────────┘
```

### 9.3. ソート・フィルター
- ファイルタイプでフィルター
- サイズ、日時でソート
- From chatのみ表示

---

**Implementation Date**: 2025-11-08
**Contributors**: Claude Code (Sonnet 4.5), User Feedback
