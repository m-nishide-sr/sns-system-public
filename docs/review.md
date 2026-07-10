# SNSシステム レビュー資料管理

## 概要

これは、SNSシステムのレビュー資料管理です。

## 構成と主なファイルの各説明

- / ： リポジトリルート
  - /.github/workflows/sns-system-review-cicd.yaml ： レビュー資料管理のCI/CDのGitHub Actions定義
  - /README.md ： システム全体の概要説明
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /api ： APIのルート
  - /auth ： 認証のルート
  - /db ： DBのルート
  - /docs ： ドキュメントのルート
    - /docs/review.md ： 人間とAI向けにレビュー資料管理の詳細の説明を記述
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料管理のルート
    - /review/AGENTS.md ： AI向けプロンプトを記述
    - /review/template.yaml ： レビュー資料公開インフラのIaC

## レビュー資料管理 インフラ構成

- /review/template.yaml
  - CloudFront ： `Type: AWS::CloudFront::Distribution`
    - `PriceClass: PriceClass_200` ： 日本を含む"200"を指定
    - `PricingPlan: Free` ： Flat-Rate PlanをFreeで作成 ※現在、定額プランはマネジメントコンソール上からしか設定できないため手動で実施。
    - 認証不要。
    - 一般的には`*.cloudfront.net`ドメインは固定でなく一時的なもののため独自ドメインを利用するが、これは関係者内でのみ参照し外部から参照する類のものではないため、`*.cloudfront.net`ドメインを使用する。
    - DefaultRootObject: index.html
  - S3
    - 上記のCloudFrontのオリジン。
    - CI/CDでテスト結果やドキュメントなどのレビュー用資料を公開する。
    - パブリックアクセスはブロックし、CloudFrontからのみアクセスさせる。
    - 認証不要。
    - 365日で削除されるようにS3 Object Expirationを設定。

## CI/CD

- プルリクエストをトリガーとして実行される。
  - `cargo tarpaulin --out Html -- --include-ignored`を実行し、カバレッジレポートを出力する。
  - `cargo doc --no-deps`を実行し、Rustのドキュメントを出力する。
  - `cargo run --bin export_openapi`を実行し、openapi.yamlを出力する。
  - `/review/template.yaml`をAWS SAMでデプロイする。
  - フロントエンドのJSDocを出力する。
  - フロントエンドのStorybookを出力する。
  - コミットID(short12桁)の名前のディレクトリを作成し、カバレッジレポート、Rustのドキュメント、openapi.yaml、JSDoc、Storybook、そしてそれぞれの`index.html`へのリンクを記述したindex.htmlを配置する。
  - 作成したドキュメントをS3にアップロードする。
  - `github.rest.repos.createDeploymentStatus`によりURL、すなわち`https://*.cloudfront.net/コミットID(short12桁)/index.html`を`environment_url`で通知する。
