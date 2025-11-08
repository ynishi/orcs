# Session List UI Design Specification v3.1.0

**Status**: ✅ Validated (2025-11-08)
**SDE Report**: `/workspace/SDE_list.md`

---

## 概要

SessionList UIの情報表示整理とカード形式レイアウトの実装仕様。

**課題**:
- Session情報が3箇所（ListItem, TOPMenu, TabName）に無秩序に分散
- ListItemに操作アイコン6個が圧迫、視認性低下

**解決策**:
- カード形式レイアウト（TOPメニュー + コンテンツエリア）
- ハイブリッド操作パターン（⭐常時 + ⋮メニュー）
- 情報の役割分担明確化（重複排除）

---

## 1. ListItemレイアウト仕様

### 1.1. カード構造

```
┌────────────────────────────────────────┐
│ ⭐ ⋮                                   │ ← TOPメニュー行
├────────────────────────────────────────┤
│ Session Title (max 2 lines, bold)      │ ← コンテンツエリア
│ 📁 Workspace • 3 msgs • just now       │
└────────────────────────────────────────┘
```

### 1.2. 実装詳細

**カード外観**:
```tsx
{
  borderRadius: '8px',
  border: '1px solid var(--mantine-color-gray-3)',
  backgroundColor: isCurrentSession ? '#e7f5ff' : 'white',
  overflow: 'hidden',
}
```

**TOPメニュー行**:
```tsx
<Group
  gap="xs"
  px="md"
  py="xs"
  justify="flex-end"
  style={{
    backgroundColor: isCurrentSession ? '#d0ebff' : '#f8f9fa',
    borderBottom: '1px solid var(--mantine-color-gray-3)',
  }}
>
  <ActionIcon>⭐</ActionIcon>  // Favorite Toggle
  <Menu.Target>
    <ActionIcon><IconDotsVertical /></ActionIcon>
  </Menu.Target>
</Group>
```

**コンテンツエリア**:
```tsx
<Box p="md">
  <Text size="sm" fw={600} lineClamp={2}>
    {session.title}
  </Text>
  <Group gap="xs" mt={4}>
    <Badge>{workspace}</Badge>
    <Text c="dimmed">{msgs} msgs</Text>
    <Text c="dimmed">{timestamp}</Text>
  </Group>
</Box>
```

### 1.3. スペーシング

| 要素 | 値 | 根拠 |
|-----|-----|------|
| アイテム間ギャップ | 4px | リスト密度 |
| 水平パディング（リスト） | 16px (md) | Material Design標準 |
| 水平パディング（カード内） | 16px (md) | 一貫性 |
| TOPメニュー垂直パディング | 8px (xs) | コンパクト |
| コンテンツ垂直パディング | 16px (md) | 読みやすさ |

---

## 2. 操作パターン（ハイブリッド）

### 2.1. 常時表示

| アイコン | 機能 | 理由 |
|---------|------|------|
| ⭐/☆ | Favorite Toggle | 高頻度操作、視覚的重要性 |
| ⋮ | Menu | 全操作へのアクセスポイント |

### 2.2. メニュー内容

```tsx
<Menu position="bottom-end" withinPortal>
  // Favoriteが2個以上の場合のみ表示
  {isFavorite && count >= 2 && (
    <>
      <Menu.Item leftSection={<IconArrowUp />}>Move Up</Menu.Item>
      <Menu.Item leftSection={<IconArrowDown />}>Move Down</Menu.Item>
      <Menu.Divider />
    </>
  )}

  <Menu.Item leftSection={<IconPencil />}>Rename</Menu.Item>
  <Menu.Item leftSection={<IconArchive />}>Archive / Unarchive</Menu.Item>
  <Menu.Divider />
  <Menu.Item leftSection={<IconTrash />} color="red">Delete</Menu.Item>
</Menu>
```

### 2.3. 利点

- ✅ 高頻度操作（Favorite）は1クリック
- ✅ ホバー時の圧迫感解消（6個→0個）
- ✅ 将来の操作追加に対応（メニュー内追加）
- ✅ タッチデバイス対応可能

---

## 3. 情報の役割分担

### 3.1. 3箇所の表示内容

| 箇所 | 役割 | 表示内容 |
|-----|------|---------|
| **ListItem** | Session選択UI（SSoT） | Title + Workspace + msgs + timestamp + ⭐ + ⋮ |
| **TOPMenu** | グローバルコンテキスト | User + Workspace |
| **TabName** | 開いているSession識別 | Session Title (truncate) |

### 3.2. 重複排除

**変更前**:
- Title: 3箇所
- msgs: 2箇所（ListItem + TOPMenu）
- Workspace: 2箇所（ListItem + TOPMenu）

**変更後**:
- Title: 2箇所（ListItem + TabName）← 役割が異なるため許容
- msgs: 1箇所（ListItem）← TOPMenuから削除
- Workspace: 2箇所 ← グローバルコンテキストとして必要

**TOPMenu変更**:
```tsx
// 削除: Session情報（Title + msgs）
// 保持: User + Workspace（グローバルコンテキスト）
```

---

## 4. 将来拡張ポイント

### 4.1. Session.summary フィールド（V4.0.0想定）

**ユーザーニーズ**:
- 過去スレ（直近10-20件）を頻繁に見返す
- Chatロード無しで概要を知りたい

**提案実装**:
```rust
pub struct Session {
    // ... 既存フィールド

    /// Session概要（2-3行、最大200文字程度）
    #[serde(default)]
    pub summary: Option<String>,
}
```

**UI拡張**:
```
┌────────────────────────────────────────┐
│ ⭐ ⋮                                   │
├────────────────────────────────────────┤
│ Title (1 line, bold)                   │
│ Summary text... (2 lines, dimmed)      │
│ 📁 Workspace • 3 msgs • just now       │
└────────────────────────────────────────┘
高さ: 88px ← 現在72pxから拡張
```

### 4.2. TAG表示（将来）

TOPメニュー行の拡張余地:
```tsx
<Group justify="space-between">  // 左右分離
  <Group gap="xs">
    {tags.map(tag => <Badge key={tag}>{tag}</Badge>)}
  </Group>
  <Group gap="xs">
    <ActionIcon>⭐</ActionIcon>
    <Menu.Target>...</Menu.Target>
  </Group>
</Group>
```

---

## 5. 実装ファイル

| ファイル | 変更内容 |
|---------|---------|
| `orcs-desktop/src/components/sessions/SessionList.tsx` | カード形式レイアウト、ハイブリッド操作パターン |
| `orcs-desktop/src/App.tsx:1378` | TOPMenuからSession情報削除 |

---

## 6. 検証結果

### 6.1. TypeScriptコンパイル
- ✅ エラーなし

### 6.2. 視認性改善
- ✅ カード形式で情報が整理され、見やすくなった
- ✅ TOPメニュー行とコンテンツエリアの分離が明確
- ✅ 操作アイコンの圧迫感解消

### 6.3. 操作性
- ✅ Favorite toggleは1クリック（高頻度操作）
- ✅ その他操作はメニューで集約（発見性向上）
- ✅ 将来の拡張に対応

---

## 7. 参考資料

- **SDE探索レポート**: `/workspace/SDE_list.md`
- **事前調査**: `/workspace/SDE_list.md` - Modern UI/UX List Design Best Practices
- **Material Design**: List Design Guidelines
- **Mantine UI**: Menu Component, Card Component

---

**Validated by**: SDE Protocol v1.0
**Implementation Date**: 2025-11-08
**Contributors**: Claude Code (Sonnet 4.5), User Feedback
