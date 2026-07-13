# FAQ

## なぜLiquibaseのバージョンはv4.33.0なのですか？

Liquibase v5ではJDBCドライバが同梱されなくなってしまったため。  
Liquibaseのドキュメントによると`lpm`コマンドを実行することでJDBCドライバをダウンロードできるようになるとのことだが動作未確認。

- 将来的な修正対象ファイル
  - `/db/README.md`
  - `/.github/workflows/sns-system-db-cicd.yaml`

## なぜaurora-dsql-sqlx-connectorのバージョンはv0.2.0なのですか？

`sea-orm`(v1)がsqlx(v0.8.6)、`aurora-dsql-sqlx-connector`(v0.2.1)がsqlx(v0.9.0)と、それぞれの依存しているsqlxのバージョン同士で互換性の無い破壊的変更が入ったため。(poolの型に変更が入った、など)
当事象が解消が確認できるまで、一時的に`aurora-dsql-sqlx-connector`のバージョンをv0.1.2のままにする。

- 将来的な修正対象ファイル
  - `api/README.md`
  - `api/lambda/Cargo.toml`

## なぜCloudFrontの定額プランをtemplate.yamlで設定していないのですか？

現在、AWS CloudFormationがCloudFrontの定額プランの設定に対応していないため。
AWS側で対応され次第バージョンアップし追随する予定。それまではマネジメントコンソール上から手動で設定するしかない。

- 将来的な修正対象ファイル
  - `frontend/template.yaml`

## なぜ`public.messages`のSELECT権限を`lambda`ロールに付与しているのですか？

ActiveModel の insert は、SeaORM 1.1.20 では内部的に exec_with_returning を呼びます（モデルを返すため）。
PostgreSQL は RETURNING で参照する列に対して SELECT 権限を要求します。
SeaORM の仕様が変わらない限りこの権限に関する変更予定はありません。
