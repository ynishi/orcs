# AGENT.md - AI Development Assistant Guide

このファイルはAI開発アシスタントが読むための簡潔なチェックリストです。
詳細は `docs/guides/` 配下のガイドを参照してください。

## 🎯 基本原則

### 1. 実装前の確認事項
- [ ] **型の整合性**: フロントエンド（TypeScript）とバックエンド（Rust）の型定義が一致しているか確認
- [ ] **既存実装の確認**: 同様の機能が既に実装されていないか検索
- [ ] **Clean Architecture**: 変更がレイヤー構造を守っているか確認

### 2. Tauriプロジェクト特有の注意点
- [ ] **Tauri Command追加時**: `docs/guides/tauri-command-workflow.md` を参照
- [ ] **型不一致修正時**: `docs/guides/frontend-backend-sync.md` を参照
- [ ] **Session関連の変更**: バックエンド→フロントエンドの順で修正

### 3. 修正時の必須手順
1. **調査**: 関連ファイルを全て読み込む（推測しない）
2. **TodoList作成**: 複数ステップの作業は必ずTodoListで管理
3. **コンパイルチェック**: `cargo check` を実行
4. **型チェック**: `npx tsc --noEmit` を実行（orcs-desktop/で実行）
5. **完了確認**: 全Todoが完了してから次のタスクへ

## 📚 詳細ガイド参照

### フロントエンド・バックエンド連携
- **型同期・修正**: `docs/guides/frontend-backend-sync.md`
  - Rust ↔ TypeScript型の対応関係
  - 型不一致時の修正手順
  - Session/Persona等の共通型の扱い

### Tauri開発
- **コマンド追加**: `docs/guides/tauri-command-workflow.md`
  - Tauriコマンドの追加手順
  - invoke_handlerへの登録
  - フロントエンドからの呼び出し

### アーキテクチャ
- Clean Architecture原則に従う
- 詳細は各crateのREADME参照

## 🚨 よくあるミス

1. **型フィールド名の不一致**: `name` vs `title`
   - Rustの定義が正: TypeScriptを合わせる
   - 影響範囲: UI表示、hooks、型定義

2. **Tauriコマンドの登録忘れ**: `invoke_handler![]` に追加し忘れ

3. **State更新忘れ**: フロントエンドのローカルStateを更新し忘れ

## 🔄 定期的な確認

コミット前に必ず：
- [ ] `cargo check` が通る
- [ ] TypeScript型エラーがない
- [ ] Todoリストが全て完了している
- [ ] 不要なconsole.log/TODOコメントを削除
