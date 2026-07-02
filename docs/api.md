# SNSシステム APIサブシステム

## 概要

これは、SNSシステムのAPI開発用サブシステムです。

## 構成と主なファイルの各説明

- / ： リポジトリルート
  - /.devcontainer/api/devcontainer.json ： API開発で利用するdevcontainer
  - /.github/workflows/sns-system-api-cicd.yaml ： このプロジェクトのAPIのCI/CDのGitHub Actions定義
  - /README.md ： システム全体の概要説明
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /api ： APIのルート
    - /api/AGENTS.md ： AI向けプロンプトを記述
    - /api/template.yaml ： APIのIaC
    - /api/openapi.yaml ： API定義(`utoipa`から自動生成)。
    - /api/lambda ： Lambdaを実装するRustパッケージ
      - /api/lambda/Cargo.toml ： パッケージのマニフェストファイル。
      - /api/lambda/src/*.rs ： Rustプログラム実装
      - /api/lambda/src/bin/export_openapi.rs ： `openapi.yaml`生成関数
  - /core ： ビジネスロジックの実装のルート
  - /db ： DBのルート
  - /docs ： ドキュメントのルート
    - /docs/api.md ： 人間とAI向けにAPIの詳細の説明を記述
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

## API

`/api/openapi.yaml`は`api/lambda/src`配下の実装に付与した`utoipa`注釈（`#[utoipa::path(...)]`等）を`api/lambda/src/bin/export_openapi.rs`で集約・出力した結果を管理する。

### 1. タイムライン取得API
最新のタイムライン（メッセージ一覧）を最大50件取得します。

* メソッド：GET
* パス：/api/v1/timeline
* 認証：必須（Authorization: Bearer <JWT>）
* 内部処理（DB）：
  * 参照元：public.messages_latest ビュー
    * 取得カラム：user_name, created_at, body, is_from_user
    * ソート・制限：ORDER BY created_at DESC LIMIT 50

#### リクエストパラメータ（Query）

* before (string/ISO8601, オプション): 指定した日時より前のメッセージを取得する場合に使用。

#### レスポンス（JSON）

* ステータスコード：200 OK
* データ形式（created_at）：UTCのZ形式（例: 2026-07-02T02:24:00Z）

```json
[
  {
    "user_name": "田中太郎",
    "created_at": "2026-07-02T02:24:00Z",
    "body": "こんにちは！",
    "is_from_user": true
  }
]
```

### 2. 投稿API
新しいメッセージをタイムラインに投稿します。

* メソッド：POST
* パス：/api/v1/messages
* 認証：必須（Authorization: Bearer <JWT>）
* 内部処理（DB）：
  * 挿入先：public.messages テーブル
    * 挿入カラム：body（リクエストから取得）、その他のカラム（リクエストヘッダ、JWTのペイロード、およびシステム日時などから、システムが自動設定）

#### リクエストボディ（JSON）

```json
{
  "body": "新しくメッセージを投稿します。"
}
```

#### レスポンス（JSON）

* ステータスコード：201 Created

```json
{
  "status": "success",
  "message": "Message created successfully"
}
```

## DB接続の実装について

このRustはAWS Lambdaで実行される。Lambdaインスタンスの実行Role`dsql:DbConnect`権限が付与されているため以下の処理でIAM認証が通り、DatabaseConnectionインスタンスが取得できる。

LambdaのAurora DSQL接続処理の例は以下の通り。

```rust
use aurora_dsql_sqlx_connector::pool;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
pub(crate) async fn create_db(
    role: &str,
    endpoint: &str,
    region: &str,
) -> Result<DatabaseConnection, Error> {
    tracing::info!("Creating database connection with Aurora DSQL SQLx connector...");
    let pool = pool::connect(format!("postgres://{role}@{endpoint}/postgres?region={region}"))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(SqlxPostgresConnector::from_sqlx_postgres_pool(pool))
}
```

## AWS LambdaのCold Start時のブーストの活用について

AWS LambdaはARMの128MBを利用しており、1/12コアしか割り当てられていない。  
Cold Start時、INITフェーズで最大2コアのCPUブースト枠が最大10000ms割り当てられるため、INITフェーズの間に重い処理をすべて終わらせておくことを意識して実装する。  
ただし、INITフェーズが10000msで完了しなかった場合初期化が失敗したとみなし環境が再構築されるが、再構築時にはCPUブースト枠は失われている。INITフェーズが10000msで完了するようにすることを意識する。

INVOKEフェーズ以降は1/12コアの割り当てに戻る。そうするとtokioのマルチスレッド処理は不要なので、`#[tokio::main(flavor = "current_thread")]`と明記しシングルスレッド固定にすることでtokioのマルチスレッド処理によるオーバーヘッドによる性能低下を回避する。

Lambdaのエントリポイントの例は以下の通り。

```rust
use lambda_runtime::{Error, run, service_fn};
use db::create_db; // 先述のAurora DSQL接続処理
use common::function_handler; // 別途メイン処理を実装する
use std::env;
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    // ここからINITフェーズ
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = env::var("DSQL_ENDPOINT").map_err(|_| {
        tracing::error!("DSQL_ENDPOINT environment variable is not set");
        anyhow::anyhow!("DSQL_ENDPOINT environment variable is not set")
    })?;

    let dsql_region = env::var("AWS_REGION").map_err(|_| {
        tracing::error!("AWS_REGION environment variable is not set");
        anyhow::anyhow!("AWS_REGION environment variable is not set")
    })?;

    // DB接続の初期化
    // DSQLでユーザー名に紐づくIAM認証をあらかじめ設定している必要がある
    let db = create_db("lambda", &dsql_endpoint, &dsql_region).await?;

    // ここまでINITフェーズ
    // runをawaitした瞬間からINVOKEフェーズ
    run(service_fn(|event| {
        let db = db.clone();
        async move { function_handler(&db, event).await }
    })).await
}
```

## Rust・AWS Lambdaのコーディング規約

- AWS Lambdaの設定
  - メモリ ： 128MB
  - アーキテクチャ ： ARM
  - ランタイム ： provided.al2023
  - タイムアウト ： 10秒
- Cargo.toml
  - `[package]`
    - edition : `2024`
  - `[profile.release]`
    - lto = true
    - codegen-units = 1
    - opt-level = "3"
    - panic = "abort"
    - strip = true
- エントリポイント
  - Lambdaの128MBに割り当てられているCPUは約0.08コア(1/12コア)であるため、シングルスレッドに最適化する。  
    ```rust
    #[tokio::main(flavor = "current_thread")]
    async fn main() {
    ```
- フォーマッター ： rustfmt
- ドキュメンテーションコメント ： 日本語で記載。ドキュメントに出力されることを常に意識して詳細に記載する。
- ドキュメント生成コマンド ： `cargo doc --no-deps`で出力する。※ドキュメントは`/review`レビュー資料管理で使用している。
- テストコード ： `#[cfg(test)]`で作成する。
- コンパイルチェックコマンド ： `cargo check`で実施する。
- 静的解析コマンド ： `cargo clippy`で実施する。
- テスト実施コマンド ： `cargo test --all-features -- --include-ignored`で実施する。
  - プロパティテストは`#[ignore = "プロパティテストは重いためデフォルトでは実行しない"]`としてignoreを指定する。
  - PostgreSQLを使用する重いテストは`#[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]`としてignoreを指定する。
- OpenAPI定義 ： `openapi.yaml`は手書きせず、`api/lambda/src/*.rs`の`utoipa`注釈から`cargo run --bin export_openapi`で生成する。

### 方針

[CONTRIBUTING.md](../api/CONTRIBUTING.md)を参照し、また、一般的なベストプラクティスに従って実装する。

## CI/CD

- jobs
  - validate
    - `sam validate --lint`で`/api/template.yaml`を検証する。
    - `cargo check`でコンパイルチェック
    - `cargo clippy`で静的解析
    - テスト実行用環境構築
      - PostgreSQL v16の立ち上げ
        ```bash
        docker run --rm --name testdb-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -p 5432:5432 -d postgres:16
        until docker exec testdb-postgres pg_isready -U postgres -d postgres; do
          sleep 1
        done
        ```
      - Liquibase v4.33.0の立ち上げ・マイグレーションの実施
        ```bash
        docker run --rm -e LIQUIBASE_HUB_MODE=off --network host -v "${{ github.workspace }}:/workspace" liquibase/liquibase:4.33.0 --search-path=/workspace --changelog-file=db/liquibase/changelog.xml --contexts=local --url=jdbc:postgresql://localhost:5432/postgres --username=postgres --password=postgres update
        ```
    - `cargo test --all-features -- --include-ignored`でテスト実行
  - deploy
    - AWSのクレデンシャルの設定
    - `sam build`でビルド
    - `sam deploy`でデプロイ

## 固定設定

### AWS SAMで使用する設定

| 設定名 | 設定値 |
|--|--|
| ステージ名 | `develop` or `release` |
| サブシステム名 | `api` or `auth` or `db` or `frontend` or `review` |
| スタック名 | sns-${SubSystem}-${Stage} |
| Lambdaの実行用AWSロール名 | sns-db-${Stage}-lambda-role |

### GitHub Actionsで使用する設定

| 設定名 | 設定値 |
|--|--|
| AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v6`の`role-to-assume`に指定するARN |
| SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |

### AWS SAMのOutputsでExportする値

| 概要 | Export名 | Value |
|--|--|
| API GatewayのURL | sns-api-${Stage}-apigatewayurl | !GetAtt <API Gatewayのリソース>.ApiUrl |
