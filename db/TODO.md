# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

## 実装タスク

テーブル設計を実施し、以下のファイルを追加・修正する。

- `db/liquibase/changes/*.sql`
  - DDLを記述する
- `db/README.md`
  - テーブル定義を記載する

## Liquibaseのv5へのバージョンアップの未対応

Liquibase v5ではJDBCドライバが同梱されなくなり、`lpm`コマンドでJDBCドライバをダウンロードできるようになるとのことだが未対応。一時的にLiquibaseのバージョンをv4.33.0に下げている。

- 将来的な修正対象ファイル
  - `db/README.md`
  - `.github/workflows/sns-system-db-cicd.yaml`
