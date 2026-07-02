# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

## 実装上の考慮事項・注意点

### api/template.yaml

- `docs/api.md` の固定設定表では `!GetAtt <API Gatewayのリソース>.ApiUrl` と記載されているが、`AWS::Serverless::HttpApi` の実際の有効な属性は `ApiUrl` ではなく `ApiEndpoint` である。`sam validate --lint` の検証に基づき `!GetAtt Api.ApiEndpoint` に変更済み。

### api/lambda/src/infrastructure/dsql_message_repository.rs

- `DsqlMessageRepository::from_env()` は環境変数 `DSQL_ENDPOINT` から接続先エンドポイントを取得する。ローカル開発時は `DB_PASSWORD` も設定することでIAM認証を省略できる。
- 統合テスト（`#[ignore]`）はCI/CDでPostgreSQL+Liquibaseを立ち上げた後に `DSQL_ENDPOINT=localhost DB_PASSWORD=lambda` で実行される。

### api/lambda/src/interface/handler.rs

- `post_message` ハンドラはAPI GatewayのJWTオーソライザーが付与する `RequestContext` を `axum::Extension<RequestContext>` で取得する。これは `lambda_http` が自動的にリクエスト拡張として追加するため、ルーターへの明示的な `layer()` 追加は不要。
