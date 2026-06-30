# sns-system-public

## GitHubリポジトリの設定

### GitHub Actionsのsecrets

| 設定名 | 設定値 |
|--|--|
| secrets.AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v4`の`role-to-assume`に指定するARN |
| secrets.SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |
| secrets.ALLOW_DOMAIN | ユーザー登録できるメールアドレスのドメイン部 |

### ブランチ保護

- `Require a pull request before merging` ： ON
  該当のブランチにコミットをpushすることができなくなり、プルリクエストの承認が必須になる
- `Require status checks to pass before merging` ： ON  
  自動テストなどが通ることが必須となる
  - `Require branches to be up to date before merging` ： ON  
    自動テストが、マージする対象の最新資産から修正したものであることが必須となる
- `Automatically request Copilot code review` ： ON  
  プルリクエストをAI(GitHub Copilot)が自動でレビューする
- `Restrict deletions` ： ON
- `Block force pushes` ： ON
