# Default Workspace仕様

## 概要

セッションは必ず何らかのワークスペースに紐づくべきという設計意図を実現するため、`workspace_id`を必須フィールド化し、デフォルトワークスペース(`~/orcs`)の自動作成ロジックを実装した。

## 課題と背景

### 設計上の問題点

**Critical Issue**: `Session.workspace_id` が `Option<String>` になっており、ワークスペースなしでセッションが作成可能だった。

- ワークスペースが1つもない状態でアプリ起動 → `workspace_id: None` でセッションが作成される
- これは「すべてのセッションは必ず何らかのワークスペースに紐づく」という設計意図と矛盾

### 既存の未使用フィールド

`AppState.default_workspace_id` フィールドが既に存在していたが、実装されていなかった:
- `get_default_workspace()` / `set_default_workspace()` メソッドは実装済み
- しかし実際に設定・使用している箇所はゼロ
- 起動時ロジックは `last_selected_workspace_id` のみを参照

## ワークスペース概念の整理

ORCSには3つの異なる「workspace」概念が存在する:

### 1. User Workspace (Project Root)
- **役割**: Agentの作業ディレクトリ（ユーザーのプロジェクトルート）
- **例**: `~/my-project`, `~/orcs`
- **用途**: Agentがコードを読み書きする対象ディレクトリ
- **管理**: `Workspace.root_path` (canonicalized path)

### 2. Workspace Metadata Storage
- **役割**: ワークスペース設定の永続化
- **場所**: `ServiceType::Workspace` で管理
  - macOS: `~/Library/Application Support/orcs/data/workspaces/`
  - Linux: `~/.local/share/orcs/data/workspaces/`
  - Windows: `%LOCALAPPDATA%\orcs\data\workspaces\`
- **内容**: `workspace_<uuid>.toml` ファイル群
- **管理**: `AsyncDirWorkspaceRepository`

### 3. Workspace Data Storage
- **役割**: ワークスペースごとのアプリ管理ファイル
- **場所**: `ServiceType::WorkspaceStorage` で管理
  - macOS: `~/Library/Application Support/orcs/workspaces/<uuid>/`
- **内容**: アップロードされたファイル、一時ファイルなど
- **管理**: `WorkspaceStorageService`

## 設計判断

### 判断1: Default Workspace のパスはどこで決定するか

**選択肢:**
- **Option A**: Infrastructure層 (`OrcsPaths`) で決定 ✅ 採用
- Option B: Desktop層 (Tauri bootstrap) で決定

**判断理由:**
- クリーンアーキテクチャの原則: Infrastructure層がプラットフォーム依存の詳細を隠蔽
- 再利用性: 他のアプリケーション層（CLI、テストなど）からも利用可能
- 一貫性: 他のパス管理（Config, Sessions, Personasなど）と同じ `OrcsPaths` で管理

**実装:**
```rust
// crates/orcs-infrastructure/src/paths.rs
impl OrcsPaths {
    pub fn default_user_workspace_path(&self) -> Result<PathBuf, PathError> {
        let home = dirs::home_dir().ok_or(PathError::HomeDirNotFound)?;
        Ok(home.join("orcs"))
    }
}
```

### 判断2: Default Workspace ID の初期化タイミング

**選択肢:**
- **Option A**: アプリ起動時に自動生成・保存 (Eager Initialization) ✅ 採用
- Option B: 最初のセッション作成時に生成 (Lazy Initialization)

**判断理由:**
- 確実性: アプリ起動時に必ず初期化されることを保証
- セッション作成の単純化: セッション作成時に「デフォルトworkspace_idの有無」を考慮不要
- 起動時コストの許容: ディレクトリ作成とWorkspace登録は軽量な処理

**実装フロー:**
```
bootstrap.rs:
1. AppStateService初期化
2. WorkspaceManager初期化
3. ensure_default_workspace() ← ~/orcs を作成・登録、IDを取得
4. AppState.default_workspace_id に保存
5. SessionManager初期化
6. replace_placeholder_sessions() ← 既存セッションのプレースホルダー置換
```

### 判断3: マイグレーション戦略（None → String）

**選択肢:**
- **Option A**: プレースホルダーパターン（段階的変換） ✅ 採用
- Option B: 既存セッション削除
- Option C: マイグレーション時に実Workspace ID生成

**判断理由:**
- データ保持: ユーザーの既存セッションを削除しない
- 責務分離: マイグレーション関数は純粋な型変換のみ（外部依存なし）
- 安全性: Bootstrap時の単一箇所で一括置換するため、置換漏れのリスクゼロ

**実装パターン:**
```rust
// Migration (DTO layer)
impl MigratesTo<SessionV3_0_0> for SessionV2_9_0 {
    fn migrate(self) -> SessionV3_0_0 {
        SessionV3_0_0 {
            workspace_id: self.workspace_id
                .unwrap_or_else(|| PLACEHOLDER_WORKSPACE_ID.to_string()),
            // ...
        }
    }
}

// Bootstrap (Application layer)
async fn replace_placeholder_sessions(
    session_repository: &Arc<AsyncDirSessionRepository>,
    default_workspace_id: &str,
) -> Result<()> {
    for mut session in session_repository.list_all().await? {
        if session.workspace_id == PLACEHOLDER_WORKSPACE_ID {
            session.workspace_id = default_workspace_id.to_string();
            session_repository.save(&session).await?;
        }
    }
}
```

### 判断4: Workspace ID の生成方法

**既存実装を踏襲:**
```rust
// WorkspaceStorageService::get_workspace_id()
fn get_workspace_id(canonical_path: &Path) -> Result<String> {
    let path_str = canonical_path.to_string_lossy();
    let namespace = Uuid::NAMESPACE_DNS;
    let uuid = Uuid::new_v5(&namespace, path_str.as_bytes());
    Ok(uuid.to_string())
}
```

**特性:**
- **Deterministic**: 同じパスからは必ず同じIDが生成される
- **Idempotent**: 複数回呼んでも安全（既存チェック → 作成 のロジック）
- **Uniqueness**: 異なるパスからは異なるIDが生成される

## 型システムによる保証

### Phase 1: AppState必須化
```rust
// Before
pub struct AppState {
    pub default_workspace_id: Option<String>,  // 任意
}

// After
pub struct AppState {
    pub default_workspace_id: String,  // 必須
}
```

**効果:**
- `get_default_workspace()` が `Option<String>` → `String` に変更
- 呼び出し側でのNoneチェックが不要に

### Phase 2: Session必須化
```rust
// Before
pub struct Session {
    pub workspace_id: Option<String>,  // 任意
}

// After
pub struct Session {
    pub workspace_id: String,  // 必須
}
```

**効果:**
- セッション作成時に必ずworkspace_idを指定する必要がある（コンパイルエラーで強制）
- Noneの分岐処理が削減される

## 影響範囲

### Core層
- `session/model.rs`: `workspace_id` 型変更
- `state/model.rs`: `default_workspace_id` 型変更
- `state/repository.rs`: trait署名変更

### Infrastructure層
- `dto/app_state.rs`: AppStateV1_3追加、マイグレーション実装
- `dto/session.rs`: SessionV3_0_0追加、マイグレーション実装
- `paths.rs`: `default_user_workspace_path()` 追加

### Application層
- `session_usecase.rs`: Option処理削除、プレースホルダー使用

### Desktop層
- `app/bootstrap.rs`:
  - `ensure_default_workspace()` 実装
  - `replace_placeholder_sessions()` 実装
  - 起動フロー統合
- 各種Tauri commands: Option処理削除

## マイグレーションパス

### AppState
```
V1.0 (initial)
  → V1.1 (+default_workspace_id: Option)
  → V1.2 (+active_session_id: Option)
  → V1.3 (default_workspace_id: String) ← 今回追加
```

### Session
```
V1.0 → ... → V2.9.0 (workspace_id: Option<String>)
  → V3.0.0 (workspace_id: String) ← 今回追加
```

## 検証項目

- ✅ 初回起動時に ~/orcs ディレクトリが作成される
- ✅ AppState に default_workspace_id が保存される
- ✅ 既存セッションのプレースホルダーが実IDに置換される
- ✅ 新規セッション作成時に必ず workspace_id が設定される
- ✅ workspace_id が None のケースが型システムで排除される
- ✅ コンパイルエラーなし
- ✅ UIからDefault Workspaceが確認可能（設定メニュー）

## 将来の拡張性

この設計により、以下が可能になる:

1. **複数のデフォルトワークスペース**:
   - 現在は `~/orcs` 固定だが、`OrcsPaths::default_user_workspace_path()` を修正するだけで変更可能

2. **ワークスペーステンプレート**:
   - デフォルトワークスペースに初期ファイル（README, .gitignoreなど）を配置

3. **Workspace required セマンティクス**:
   - 型システムで「すべてのセッションは必ずワークスペースを持つ」が保証される
   - 将来的に workspace_id に基づく機能追加が安全に行える
