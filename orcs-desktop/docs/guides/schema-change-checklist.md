# Schema/Domain変更時のチェックリスト

## 概要

Domain/Schemaを変更した際に、影響範囲を洗い出し、対応漏れを防ぐための実践的ガイド。

**典型的な失敗パターン**:
- Domainモデルを変更してコミット
- フロントエンドの型定義やUIコンポーネントの対応を忘れる
- 実行時にundefined/nullが発生

## 🎯 基本フロー

### 1. 変更の特定
まず何を変更したか明確にする：
```
例: Session domain
- フィールド名変更: `name` → `title`
- 場所: crates/orcs-core/src/session.rs
```

### 2. 影響範囲の洗い出し

#### A. Rust側の影響範囲

**必須チェック箇所**:
```bash
# 1. Domain定義
crates/orcs-core/src/session.rs

# 2. Infrastructure層（DTO & Migration）
crates/orcs-infrastructure/src/dto.rs         # SessionV1の更新
crates/orcs-infrastructure/src/migration.rs   # From trait実装の更新
crates/orcs-infrastructure/src/repository.rs

# 3. SessionManager（ビジネスロジック）
crates/orcs-core/src/session_manager.rs

# 4. Tauriコマンド（API層）
orcs-desktop/src-tauri/src/main.rs

# 5. テストコード
**/*_test.rs, **/tests/*.rs
```

**🔑 重要: DTOバージョニング（Semantic Versioning）**

このプロジェクトでは**DTO層でSemantic Versioning**を採用：

#### バージョン番号の付け方

| 変更内容 | バージョン | 対応方法 | 例 |
|---------|-----------|---------|-----|
| **破壊的変更** | MAJOR (X.0.0) | 新しいDTO構造体を作成（V2, V3...） | フィールド削除、型変更 |
| **後方互換の追加** | MINOR (1.X.0) | `Option<T>` で既存DTOに追加 | 新フィールド追加 |
| **バグ修正** | PATCH (1.0.X) | 既存DTOをそのまま修正 | ドキュメント修正等 |

#### 現在のバージョン履歴

```rust
// crates/orcs-infrastructure/src/dto.rs
pub const SESSION_V1_VERSION: &str = "1.1.0";

// V1.0.0: Initial schema (title field)
// V1.1.0: Added optional created_at field
```

**重要な原則**:
- ✅ **追加だけなら V1 のまま `Option<T>` で対応**（V2を作らない）
- ✅ **削除・型変更なら V2 を作成**
- ✅ **保存時は常に最新バージョン番号**（`SESSION_V1_VERSION`）
- ✅ **読込時は古いバージョンも対応**（後方互換性）

**検索方法**:
```bash
# 旧フィールド名で検索
rg "\.name" --type rust

# 構造体初期化を検索
rg "Session \{" --type rust -A 10
```

#### B. TypeScript側の影響範囲

**必須チェック箇所**:
```bash
# 1. 型定義
src/types/session.ts

# 2. Hooks（API呼び出し）
src/hooks/useSessions.ts

# 3. UIコンポーネント（表示・入力）
src/components/sessions/*.tsx

# 4. その他の参照箇所
**/*.tsx, **/*.ts
```

**検索方法**:
```bash
# 旧フィールド名で検索
rg "\.name" --type ts --type tsx

# sessionオブジェクトの使用箇所
rg "session\." --type ts --type tsx
```

### 3. 修正の実施順序

**重要**: バックエンド→フロントエンドの順で修正

#### Phase 1: Rust Domain層
1. **Domain model** (`crates/orcs-core/src/session.rs`)
   ```rust
   pub struct Session {
       pub id: String,
       pub title: String,  // name → title に変更
       // ...
   }
   ```

2. **DTOバージョン判定** - 変更内容を確認
   - **フィールド追加のみ** → `Option<T>` で V1.X.0 に
   - **破壊的変更** → V2 を新規作成

3. **DTO** (`crates/orcs-infrastructure/src/dto.rs`)

   **追加の場合（V1.X.0）**:
   ```rust
   pub const SESSION_V1_VERSION: &str = "1.1.0";  // バージョン更新

   pub struct SessionV1 {
       pub schema_version: String,
       pub id: String,
       pub title: String,

       // 🆕 新フィールドは Option で追加
       #[serde(default)]
       pub created_at: Option<String>,  // V1.1.0で追加
       // ...
   }
   ```

   **破壊的変更の場合（V2.0.0）**:
   ```rust
   pub const SESSION_V2_VERSION: &str = "2.0.0";

   pub struct SessionV2 {
       pub schema_version: String,
       // 完全に新しい構造
   }
   ```

4. **Migration** (`crates/orcs-infrastructure/src/migration.rs`)

   **SessionV1 → Domain（読込時）**:
   ```rust
   use semver::Version;
   use crate::dto::{SessionV1, SESSION_V1_VERSION};

   impl From<SessionV1> for Session {
       fn from(dto: SessionV1) -> Self {
           // バージョンをパースして将来的な分岐に対応
           let _version = Version::parse(&dto.schema_version)
               .unwrap_or_else(|_| Version::new(1, 0, 0));

           Session {
               id: dto.id,
               title: dto.title,

               // 🔑 Option<T> のハンドリング
               // V1.0.0（created_atがNone）→ updated_atで代用
               // V1.1.0（created_atがSome）→ そのまま使用
               created_at: dto.created_at
                   .unwrap_or_else(|| dto.updated_at.clone()),
               // ...
           }
       }
   }
   ```

   **Domain → SessionV1（保存時）**:
   ```rust
   impl From<&Session> for SessionV1 {
       fn from(session: &Session) -> Self {
           SessionV1 {
               // 🔑 常に最新バージョンで保存
               schema_version: SESSION_V1_VERSION.to_string(),
               id: session.id.clone(),
               title: session.title.clone(),

               // 🔑 新規保存時は必ず Some で保存
               created_at: Some(session.created_at.clone()),
               // ...
           }
       }
   }
   ```

4. **Repository実装** (`repository.rs`) - テストコード内の初期化
5. **Tests**

#### Phase 2: Rust Application層
6. SessionManager
7. Tauri commands (`main.rs`)

#### Phase 3: TypeScript層
8. 型定義 (`types/session.ts`)
9. Hooks (`useSessions.ts`)
10. Components (`SessionList.tsx` など)

### 4. 検証方法

```bash
# Rust
cargo check
cargo test

# TypeScript
cd orcs-desktop
npx tsc --noEmit

# 実行確認
npm run dev
```

## 📋 チェックリストテンプレート

新しいSchema変更時にコピーして使用：

```markdown
## Schema変更: [モデル名] - [変更内容]

### 🔢 バージョニング判定
- [ ] **変更種類を確認**
  - [ ] フィールド追加のみ → V1.X.0 に（`Option<T>` 使用）
  - [ ] 破壊的変更（削除・型変更） → V2.0.0 を作成

### Rust側
- [ ] **Domain定義を変更** (`crates/orcs-core/src/session.rs`)
  - [ ] フィールド名/型を変更
- [ ] **DTOバージョンを更新** (`crates/orcs-infrastructure/src/dto.rs`)
  - [ ] `SESSION_V1_VERSION` を更新（例: "1.1.0" → "1.2.0"）
  - [ ] 新フィールドは `#[serde(default)]` + `Option<T>` で追加
  - [ ] ドキュメントにバージョン履歴を追記
- [ ] **Migrationを更新** (`crates/orcs-infrastructure/src/migration.rs`)
  - [ ] `From<SessionV1> for Session` を更新
    - [ ] `Option<T>` のデフォルト値ハンドリングを追加
  - [ ] `From<&Session> for SessionV1` を更新
    - [ ] 新フィールドを `Some(...)` で保存
- [ ] **Repository実装を確認** (`repository.rs`)
  - [ ] テストコード内の構造体初期化を修正
- [ ] **SessionManager等のビジネスロジックを確認**
- [ ] **Tauriコマンドを確認・修正** (`src-tauri/src/main.rs`)
- [ ] **テストを修正・追加**
- [ ] `cargo check` が通る
- [ ] `cargo test` が通る

### TypeScript側
- [ ] 型定義を変更 (`src/types/*.ts`)
- [ ] Hooksを確認・修正 (`src/hooks/*.ts`)
- [ ] Componentsを確認・修正 (`src/components/**/*.tsx`)
- [ ] `npx tsc --noEmit` が通る
- [ ] 実際の動作確認（dev環境で）

### 影響範囲の洗い出し
- [ ] `rg "旧名"` で検索実施
- [ ] 構造体初期化箇所を全て確認
- [ ] 見落としがないか再確認
```

## 🔍 今回のケース: Session - V1.0.0 → V1.1.0

### バージョン変更内容

**V1.0.0 → V1.1.0**:
- **変更種類**: 後方互換のフィールド追加（MINOR）
- **追加フィールド**: `created_at: Option<String>`
- **理由**: セッション作成日時を保持してソート順を正しくするため
- **後方互換性**: V1.0.0のファイルでは`created_at`が`None`、`updated_at`で代用

### 修正箇所一覧

**Rust - Infrastructure層**:
- ✅ `crates/orcs-core/src/session.rs` - Sessionフィールド追加
  ```rust
  pub struct Session {
      pub created_at: String,  // 🆕 追加
      // ...
  }
  ```
- ✅ `crates/orcs-infrastructure/src/dto.rs` - SessionV1バージョン更新
  ```rust
  pub const SESSION_V1_VERSION: &str = "1.1.0";  // "1" → "1.1.0"

  pub struct SessionV1 {
      #[serde(default)]
      pub created_at: Option<String>,  // 🆕 Option で追加
      // ...
  }
  ```
- ✅ `crates/orcs-infrastructure/src/migration.rs` - From trait実装の対応
  ```rust
  // From<SessionV1> for Session (読込時)
  created_at: dto.created_at
      .unwrap_or_else(|| dto.updated_at.clone()),  // 🔑 None時の代用

  // From<&Session> for SessionV1 (保存時)
  schema_version: SESSION_V1_VERSION.to_string(),  // 🔑 "1.1.0"
  created_at: Some(session.created_at.clone()),    // 🔑 必ずSome
  ```
- ✅ `crates/orcs-infrastructure/src/repository.rs` - テストコード内の初期化

**Rust - Application層**:
- ✅ `crates/orcs-interaction/src/lib.rs` - InteractionManagerの修正
  ```rust
  pub struct InteractionManager {
      title: Arc<RwLock<String>>,      // 🆕 titleフィールド追加
      created_at: String,               // 🆕 created_atフィールド追加
      // ...
  }

  pub async fn to_session(&self, app_mode: AppMode) -> Session {
      Session {
          created_at: self.created_at.clone(),        // 🔑 保持
          updated_at: chrono::Utc::now().to_rfc3339(), // 🔑 毎回更新
          // ...
      }
  }
  ```
- ✅ `crates/orcs-core/src/session_manager.rs` - rename_sessionメソッド追加
- ✅ `orcs-desktop/src-tauri/src/main.rs` - Tauriコマンド `rename_session` 追加

**TypeScript**:
- ✅ `src/types/session.ts` - Session interface `name` → `title`
- ✅ `src/components/sessions/SessionList.tsx` - 表示・編集
  ```typescript
  // 表示
  {session.title}  // session.name から変更

  // 編集開始
  setEditingTitle(session.title);  // session.name から変更
  ```
- ✅ `src/hooks/useSessions.ts` - renameSession実装完了

### 見落としやすいポイント

1. **DTOバージョン番号の更新忘れ**
   - `SESSION_V1_VERSION` 定数の更新を忘れる
   - 保存時に古いバージョン番号が書き込まれる

2. **Option<T>のデフォルト値ハンドリング**
   - `#[serde(default)]` を付け忘れると既存ファイルが読めない
   - `unwrap_or_else()` でのフォールバック処理を忘れる

3. **InteractionManagerのフィールド保持**
   - `to_session()`で毎回生成するとタイムスタンプが壊れる
   - `created_at`や`title`は構造体フィールドとして保持する

4. **テストコード内の構造体初期化**
   - 本体は変更したがテストは変更し忘れ

5. **UI表示だけでなく編集機能も**
   - `session.name` の表示だけでなく
   - `setEditingTitle(session.name)` など編集系も

6. **ローカルState更新**
   - `setSessions(prev => prev.map(s => {...s, name: newName}))`
   - フィールド名変更に追従

## 💡 自動化のヒント

### エディタ設定
- TypeScript: strict mode有効化
- Rust: clippy有効化

### CI/CD
```yaml
# 例: GitHub Actions
- run: cargo check
- run: cargo test
- run: cd orcs-desktop && npx tsc --noEmit
```

### Pre-commit hook
```bash
#!/bin/bash
cargo check || exit 1
cd orcs-desktop && npx tsc --noEmit || exit 1
```

## 📚 参考

### Semantic Versioning
- 公式サイト: https://semver.org/
- Rustクレート: `semver = { version = "1.0", features = ["serde"] }`

### アーキテクチャ原則
- **Clean Architecture**: Domain → Application → Infrastructure → Presentation の順で修正
- **DTOでバージョニング**: Infrastructure層でスキーマ進化を管理
- **後方互換性**: `Option<T>` + `#[serde(default)]` で対応
