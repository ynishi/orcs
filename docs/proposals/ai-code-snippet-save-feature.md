# AI Code Snippet Save Feature - Technical Proposal

## 概要

AIが返答に含めたコードスニペットやファイルを、ユーザーが承認して直接保存できる機能の実装提案。

**例：** Aiちゃんが `ai_architect.toml` ファイルの内容を提示した際、ユーザーが「保存する？ <YES>」ボタンをクリックするだけでファイルが保存される。

## 背景

現在、AIペルソナが以下のような有用な情報を提供する際：
- 設定ファイル（TOML/JSON/YAML等）
- コードスニペット
- シェルスクリプト
- ドキュメント

ユーザーは手動でコピー&ペーストしてファイルを作成する必要があり、UX上の摩擦が大きい。

## 実装アプローチの検討

### アプローチ1: Rich UI with Interactive Buttons（推奨）

**概要:**
- Markdown のコードブロックに特別なメタデータを埋め込む
- フロントエンドでパースして、保存ボタン付きのリッチUIとして表示
- ユーザーがボタンをクリックして承認すると、Tauriコマンド経由でファイル保存

**実装詳細:**

1. **System Prompt への指示追加**
   ```markdown
   When providing file content, use this format:

   \`\`\`toml:path=/path/to/file.toml:saveable
   # file content here
   \`\`\`

   - Add `:saveable` to indicate this can be saved
   - Add `:path=/actual/path` to specify destination
   ```

2. **Markdown パーサー拡張**
   - コードブロックのメタデータを解析
   - `saveable` フラグと `path` を抽出

3. **UI コンポーネント作成**
   ```tsx
   interface SaveableCodeBlockProps {
     language: string;
     code: string;
     suggestedPath?: string;
     onSave: (path: string, content: string) => Promise<void>;
   }

   function SaveableCodeBlock({ code, suggestedPath, onSave }: SaveableCodeBlockProps) {
     const [isEditingPath, setIsEditingPath] = useState(false);
     const [targetPath, setTargetPath] = useState(suggestedPath);

     return (
       <Paper>
         <Group justify="space-between">
           <Text size="sm" c="dimmed">
             {targetPath || 'No path specified'}
           </Text>
           <Button
             size="xs"
             onClick={() => onSave(targetPath, code)}
           >
             Save to file
           </Button>
         </Group>
         <CodeHighlight code={code} />
       </Paper>
     );
   }
   ```

4. **Backend Tauri Command**
   ```rust
   #[tauri::command]
   async fn save_code_snippet(
       file_path: String,
       content: String,
       workspace_id: Option<String>,
       state: State<'_, AppState>,
   ) -> Result<(), String> {
       use std::fs;
       use std::path::Path;

       let path = Path::new(&file_path);

       // パス検証
       if !path.is_absolute() {
           return Err("Path must be absolute".to_string());
       }

       // 親ディレクトリが存在するか確認
       if let Some(parent) = path.parent() {
           fs::create_dir_all(parent)
               .map_err(|e| format!("Failed to create directory: {}", e))?;
       }

       // ファイル書き込み
       fs::write(path, content)
           .map_err(|e| format!("Failed to write file: {}", e))?;

       // システムメッセージとして記録
       let manager = state.session_manager
           .active_session()
           .await
           .ok_or("No active session")?;

       manager.add_system_conversation_message(
           format!("Saved file: {}", file_path),
           Some("file_save".to_string()),
           None,
       ).await;

       Ok(())
   }
   ```

**メリット:**
- ✅ 既存アーキテクチャに自然に統合
- ✅ ユーザーがパスを編集可能
- ✅ プレビュー→保存の明確なフロー
- ✅ 段階的実装が可能

**デメリット:**
- ⚠️ System Prompt のメンテナンスが必要
- ⚠️ AIが指示に従わない可能性

---

### アプローチ2: Agent-to-Agent (A2A) Protocol

**概要:**
- ORCSコマンド実行プロトコルを定義
- AIが `@orcs save-file` のような特殊コマンドを発行
- バックエンドがコマンドを解釈して実行

**実装詳細:**

1. **Protocol 定義**
   ```typescript
   interface OrcsCommand {
     type: 'save_file' | 'execute_shell' | 'create_persona';
     payload: {
       path?: string;
       content?: string;
       command?: string;
     };
     requiresApproval: boolean;
   }
   ```

2. **AI 側の発行形式**
   ```markdown
   @orcs {
     "type": "save_file",
     "payload": {
       "path": "/path/to/file.toml",
       "content": "# file content..."
     },
     "requiresApproval": true
   }
   ```

3. **Parser & Executor**
   ```rust
   struct OrcsCommandParser;

   impl OrcsCommandParser {
       fn parse(text: &str) -> Option<OrcsCommand> {
           // Parse @orcs JSON blocks
       }

       async fn execute(cmd: OrcsCommand) -> Result<(), String> {
           match cmd.type {
               CommandType::SaveFile => {
                   // Execute file save
               }
               // ...
           }
       }
   }
   ```

**メリット:**
- ✅ 拡張性が高い（他のコマンドも追加可能）
- ✅ 型安全なプロトコル
- ✅ AI の意図が明確

**デメリット:**
- ⚠️ 大規模な実装が必要
- ⚠️ AIがプロトコルを正確に守る必要がある
- ⚠️ 影響範囲が広い

---

### アプローチ3: MCP (Model Context Protocol) Integration

**概要:**
- Anthropic の MCP を利用
- AIが `use_mcp_tool` で `save_file` ツールを呼び出し
- ORCS がツールハンドラーとして動作

**実装詳細:**

1. **MCP Server Implementation**
   ```rust
   struct OrcsMcpServer {
       workspace_manager: Arc<FileSystemWorkspaceManager>,
   }

   impl McpToolProvider for OrcsMcpServer {
       fn list_tools(&self) -> Vec<McpTool> {
           vec![
               McpTool {
                   name: "save_file",
                   description: "Save content to a file",
                   input_schema: json!({
                       "type": "object",
                       "properties": {
                           "path": { "type": "string" },
                           "content": { "type": "string" }
                       }
                   }),
               }
           ]
       }

       async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
           match name {
               "save_file" => self.save_file(args).await,
               _ => Err("Unknown tool".into())
           }
       }
   }
   ```

**メリット:**
- ✅ 標準プロトコル
- ✅ 将来的な拡張性
- ✅ Claude の native サポート

**デメリット:**
- ⚠️ MCP インフラの実装が必要
- ⚠️ ユーザー承認フローの設計が複雑
- ⚠️ 現時点では over-engineering の可能性

---

## 推奨アプローチ

**アプローチ1: Rich UI with Interactive Buttons** を推奨します。

理由:
1. **段階的実装が可能** - 最小限の変更から始められる
2. **既存アーキテクチャとの親和性** - Tauri コマンドパターンに沿う
3. **ユーザー体験の向上** - 視覚的にわかりやすく、承認フローも自然
4. **リスクが低い** - 既存機能への影響が最小限

## 実装フェーズ

### Phase 1: 基本実装 (1-2 days)
- [ ] Markdown コードブロックパーサー拡張
- [ ] `SaveableCodeBlock` コンポーネント作成
- [ ] `save_code_snippet` Tauri コマンド実装
- [ ] 基本的な UI テスト

### Phase 2: System Prompt 統合 (0.5 days)
- [ ] ペルソナ System Prompt にファイル保存指示を追加
- [ ] 実際の AI 応答でテスト
- [ ] ガイドライン文書作成

### Phase 3: UX 改善 (1 day)
- [ ] パス編集機能
- [ ] ファイル存在確認と上書き警告
- [ ] 保存成功/失敗のフィードバック
- [ ] 保存履歴の記録

### Phase 4: 拡張機能 (Optional)
- [ ] ディレクトリピッカー
- [ ] 複数ファイル一括保存
- [ ] Workspace との統合
- [ ] Git 自動コミット

## セキュリティ考慮事項

1. **パス検証**
   - 絶対パスのみ許可
   - サンドボックス外への書き込み防止
   - シンボリックリンク攻撃対策

2. **上書き確認**
   - 既存ファイルの上書き前に確認ダイアログ
   - バックアップ作成オプション

3. **実行権限**
   - スクリプトファイルの実行権限は自動付与しない
   - ユーザーが明示的に許可する必要がある

## 代替案の将来的検討

アプローチ1を実装後、以下の機能が必要になった場合は A2A Protocol や MCP への移行を検討：

- AIが複雑なワークフローを自動実行する必要がある
- 複数のツールを組み合わせた操作が頻繁に発生する
- 外部システムとの連携が必要になる

## 結論

Rich UI アプローチで最小限の実装から始め、ユーザーフィードバックを収集しながら段階的に機能を拡張することを推奨します。

---

**作成日:** 2025-11-05
**提案者:** Claude (via User Request)
**ステータス:** Draft / Under Review
