# ＋ボタンスタイル調査レポート

**調査日**: 2025-11-24
**対象**: orcs-desktop の IconPlus 使用箇所

---

## 📊 使用箇所一覧

### 1. App.tsx:2340-2350 - タブ新規作成ボタン
```tsx
<ActionIcon
  variant="subtle"
  color="blue"
  size="md"
  onClick={...}
  style={{ marginLeft: '8px' }}
>
  <IconPlus size={16} />
</ActionIcon>
```
- **特徴**: ActionIcon に `size="md"` 指定あり、`marginLeft` あり

### 2. DialoguePresetList.tsx:125-127 - プリセット追加ボタン
```tsx
<ActionIcon variant="subtle" color="blue" onClick={...}>
  <IconPlus size={16} />
</ActionIcon>
```
- **特徴**: シンプルな標準スタイル

### 3. PersonasList.tsx:380-382 - ペルソナ追加ボタン
```tsx
<ActionIcon variant="subtle" color="blue" onClick={...}>
  <IconPlus size={16} />
</ActionIcon>
```
- **特徴**: シンプルな標準スタイル

### 4. SlashCommandList.tsx:216-218 - コマンド追加ボタン
```tsx
<ActionIcon variant="subtle" color="blue" onClick={...}>
  <IconPlus size={16} />
</ActionIcon>
```
- **特徴**: シンプルな標準スタイル

### 5. WorkspacePanel.tsx:247-255 - ファイルアップロードボタン（上部）
```tsx
<ActionIcon
  component="label"
  variant="subtle"
  color="blue"
  aria-label="Upload file"
>
  <IconPlus size={18} /> ⚠️ サイズ違い
  <input type="file" multiple hidden onChange={...} />
</ActionIcon>
```
- **特徴**: `size={18}` で他と異なる、`component="label"` でファイル入力
- **問題**: アイコンサイズが 18 で統一されていない

### 6. WorkspacePanel.tsx:317-324 - ファイルアップロードボタン（下部）
```tsx
<ActionIcon
  component="label"
  variant="subtle"
  color="blue"
  aria-label="Upload file"
>
  <IconPlus size={18} /> ⚠️ サイズ違い
  <input type="file" multiple hidden onChange={...} />
</ActionIcon>
```
- **特徴**: 同上、アイコンサイズ 18
- **問題**: アイコンサイズが 18 で統一されていない

### 7. WorkspaceSwitcher.tsx:272-276 - ワークスペース作成メニュー
```tsx
<Menu.Item
  leftSection={<IconPlus size={16} />}
  onClick={...}
  style={{
    backgroundColor: 'var(--mantine-color-green-light)',
  }}
>
  Create New Workspace
</Menu.Item>
```
- **特徴**: Menu.Item なので ActionIcon ではない、背景色が緑
- **問題**: 背景色の指定方法が他と異なる（直接 style 指定）

---

## 🎯 スタイル統一方針

### 標準スタイル（推奨）
```tsx
<ActionIcon variant="subtle" color="blue">
  <IconPlus size={16} />
</ActionIcon>
```

### 統一すべき項目

| 項目 | 標準値 | 現状の不統一 |
|------|--------|-------------|
| variant | `"subtle"` | ✅ 全箇所統一済み |
| color | `"blue"` | ✅ 全箇所統一済み |
| IconPlus size | `16` | ⚠️ WorkspacePanel の2箇所が `18` |
| ActionIcon size | 指定なし（デフォルト） | ⚠️ App.tsx のみ `"md"` 指定 |

---

## 🔧 修正が必要な箇所

### 優先度：高

#### 1. WorkspacePanel.tsx - アイコンサイズの統一
**箇所**: 2箇所（253行目、322行目）
**修正内容**:
```tsx
// Before
<IconPlus size={18} />

// After
<IconPlus size={16} />
```

### 優先度：低

#### 2. App.tsx - ActionIcon size の扱い
**箇所**: 2340行目
**現状**: `size="md"` が指定されている
**判断**:
- このボタンは特別な位置（タブリストの右端）にあるため、視認性のために `size="md"` が必要
- **対応**: 現状維持でOK、または他の＋ボタンも `size="md"` に統一するか検討

#### 3. WorkspaceSwitcher.tsx - Menu.Item のスタイル
**箇所**: 272-276行目
**現状**: backgroundColor を直接指定
**判断**:
- Menu.Item は ActionIcon ではないため、スタイル指定方法が異なるのは自然
- **対応**: 現状維持でOK

---

## ✅ 実装計画

### Phase 1: アイコンサイズの統一（必須）
1. WorkspacePanel.tsx の IconPlus を `size={16}` に変更

### Phase 2: ActionIcon size の検討（オプション）
2つの選択肢：
- **A**: App.tsx の `size="md"` を削除して全体をデフォルトに統一
- **B**: 全ての ActionIcon に `size="md"` を追加して統一

**推奨**: A（デフォルトサイズで統一）
- 理由: Mantineのデフォルトサイズは適切で、個別にサイズ指定する必要性は低い

---

## 📝 統一後の標準パターン

### パターン1: 通常の追加ボタン
```tsx
<Tooltip label="Add xxx" withArrow>
  <ActionIcon variant="subtle" color="blue" onClick={...}>
    <IconPlus size={16} />
  </ActionIcon>
</Tooltip>
```

### パターン2: ファイルアップロードボタン
```tsx
<ActionIcon
  component="label"
  variant="subtle"
  color="blue"
  aria-label="Upload file"
>
  <IconPlus size={16} />
  <input type="file" multiple hidden onChange={...} />
</ActionIcon>
```

### パターン3: メニューアイテム
```tsx
<Menu.Item
  leftSection={<IconPlus size={16} />}
  onClick={...}
>
  Menu Item Text
</Menu.Item>
```

---

## 🎨 統一のメリット

1. **視覚的一貫性**: 全ての＋ボタンが同じサイズ・スタイルで表示
2. **認知負荷の軽減**: ユーザーが「追加」機能を即座に認識
3. **保守性の向上**: 新しい＋ボタンを追加する際の標準パターンが明確
4. **デザインの品質**: プロフェッショナルな印象

---

## 次のステップ

- [ ] WorkspacePanel.tsx の IconPlus を `size={16}` に変更
- [ ] App.tsx の ActionIcon `size="md"` の扱いを決定
- [ ] 視覚的な確認とテスト
- [ ] 標準パターンをドキュメント化（オプション）
