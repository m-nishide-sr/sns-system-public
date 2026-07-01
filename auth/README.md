# SNSシステム AUTHサブシステム

## 概要

これは、SNSシステムの認証基盤管理用サブシステムです。

## 構成と主なファイルの各説明

- / ： リポジトリルート
  - /.github/workflows/sns-system-auth-cicd.yaml ： このプロジェクトの認証のCI/CDのGitHub Actions定義
  - /README.md ： システム全体の概要説明
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /api ： APIのルート
  - /auth ： 認証のルート
    - /auth/README.md ： 人間とAI向けにDBの詳細の説明を記述する。
    - /auth/AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
    - /auth/template.yaml ： 認証のIaC
    - /auth/PreSignUpFunction ： PreSignUpFunctionを実装するルートディレクトリ
      - /auth/PreSignUpFunction/Cargo.toml ： パッケージのマニフェストファイル
      - /auth/PreSignUpFunction/src/main.rs ： PreSignUpFunctionの実装
  - /db ： DBのルート
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

## 認証サブシステム インフラ構成

- /.github/workflows/sns-system-auth-cicd.yaml ： CI/CD
  - jobs
    - validate
      - steps
        - `sam validate --lint`の実行
        - `cargo check`でコンパイルチェック
        - `cargo clippy`で静的解析
        - `cargo test --all-features -- --include-ignored`でテスト実行
    - deploy
      - needs: validate
      - if: 
        - github.event_name == 'push'
        - ブランチが`develop` or `release`
      - steps
        - AWSのクレデンシャルの設定
          - uses: aws-actions/configure-aws-credentials@v6
            - with:
              - role-to-assume: ${{ secrets.AWS_DEPLOY_ROLE_ARN }}
              - aws-region: ap-northeast-3
        - `sam build`の実行
        - `sam deploy`の実行
          - `--role-arn ${{ secrets.SAM_DEPLOY_ROLE_ARN }}`
          - `--parameter-overrides`
            - Stage=developブランチでは`develop`、releaseブランチでは`release`
- /auth/template.yaml
  - Parameters
    - Stage ： デプロイステージ
      - Type: String
      - AllowedValues:
        - develop
        - release
    - SubSystem ： サブシステム分類名(ここでは`auth`のみ)
      - Type: String
      - Default: auth
      - AllowedValues:
        - auth
  - Resources
    - Cognito
      - ユーザプール ： `Type: AWS::Cognito::UserPool`
        - パスワードは一般的な強度を設定
        - `AutoVerifiedAttributes`および`UsernameAttributes`は`email`を指定
        - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`
      - ユーザプールクライアント ： `Type: AWS::Cognito::UserPoolClient`
        - セキュリティ設定は一般的なベストプラクティスに従い設定し、コメントには「なぜ」を記載する
        - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`
    - Lambda ： `Type: AWS::Serverless::Function`
      - Properties
        - Architectures
          - arm64
        - Timeout: 30
        - MemorySize: 128
        - Runtime: provided.al2023
        - CodeUri: ./PreSignUpFunction
        - Environment:
          - Variables:
            - ALLOWED_EMAIL_DOMAINS: secrets.ALLOW_DOMAIN
      - Metadata
        - `BuildMethod: rust-cargolambda`

## PreSignUpFunction

Rustで実装する。

メールアドレスのドメイン部が`ALLOWED_EMAIL_DOMAINS`(secrets.ALLOW_DOMAIN)と一致するか検証する。

## 設定

### GitHub Actionsのsecrets

| 設定名 | 設定値 |
|--|--|
| secrets.AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v6`の`role-to-assume`に指定するARN |
| secrets.SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |
| secrets.ALLOW_DOMAIN | ユーザー登録できるメールアドレスのドメイン部 |

### AWS SAMで使用する設定

| 設定名 | 設定値 |
|--|--|
| ステージ名 | `develop` or `release` |
| サブシステム名 | `api` or `auth` or `db` or `frontend` or `review` |
| スタック名 | sns-${SubSystem}-${Stage} |

### AWS SAMのOutputsでExportする値

| 概要 | Export名 | Value |
|--|--|
| CognitoのユーザプールID | sns-${SubSystem}-${Stage}-UserPoolId | !Ref <`Type: AWS::Cognito::UserPool`のリソース> |
| CognitoのユーザプールクライアントID | sns-${SubSystem}-${Stage}-UserPoolClientId | !Ref <`Type: AWS::Cognito::UserPoolClient`のリソース> |
| CognitoのJWT Issuer URL | sns-${SubSystem}-${Stage}-CognitoIssuer | !Sub https://cognito-idp.${AWS::Region}.amazonaws.com/${UserPoolId} |
