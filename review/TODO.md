# TODO

考慮不足、矛盾点、未解決問題、外部パッケージの修正待ち、その他TODOをここにリストアップする。

## 考慮不足・未解決事項

- **JSDocが未実装**: `docs/review.md` にはフロントエンドのJSDoc生成が要件として記載されているが、`frontend/package.json` にJSDocの依存関係・スクリプトが存在しない。フロントエンドにJSDoc（またはTypedoc）のセットアップが必要。CI/CDワークフローでは現時点でJSDocの生成ステップを省略している。

- **CloudFront Flat-Rate Plan（PricingPlan: Free）の手動設定**: `docs/review.md` に「定額プランはマネジメントコンソール上からしか設定できないため手動で実施」と記載されている通り、`review/template.yaml` ではFree Flat-Rate Planを自動設定できない。CloudFormation/SAMのリソースとしてPricingPlanを定義できないため、初回デプロイ後に手動でPricingPlanをFreeに変更する必要がある。

- **cargo-tarpaulinのインストール時間**: CI/CDで `cargo install cargo-tarpaulin --locked` を実行するため、毎回コンパイルが走りビルド時間が増大する。`taiki-e/install-action` の使用やバイナリキャッシュの導入を検討する。

- **デプロイ失敗時のGitHub Deployment Statusの未処理**: deployジョブでS3アップロードやSAMデプロイが失敗した場合、GitHub Deploymentのステータスが `in_progress` のまま残る。`if: always()` を使ったfailureステータスへの更新を追加することを検討する。

- **Storybookビルドの環境変数依存**: `npm run build-storybook` が特定の環境変数（Cognito設定等）を必要とする場合、CI上で失敗する可能性がある。必要に応じてモック設定やビルド時の環境変数設定を追加する。
