# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

各サブプロジェクトのTODO項目については、各ディレクトリのルートの`TODO.md`ファイルにリストアップする。

## 認証基盤（auth）

- `aws-actions/configure-aws-credentials` のバージョン不一致：`auth/README.md` では `@v4` を指定しているが、`.github/workflows/README.md` では `@v6` が記述されている。どちらのバージョンを正とするか要確認・統一が必要。現在は `auth/README.md` の指定（`@v4`）に従っている。
- `DeletionPolicy: !If [IsRelease, Retain, Delete]` の使用：CloudFormation の公式ドキュメントでは `DeletionPolicy` に組み込み関数（`!If`）を使用することは明示されていないが、実際には動作する。`sam validate --lint` は現時点で問題なく通過している。将来的なCloudFormationの仕様変更に注意が必要。
