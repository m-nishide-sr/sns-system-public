# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

各サブプロジェクトのTODO項目については、各ディレクトリのルートの`TODO.md`ファイルにリストアップする。

## 既存不整合の確認事項

- `db/sea_orm_entities/src/lib.rs` が `pub mod entity;` を参照しているが、`src/entity` が未生成のため `cargo check` が失敗する。DB側のORM生成フローで追って解消が必要。
