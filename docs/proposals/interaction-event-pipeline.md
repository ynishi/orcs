# Interaction/Event Pipeline Refactor

## 背景

- 現状は `orcs-desktop/src/App.tsx` から `invoke('handle_input')` を直接呼び出し、SlashCommand の結果テキストもそのまま backend へ送っている。
- SystemMessage の多くは UI 内で `handleSystemMessage()` によって描画され、永続化が必要なときのみ個別に `append_system_messages` を呼ぶ。  
  → backend 側の `InteractionManager` には「何が会話履歴に残ったのか」が部分的にしか伝わらない。
- `InteractionManager::handle_input_with_streaming` (`crates/orcs-interaction/src/lib.rs`) が必ず実行されるため、Shell SlashCommand のようにフロントだけで完結させたいケースでも Dialogue/Agent が起動してしまう。
- `llm_toolkit::agent::dialogue` には `ExecutionModel` 等の戦略はあるが、任意の Reaction/Hook を注入する仕組みがないため、Conversation への加工や検証を追加したい場合に Application 側で別レイヤを挟む必要がある。

## 調査結果の要点

| 観点 | 現状 | 課題 |
| --- | --- | --- |
| チャネル | UI → `handle_input` の単一路線 | SlashCommand のような system-only イベントも agent に送信される |
| SystemMessage | UI 任意、永続化は `append_system_messages` 呼び出し有無しだい | backend から見た履歴が不完全。再起動/他クライアントでの整合性が取れない |
| Dialogue | `InteractionManager` が `Dialogue::partial_session` を作り streaming | Reaction 等の挙動を注入するポイントが存在しない |
| 将来 API | Tauri IPC + callback に依存 | 完全非同期 API (gRPC/WebSocket 等) に発展させにくい |

## 目指すアーキテクチャ

1. **SessionEvent の一本化**
   - UI は `invoke('publish_session_event', enum SessionEvent)` を呼び、UserInput/SystemEvent/CommandResult などの種別を backend に委ねる。
   - `InteractionManager` が event を解釈し、UserInput なら Dialogue 実行、SystemEvent なら `system_messages` へ保存、CommandResult は必要に応じて Dialogue に投入。
   - SlashCommand Shell のように agent へ送らないケースは `shouldSendToAgent = false` を返すのではなく、event の時点で Dialogue を呼ばない。

2. **SystemMessage の backend 集約**
   - 新たに `publish_system_event` Tauri コマンドを追加し、React 側は表示のみを担当。
   - `InteractionManager::add_system_conversation_message` を単一入口とし、`include_in_dialogue` や `system_message_type` を統一管理。
   - UI は `dialogue-turn` とは別に `system-event` IPC を購読し、同一ソースから描画と履歴同期を行う。

3. **Reaction Strategy Hook**
   - `llm_toolkit::agent::dialogue::Dialogue` に `with_reaction_handler(handler)` のような API を追加し、各 `DialogueTurn` を UI へ渡す前/永続化前に任意処理を挟めるようにする。
   - Handler では、例えば自動要約・評価・メタデータタグ付けなどを行い、その結果を `ConversationMessage.metadata` に格納可能。
   - Application 層は Reaction の差し替えのみで挙動を変更でき、フロント側は単純な consumer になる。

4. **非同期ストリーム基盤**
   - `InteractionManager` 内部で `tokio::sync::broadcast::Sender<DialogueMessage>` を持ち、`handle_input_with_streaming` は購読専用の helper とする。
   - Tauri コマンドは broadcast を listen するだけなので、将来的に HTTP/WebSocket/gRPC で同じチャネルを expose できる。
   - SystemEvent 用にも同様のチャネルを用意すれば、UI はサーバー主導の push を受ける構造に移行できる。

## 遷移ステップ案

1. 最初に `SessionEvent` enum と `publish_session_event` Command を実装し、SlashCommand pipeline を event 化する。
2. `handleSystemMessage` 呼び出し箇所は `publish_system_event` に置き換え、UI は IPC push を描画するだけにする。
3. `llm_toolkit` に Reaction Hook を追加し、`InteractionManager` で差し込めるようにする。
4. Tauri ハンドラを broadcast ベースに載せ替え、段階的に既存 UI をサブスクライバに移行。  
   → 完了すればバックエンドは完全非同期 API として切り出せる。

## 判断と期待効果

- **SystemMessage を backend 管理に寄せる**: ダブルソース (UI/バックエンド) を解消し、セッション永続化・Agent コンテキストを一貫化できる。  
  フロント再描画/別クライアント接続でも同じ履歴を即時再生可能。
- **SessionEvent 経由の入力一本化**: SlashCommand など UI 側の分岐が不要になり、必要に応じて backend が agent 実行をスキップできる。  
  ReactionStrategy など上位ロジックが介入しやすくなる。
- **非同期ストリーム化**: 既存 Tauri ベースでも `broadcast` チャネルを介するだけなので段階的に導入でき、将来的に別プロセス/API へ切り出す際の軸になる。
- **Reaction Hook**: Agent 反応ごとに検証・補助処理を差し込む場所が生まれ、Engine 的な作り (自動評価・要約・次アクション提示など) を Dialogue 内で完結できる。

## リスク / オープン課題

- SessionEvent 導入時は既存の `invoke('handle_input')` 呼び出しを互換維持する必要があるため、移行期間は双方をサポートするラッパーが必要。
- SystemEvent push を UI へ届けるための IPC / broadcast 実装は、Tauri のライフタイム管理 (リスナー解除など) を考慮する。
- `llm_toolkit` への Reaction Hook 追加は API 変更を伴うため、別 crate としての互換性を慎重に扱う必要がある。

## 次のアクション

1. SessionEvent / publish_session_event の API スケッチを行い、SlashCommand 系入力を backend へ委譲する PoC を作成。
2. SystemEvent push (append + broadcast) を組み込み、UI 側表示を backend 主導で再生するサンプルを実装。
3. Dialogue Reaction Hook の API 仕様を `llm_toolkit` の ADR としてまとめ、プロトタイプを追加する。

## 将来的なキラーフィーチャー方向

1. **Reaction Layer による「儀式テンプレート」の標準化**  
   - Reaction Hook を使って「設計 → レビュー → PM 承認」などのワークフローを定義し、各 turn に対して自動的に評価や追記を行う。  
   - ペルソナの差し替えだけで同じオペレーティング・モデルを高速に複製でき、エンタープライズの“AI 運用ルール”をプロダクトとして提供できる。

2. **Support/Moderator ペルソナによる自動モデレーション**  
   - 既存の SupportAgent（セッションタイトル更新など）の機能を SessionEvent/Reaction と統合し、会話ログを監視しながら自動でモード切替・参加者の追加/削除・タスク化を行う。  
   - ユーザー操作なしで「必要なタイミングで必要なペルソナが登場する」体験を作り、完全自律的な Multi-Agent コラボレーションに近づける。

これらを同じ設計の延長に置くことで、マルチペルソナ・マルチクライアントを“運用可能な AI チーム”として差別化できる。

これらの実施後に、完全非同期 API への拡張ロードマップ (WebSocket/HTTP) を整理する。
