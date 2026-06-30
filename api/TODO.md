# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

## 依存sqlxのバージョン不整合

`sea-orm`(v1)がsqlx(v0.8.6)、`aurora-dsql-sqlx-connector`(v0.2.1)がsqlx(v0.9.0)と、それぞれの依存しているsqlxのバージョン同士で互換性の無い破壊的変更がある。(poolの型に変更が入った、など)
当事象が解消されるまで、一時的に`aurora-dsql-sqlx-connector`のバージョンをv0.2.0に下げている。

- 将来的な修正対象ファイル
  - `api/README.md`
  - `api/lambda/Cargo.toml`
