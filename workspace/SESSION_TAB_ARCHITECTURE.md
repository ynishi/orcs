# ORCS Desktop – Session Tab Architecture Plan

## 背景と課題
- 左ペインの `SessionList` でセッションを切り替えるたびにバックエンドへ `switch_session` を発行し、履歴を再ロードしているため UX が硬い。
- `useWorkspace()` / `useSessions()` を複数箇所で個別に呼んでおり、Tauri イベント (`workspace-switched`, `workspace-files-changed`) を受けてもビュー全体で状態が同期しない。
- 将来的に Multi Workspace／Expert Persona を同時進行させたいが、現在は「アクティブセッションは 1 つ」前提で UI/状態が組まれている。

## 目標
1. ブラウザのようにタブ単位でセッション状態を保持し、切り替え時にリロードを発生させない。
2. Tauri イベント購読を一元化し、Workspace や Session の状態が UI 全体で同期するようにする。
3. バックエンドの `SessionUseCase` / `WorkspaceManager` など 3 層構造は維持しつつ、将来的な CLI・マルチウィンドウ展開と整合的にする。

## フロントエンド設計

### Provider 再構成
- `WorkspaceProvider` と `SessionProvider` を新設し、`useWorkspace()` / `useSessions()` のロジックを移行。
- Provider 内で Tauri イベントを購読し、`refresh()` を一元化。StrictMode の二重購読を避けるため、登録フラグを持つ。
- Mantine / React Context / Zustand などを利用して、各コンポーネントが同じ state を参照できるようにする。

### タブ管理
- `openedTabs: Array<{ type: 'session'; id: string; title: string; isDirty: boolean }>` のような構造を Provider で管理。
- 右ペイン (`AppShell.Main`) を `<Tabs>` に変更。タブ操作（開く・閉じる・順序入れ替え）を Provider のアクションで制御。
- タブフォーカス時のみバックエンドへ `switch_session` を行い、非アクティブタブはローカル state を保持したままにする。
- タブクローズ時に未保存データの確認ダイアログなどを差し込めるよう、`isDirty` を管理。

### 左ペインとの連携
- `SessionList` から「タブを開く」操作を呼び出すよう変更（選択＝タブフォーカス、ダブルクリック＝新タブなど）。
- `WorkspacePanel` や `WorkspaceSwitcher` は Provider の state を直接参照し、`refresh()` を共有。
- `Navbar` のタブ切り替え（sessions / workspace / …）はそのまま残しつつ、Workspace 情報は Context 由来に置換。

## バックエンド連携方針

- 既存の `SessionUseCase` / `SessionManager` / `WorkspaceManager` 構成はそのまま利用する。
- タブ化後も「アクティブセッション」は 1 つのまま運用し、メッセージ送信など副作用のある操作の直前にのみ `switch_session` を呼ぶ。
- 新規で「セッションのスナップショットを取得する軽量 API (`get_session_snapshot`)」を追加すると、タブ初期化時にバックエンド状態を乱さず読み取れる。
- `AppStateService` は `active_session_id` / `last_selected_workspace_id` を記録する既存仕様を継続利用。必要に応じて `open_session_ids` 等を将来拡張する。

## 実装ステップ案
1. **Provider リファクタ**  
   - `useWorkspace()` / `useSessions()` を Context ベースに置き換え、イベント購読を一本化。
   - 左ペインの参照先を Context に変更しつつ、既存機能が壊れていないことを確認。
2. **タブビュー導入**  
   - `AppShell.Main` を Tabs に変更し、`openedTabs` の状態管理・タブ UI を実装。
   - Tab と Backend `switch_session` の連携を調整（フォーカス時のみ `switch_session`）。
3. **UX 調整**  
   - セッションタブの閉じるボタン、順序操作、未保存警告などを段階的に実装。
   - Workspace 切り替え時のタブ更新／自動クローズなどの挙動を整理。
4. **オプション (将来)**  
   - `get_session_snapshot` の追加、CLI 連携、マルチウィンドウ対応、タブ集合の永続化など。

## リスクと緩和策
- **状態の二重管理** → Provider 化で単一ソースにまとめ、Hook から Provider を参照する形にする。
- **StrictMode でのイベント重複** → リスナー登録フラグ＆クリーンアップを実装。
- **タブと Backend 状態の不整合** → タブフォーカス時のみ Backend を切り替えるガードを設け、UI 内での状態遷移はローカルに閉じる。
- **パフォーマンス** → 遅延ロード (`React.lazy` + Suspense) やデータキャッシュで初期レンダリングを抑え、タブが多くても軽量に保つ。

## 次アクション
- Provider 実装のブランチを切り、`App.tsx` / `Navbar` / `WorkspacePanel` / `SessionList` などで新 Context を利用するよう順次改修。
- タブ UI は別 PR とし、段階的に QA しながら導入する。


