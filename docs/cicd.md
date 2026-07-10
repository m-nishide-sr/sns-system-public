# SNSシステム CI/CD

## 概要

これは、SNSシステムのCI/CDです。

## 構成と主なファイルの各説明

- /.github/workflows/ ： CI/CD格納ディレクトリ
  - /.github/workflows/sns-system-api-cicd.yaml ： APIのCI/CDのGitHub Actions定義
    - jobs
      - validate ： 自動テスト・静的解析を実施する
      - deploy ： デプロイを実施する
  - /.github/workflows/sns-system-auth-cicd.yaml ： AuthのCI/CDのGitHub Actions定義
    - jobs
      - validate ： 自動テスト・静的解析を実施する
      - deploy ： デプロイを実施する
  - /.github/workflows/sns-system-db-cicd.yaml ： DBのCI/CDのGitHub Actions定義
    - jobs
      - validate ： 自動テスト・静的解析を実施する
      - migrate ： マイグレーションを実施する
      - generate_orm ： ORM定義を生成する
  - /.github/workflows/sns-system-frontend-cicd.yaml ： フロントエンドのCI/CDのGitHub Actions定義
    - jobs
      - validate ： 自動テスト・静的解析を実施する
      - deploy ： ビルド・デプロイを実施する
  - /.github/workflows/sns-system-review-cicd.yaml ： レビュー用資料管理のCI/CDのGitHub Actions定義
    - jobs
      - validate ： 自動テスト・静的解析を実施し、結果を保持する
      - deploy ： 結果をデプロイする

## 固定設定

### AWS SAMで使用する設定

| 設定名 | 設定値 |
|--|--|
| ステージ名 | `develop` or `release` |
| サブシステム名 | `api` or `auth` or `db` or `frontend` or `review` |
| スタック名 | sns-${SubSystem}-${Stage} |
| Lambdaの実行用AWSロール名 | sns-db-${Stage}-lambda-role |

### GitHub Actionsで使用する設定

| 設定名 | 設定値 |
|--|--|
| secrets.AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v6`の`role-to-assume`に指定するARN |
| secrets.SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |

認証はGitHubのOIDCで`aws-actions/configure-aws-credentials@v6`から認証を通るように設定済みなので、`role-to-assume`に適切なロールのARNを設定していればSAMをデプロイ可。
