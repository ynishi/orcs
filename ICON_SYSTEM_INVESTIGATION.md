# アイコンシステム調査レポート

**調査日**: 2025-11-24
**対象**: orcs-desktop アイコン使用状況

---

## 📊 現状サマリー

### 使用されているアイコンシステム

1. **絵文字（Unicode）**: 26ファイルで使用
2. **@tabler/icons-react（SVG）**: 15ファイルで使用

### 混在の問題

- **UI一貫性の欠如**: 同じUI要素でも絵文字とSVGが混在
- **視覚的な不統一**: フォント依存の絵文字とベクターのSVGで見た目が異なる
- **スケーラビリティ**: 絵文字はサイズ調整が難しい
- **カスタマイズ性**: SVGはテーマカラーに合わせられるが、絵文字は固定

---

## 🔍 詳細な使用箇所

### 1. 左アイコンバー（Navbar）- すべて絵文字

**ファイル**: `src/components/navigation/Navbar.tsx:114-151`

| 機能 | 現在のアイコン | 行番号 |
|------|---------------|--------|
| Sessions | 💬 | 115 |
| Workspace | 📁 | 122 |
| Tasks | ✅ | 128 |
| **Personas** | **⭐️** | 135 |
| Slash Commands | ⚡ | 141 |
| Dialogue Presets | 🎨 | 147 |

**問題点**:
- ⭐️（星）がペルソナ機能を表していない → **ミスリード**
- すべて絵文字なので、テーマカラーとの統一感がない

### 2. アクションボタン - すべて@tabler/icons-react

**PersonasList.tsx** (`src/components/personas/PersonasList.tsx:3`)
- `IconPlus` - 新規作成
- `IconPencil` - 編集
- `IconTrash` - 削除

**TaskList.tsx** (`src/components/tasks/TaskList.tsx:2`)
- `IconDeviceFloppy` - 保存

**ChatPanel.tsx** (`src/components/chat/ChatPanel.tsx:21`)
- `IconSettings` - 設定
- `IconClipboardList` - クリップボード
- `IconFileText` - ファイル
- `IconBulb` - アイデア

### 3. システムメッセージ・通知 - 主に絵文字

**App.tsx**
- 📎 - ファイル添付 (1124, 1424, 1447, 1466, 1506行目)
- 💾 - 保存 (1600, 1611, 1719行目)
- 🚀 - タスク実行 (1626行目)
- 📂 - フォルダ (1803行目)
- 🗑️ - 削除 (1830行目)
- 👋 - ウェルカム (2175行目)

**useSlashCommands.ts**
- 🚀 - タスク実行 (163行目)
- 🔶 - エキスパート作成 (213, 217行目)
- 📋💡📝🔧✅ - プランニングステップ (262, 271行目)

**AITextField.tsx**
- 💫 - 生成 (134, 197行目)
- 🖌️ - 修正 (144, 207行目)
- 🗒️ - 履歴 (169行目)
- 🏷️ - 方向性 (173, 177行目)
- 💬 - チャット (216, 223行目)

### 4. タスクステータス - 絵文字

**types/task.ts:64**
```typescript
return '🔄'; // Running状態のアイコン
```

---

## 📋 統一の方向性

### オプション1: @tabler/icons-reactに統一（推奨）

**メリット**:
- Mantineとの統合が良い
- テーマカラーに自動適応
- サイズ・色のカスタマイズが容易
- プロフェッショナルな見た目
- アクセシビリティ対応（aria-label等）

**デメリット**:
- 移行コストが高い（26ファイル修正）
- バンドルサイズが若干増加
- カジュアルさが失われる可能性

**変更例**:
```tsx
// Before
<NavbarIcon icon="💬" label="Sessions" />

// After
import { IconMessage } from '@tabler/icons-react';
<NavbarIcon icon={<IconMessage />} label="Sessions" />
```

### オプション2: 絵文字に統一

**メリット**:
- 移行コストが低い（15ファイル修正）
- バンドルサイズが小さい
- カジュアルで親しみやすい
- 追加の依存関係なし

**デメリット**:
- OS/ブラウザによる見た目の差異
- テーマカラーとの統合が困難
- サイズ調整が難しい
- プロフェッショナルさに欠ける

### オプション3: ハイブリッド（現状維持だが整理）

**方針**:
- **UI要素（ボタン、アイコンバー）**: @tabler/icons-react
- **テキスト装飾（メッセージ、通知）**: 絵文字

**メリット**:
- 各システムの強みを活かせる
- 移行コストが中程度
- UI部分だけプロフェッショナルに

**デメリット**:
- 明確なルールが必要
- 境界線が曖昧になる可能性

---

## 🎯 推奨アクション

### 段階的移行プラン

#### Phase 1: Navbarの統一（優先度: 高）
**理由**: ユーザーが最も目にする箇所、ペルソナアイコンのミスリード解消

**対象ファイル**:
1. `src/components/navigation/Navbar.tsx`
2. `src/components/navigation/NavbarIcon.tsx`

**変更内容**:
| 機能 | 現在 | 変更後 | アイコン名 |
|------|------|--------|-----------|
| Sessions | 💬 | 💬 or SVG | `IconMessage` / `IconMessages` |
| Workspace | 📁 | 📁 or SVG | `IconFolder` |
| Tasks | ✅ | ✅ or SVG | `IconCheckbox` / `IconChecklist` |
| **Personas** | ⭐️ | 🎭 / 😊 or SVG | `IconMask` / `IconMoodSmile` / `IconUsers` |
| Slash Commands | ⚡ | ⚡ or SVG | `IconBolt` / `IconCommand` |
| Dialogue Presets | 🎨 | 🎨 or SVG | `IconPalette` |

**特に重要**: Personasアイコンを⭐️から🎭（仮面）または`IconMask`/`IconUsers`に変更

#### Phase 2: システムメッセージの整理（優先度: 中）
**方針**: 絵文字をそのまま維持、またはSVGに統一するか決定

#### Phase 3: 全体統一（優先度: 低）
**方針**: Phase 1, 2の結果を踏まえて、全体の統一方針を決定

---

## 🛠️ 実装上の注意点

### NavbarIcon コンポーネントの変更

現在の`NavbarIcon`は`icon`プロパティとして`string`（絵文字）を受け取っています。

```typescript
// 現在の型定義: src/components/navigation/NavbarIcon.tsx:4
interface NavbarIconProps {
  icon: string;  // 絵文字文字列
  label: string;
  active: boolean;
  onClick: () => void;
  badge?: number;
}
```

**オプション1: Union型で両方サポート**
```typescript
interface NavbarIconProps {
  icon: string | React.ReactNode;  // 絵文字 or SVGコンポーネント
  // ...
}
```

**オプション2: 完全にSVGに移行**
```typescript
interface NavbarIconProps {
  icon: React.ReactNode;  // SVGコンポーネントのみ
  // ...
}
```

---

## 📝 次のステップ

1. **決定事項**:
   - [ ] 統一方針を決定（オプション1, 2, 3のどれか）
   - [ ] Personasアイコンの変更先を決定（🎭 / 😊 / SVG）

2. **実装**:
   - [ ] NavbarIcon コンポーネントの修正
   - [ ] Navbar.tsx のアイコン変更
   - [ ] 視覚的な確認とテスト

3. **展開**:
   - [ ] 他の箇所も同様に統一（必要に応じて）

---

## 📚 参考情報

- **@tabler/icons-react**: https://tabler.io/docs/icons/react
- **利用可能なアイコン数**: 5,000+
- **現在のバージョン**: 3.35.0 (package.json:19)

### ペルソナ関連の候補アイコン

@tabler/icons-reactから候補を抽出：
- `IconMask` - 仮面（演劇、ペルソナの象徴）
- `IconMoodSmile` - スマイル（人格、表情）
- `IconUsers` - 複数ユーザー（複数ペルソナを示唆）
- `IconUser` - 単一ユーザー
- `IconUserCircle` - ユーザーサークル
- `IconIdBadge` - IDバッジ（アイデンティティ）

**推奨**: `IconMask` または `IconUsers`
- `IconMask`: ペルソナの概念（仮面・役割）を直感的に表現
- `IconUsers`: 複数のペルソナ/AI参加者を示唆

---

## 結論

**即座に対応すべき項目**:
1. ✅ **Personasアイコンを⭐️から変更** - ミスリード防止のため最優先
2. 🔧 NavbarIcon全体の統一を検討

**長期的な改善**:
- 全体的なアイコンシステムの統一方針を確立
- デザインガイドラインの作成
