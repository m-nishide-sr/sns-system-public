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
    - /api/README.md ： 人間とAI向けにAPIの詳細の説明を記述する。
    - /api/AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
    - /api/template.yaml ： APIのIaC
    - /api/openapi.yaml ： API定義(自動生成)。
    - /api/lambda ： Lambdaを実装するRustパッケージ
      - /api/lambda/Cargo.toml ： パッケージのマニフェストファイル。
      - /api/lambda/src/*.rs ： Rustプログラム実装
      - /api/lambda/src/bin/export_openapi.rs ： `openapi.yaml`生成関数
  - /db ： DBのルート
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

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

### 方針

[CONTRIBUTING.md](./CONTRIBUTING.md)を参照し、また、一般的なベストプラクティスに従って実装する。

## CI/CD

- jobs
  - validate
    - `sam validate --lint`で`/api/template.yaml`を検証する。
    - `cargo check`でコンパイルチェック
    - `cargo clippy`で静的解析
    - テスト実行用環境構築
      - PostgreSQL v16の立ち上げ
        ```bash
          docker run --name testdb-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -p 5432:5432 -d postgres:16
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

