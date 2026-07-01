# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

## 矛盾点・確認事項

- `docs/db.md` の「固定設定 > AWS SAMのOutputsでExportする値」テーブルにおいて、`sns-${SubSystem}-${Stage}-LambdaRoleArn` の Value が `!GetAtt <'Type: AWS::Cognito::UserPoolClient'のリソース>.Arn` と記載されているが、DBサブシステムにCognitoリソースは存在しない。IAMロールの誤記と判断し、`!GetAtt LambdaRole.Arn` として実装した。
- `docs/db.md` の「CI/CD > validate」にて `sam validate --lint` の対象が `/api/template.yaml` と記載されているが、DBのCI/CDとして `db/template.yaml` を対象に実装した。
- `docs/db.md` に `aws dsql get-token` (migrateジョブ) と `aws dsql generate-db-connect-admin-auth-token` (generate_ormジョブ) の2種類のトークン取得コマンドが記載されている。それぞれのコマンドを使用して実装した。

## 実装メモ

- Liquibaseの `--password` 引数はシークレットスキャナーによりマスクされるため、環境変数 `LIQUIBASE_COMMAND_PASSWORD` を使用して渡す形式で実装した。
- `db/sea_orm_entities/src/entity/mod.rs` は `sea-orm-cli generate entity` 実行前のコンパイルエラー回避のためのプレースホルダーとして作成した。generate_orm ジョブ実行後に自動上書きされる。

