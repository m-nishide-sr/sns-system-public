# sns-system-public

これはSNSシステムの公開リポジトリです。

## 名前の由来

S(すごくAIとか活用して開発している)N(内部コミュニケーション用の)S(社交的交流を促進するための)システム

# リポジトリの説明

## リポジトリ構成

- / ： リポジトリルート
  - /.github/workflows ： CI/CDのGitHub Actions定義格納ディレクトリ
  - /README.md ： このファイル
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /TODO.md ： バックログや懸念事項等のほか、アイデアやビジョンなども随時追記しメンテナンスする。
  - /api ： API用サブプロジェクト格納ディレクトリ
  - /auth ： 認証用サブプロジェクト格納ディレクトリ
  - /db ： DB用サブプロジェクト格納ディレクトリ
  - /docs ： 基本設計書格納ディレクトリ
  - /frontend ： フロントエンド用サブプロジェクト格納ディレクトリ
  - /review ： レビュー資料デプロイ用サブプロジェクト格納ディレクトリ

## GitHubリポジトリの設定

### GitHub Actionsのsecrets

| 設定名 | 設定値 |
|--|--|
| secrets.AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v6`の`role-to-assume`に指定するARN |
| secrets.SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |
| secrets.ALLOW_DOMAIN | ユーザー登録できるメールアドレスのドメイン部("@"は含まない) |
| secrets.SECRET_REFERER | レビュー資料用のCloudFrontとオリジンとするS3のWebsiteURL間でヘッダーの`aws:Referer`に使用するシークレット文字列 |

### ブランチ保護

- `Require signed commits` ： ON
  署名付きのコミットのみマージできるようになる
- `Require a pull request before merging` ： ON
  該当のブランチにコミットをpushすることができなくなり、プルリクエストの承認が必須になる
- `Require status checks to pass before merging` ： ON  
  自動テストなどをパスすることが必須となる
  - `Require branches to be up to date before merging` ： ON  
    自動テストが、マージする対象の最新資産から修正したものであることが必須となる
  - `Status checks that are required` ： `SNSシステム レビュー資料管理 CI/CD / 検証`  
    パス必須にするチェック名（通常は「<workflow name> / <job name>」の形式で表示される）
- `Automatically request Copilot code review` ： ON  
  プルリクエストをAI(GitHub Copilot)が自動でレビューする
- `Restrict deletions` ： ON
- `Block force pushes` ： ON
