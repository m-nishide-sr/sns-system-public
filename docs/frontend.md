# SNSシステム フロントエンドサブシステム

## 概要

これは、SNSシステムのフロントエンド開発用サブシステムです。

## 構成と主なファイルの各説明

- / ： リポジトリルート
  - /.devcontainer/api/devcontainer.json ： API開発で利用するdevcontainer
  - /.github/workflows/sns-system-frontend-cicd.yaml ： このプロジェクトのフロントエンドのCI/CDのGitHub Actions定義
  - /README.md ： システム全体の概要説明
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /api ： APIのルート
  - /auth ： 認証のルート
  - /db ： DBのルート
  - /docs ： ドキュメントのルート
    - /docs/frontend.md ： 人間とAI向けにフロントエンドの詳細の説明を記述
  - /frontend ： フロントエンドのルート
    - /frontend/template.yaml ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

## フロントエンド 概要

- 静的SPAサイト
  - 実装フレームワーク：Next.js
  - 実装言語：TypeScript
  - 概要：SPAで、URLのハッシュルーティングでページを構成する。実装の実物HTMLファイルは`index.html`とエラーページの`403.html`のみ。
  - デザイン：モダンでおしゃれで洗練された見た目の、どんな端末でも見やすいレスポンシブデザインとする。ダークモードは不要。
  - ビルド：ビルドはCI/CDで実行する。
    - 環境変数：ビルド時に、環境変数を参照してSPAサイトに静的に埋め込む。詳細は後述。
      - CognitoのWEBクライアントID
      - CognitoのユーザプールID
      - リージョン
  - 通信処理部
    - URL ： APIのエンドポイント。相対パスで`/api/*`。
    - headers:
      - 'Content-Type': 'application/json'
      - Authorization: `Bearer ${token}`
    - API
      - チャット取得API ： GET `/api/chat`
        - body ： なし
      - チャット投稿API ： POST `/api/chat`
        - body: JSON.stringify(以下の通り)
          - "body": テキストエリアの文字列
  - 認証部
      - 認証は`aws-amplify`を使用して実装する。
      - `aws-amplify`の設定は`Amplify.configure`で設定する。AuthのCognitoの`region`、`userPoolId`、および`userPoolWebClientId`はCI/CDのビルド時に環境変数の値をセットする。この環境変数はCI/CDでAWS SAMの`aws cloudformation describe-stacks`で取得したもの。
  - ページ構成
    - トップページ：適当な説明文を表示する。ログインページへのリンクはわかりやすい場所に配置する。
    - マイページ：パスワード変更や退会、ログアウトなどができる。退会は`aws-amplify/auth`の`deleteUser`をコールするだけで、チャットデータの削除などは実施しない。
    - チャットページ：初期表示時にチャット取得クエリを実行する。
      - 投稿部：テキストエリアと投稿ボタン。文字数チェックはしない。投稿ボタンでチャット投稿クエリを実行する。
      - 更新ボタン：チャット取得クエリを実行する。
    - ログインページ兼新規登録ページ
      - 新規登録ページでは、メールアドレスは"secrets.ALLOW_DOMAIN"しか登録できないことを明記し、実際にCognitoの`PreSignUp`ではメールアドレスのドメイン部を検証する。
      - 新規登録後、および、メールアドレス未承認ステータスの場合に、MFAコードを入力するインターフェースを表示する。
    - 利用規約ページ：(**TODO:あとで修正するので、取り急ぎ無難な内容をtsx上にべた書きしておいて下さい。**)
      - 一度投稿した内容は修正も削除もできないことを記載する。
      - パスワードが流出し、攻撃者によりパスワードが不正に変更されてしまった場合、回復できないことを記載する。
  - 共通部
    - ヘッダー：ページ上部に固定するアプリケーションヘッダ。ただしトップページにはヘッダは表示しない。左端にハンバーガーメニュー、右端にマイページボタンもしくはログインボタン。真ん中にはページタイトルで、文字列を表示するのに幅が足りない場合は`text-overflow: ellipsis`で省略する。
    - フッター：著作権表示。万国著作権条約に従う。
    - メニュー：ハンバーガーメニューを押したときに表示する。各ページへのリンクをリストアップする。
    - トースター：成功時や失敗時のメッセージを表示する。しばらくしたら自動的に消える。
  - 特記事項
    - 各コンポーネントはStorybookでのテスト・確認の実施が容易なように、Presentational and Container Componentsパターンを意識して実装すること。

## フロントエンド インフラ構成

- /frontend/template.yaml
  - CloudFront ： `Type: AWS::CloudFront::Distribution`
    - `PricingPlan: Free` ： Flat-Rate PlanをFreeで作成
    - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`
    - `Origins`
      - API Gatewayオリジン ： !ImportValue sns-api-${Stage}-apigatewayurl
      - S3オリジン ： 以下に記載
    - `CacheBehaviors`
  - S3
    - 上記のCloudFrontのオリジン。
    - フロントエンドの静的WEBページを公開する。
    - パブリックアクセスはブロックし、CloudFrontからのみアクセスさせる。
    - 認証不要。

### AWS SAMで使用する設定

| 設定名 | 設定値 |
|--|--|
| ステージ名 | `develop` or `release` |
| サブシステム名 | `api` or `auth` or `db` or `frontend` or `review` |
| スタック名 | sns-${SubSystem}-${Stage} |

### AWS SAMのOutputsでExportする値

| 設定名 | Export名 | Value |
|--|--|
| CloudFrontのドメイン名 | sns-${SubSystem}-${Stage}-DomainName | !GetAtt <`Type: AWS::CloudFront::Distribution`のリソース>.DomainName |