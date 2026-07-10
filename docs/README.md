# SNSシステム 基本設計書

## システム概要

### SNSシステムの名前の由来

「S(すごくAIとか活用して開発している)N(内部コミュニケーション用の)S(社交的交流を促進するための)システム」が由来です。

## 目次

### システムを構成する各サブプロジェクトについて

* [レビュー資料管理](./review.md)
* [API](./api.md)
* [認証基盤](./auth.md)
* [CI/CD](./cicd.md)
* [ビジネスロジック](./core.md)
* [DB](./db.md)
* [フロントエンド](./frontend.md)

### 各レビュー用資料へのリンク

<ul>
  <li><a href="coverage/tarpaulin-report.html">カバレッジレポート</a></li>
  <li><a href="rust-doc/sns_system_api_lambda/sns_system_api_lambda/index.html">Rustドキュメント(API)</a></li>
  <li><a href="rust-doc/sns_system_auth_pre_signup_function/bootstrap/index.html">Rustドキュメント(PreSignUpFunction)</a></li>
  <li><a href="rust-doc/sns_system_db_sea_orm_entities/sea_orm_entities/index.html">Rustドキュメント(Sea Orm Entities)</a></li>
  <li><a href="rust-doc/sns_system_core/core_common/index.html">Rustドキュメント(core/common)</a></li>
  <li><a href="rust-doc/sns_system_core/core_domain/index.html">Rustドキュメント(core/domain)</a></li>
  <li><a href="rust-doc/sns_system_core/core_infrastructure/index.html">Rustドキュメント(core/infrastructure)</a></li>
  <li><a href="rust-doc/sns_system_core/core_usecase/index.html">Rustドキュメント(core/usecase)</a></li>
  <li><a href="openapi.yaml">OpenAPI仕様</a></li>
  <li><a href="stoplight/index.html">API仕様書(Stoplight Elements)</a></li>
  <li><a href="redoc/index.html">API仕様書(Redoc)</a></li>
  <li><a href="rapidoc/index.html">API仕様書(RapiDoc)</a></li>
  <li><a href="storybook/index.html">Storybook</a></li>
</ul>

### その他

* [人間向けREADME](./HUMAN_README.md)
* [バックログ](./TODO.md)
* [よくある質問](./faq.md)
* [履歴](./history.md)
