# FAQ

## なぜLiquibaseのバージョンはv4.33.0なのですか？

Liquibase v5ではJDBCドライバが同梱されなくなってしまったため。  
Liquibaseのドキュメントによると`lpm`コマンドを実行することでJDBCドライバをダウンロードできるようになるとのことだが動作未確認。

- 将来的な修正対象ファイル
  - `/db/README.md`
  - `/.github/workflows/sns-system-db-cicd.yaml`

## なぜaurora-dsql-sqlx-connectorのバージョンはv0.2.0なのですか？

`sea-orm`(v1)がsqlx(v0.8.6)、`aurora-dsql-sqlx-connector`(v0.2.1)がsqlx(v0.9.0)と、それぞれの依存しているsqlxのバージョン同士で互換性の無い破壊的変更が入ったため。(poolの型に変更が入った、など)
当事象が解消が確認できるまで、一時的に`aurora-dsql-sqlx-connector`のバージョンをv0.2.0のままにする。

- 将来的な修正対象ファイル
  - `api/README.md`
  - `api/lambda/Cargo.toml`
