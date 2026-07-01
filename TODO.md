# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

各サブプロジェクトのTODO項目については、各ディレクトリのルートの`TODO.md`ファイルにリストアップする。

- [ ] `docs/db.md` の `AWS::DSQL::Cluster` 出力属性が `ConnectionString` になっているが、`sam validate --lint` では `Endpoint` のみ有効だったため、ドキュメントとの整合を確認する。
- [ ] `docs/db.md` の `public.messages.cognito_id` のデフォルト値が `''` になっているが、`uuid` 型では不正なため、デフォルトなし前提でよいか仕様を確認する。
