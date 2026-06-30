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
  - /db ： DBのルート
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

## 認証サブシステム インフラ構成

- /auth/template.yaml
  - Cognito
    - ユーザプール ： `Type: AWS::Cognito::UserPool`
      - パスワードは一般的な強度を設定
      - `AutoVerifiedAttributes`および`UsernameAttributes`は`email`を指定
      - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`
    - ユーザプールクライアント ： `Type: AWS::Cognito::UserPoolClient`
      - セキュリティ設定は一般的なベストプラクティスに従い設定し、コメントには「なぜ」を記載する
      - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`

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
