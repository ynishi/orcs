# 追加アイコン修正リスト

**調査日**: 2025-11-24

---

## 📋 修正箇所一覧

### 1. Navbar.tsx - DialoguePresetsアイコン
**現在**: `IconPalette`（絵のパレット）
**問題**: 絵のパレットなので一見よくわからない
**変更先候補**:
- `IconAdjustments` - 設定感のあるスライダーアイコン
- `IconStack` - 紙の束（まとまり感）
- `IconLayoutList` - リスト形式

**推奨**: `IconAdjustments`（設定感）

---

### 2. StatusBar.tsx:239 - DialoguePresetアイコン
**現在**: `🎨` 絵文字
**問題**: 絵のパレットになっている
**変更先**: `<IconAdjustments size={16} />` または `<IconStack size={16} />`

---

### 3. MessageItem.tsx:230, 346 - コピーボタン
**現在**: `{copied ? '✓' : '📋'}` 絵文字
**問題**: 絵文字になっている
**変更先**: `<IconCheck />` と `<IconClipboard />`

---

### 4. ChatPanel.tsx:920 - MUTEボタン
**現在**: `{isMuted ? '🔇' : '🔊'}` 絵文字
**問題**: 絵文字になっている
**変更先**: `<IconVolume />` と `<IconVolumeOff />`

---

### 5. ChatPanel.tsx:931 - AUTOボタン
**現在**: `{autoMode ? '⏹️' : '▶️'}` 絵文字
**問題**: 絵文字になっている
**変更先**: `<IconPlayerStop />` と `<IconPlayerPlay />`

---

## 実装順序

1. Navbar DialoguePresets アイコン変更
2. StatusBar DialoguePreset アイコン変更
3. MessageItem コピーボタン変更
4. ChatPanel MUTE/AUTO ボタン変更
