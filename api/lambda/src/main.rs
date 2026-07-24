//! # バックエンドAPIデータベース Lambda ハンドラー
//!
//! このモジュールは、AWS Lambda上で動作するコンタクトフォームAPIのエントリーポイントです。
//! Amazon API Gateway HTTP API からのリクエストを受け取り、JWTトークンによる認証を行った後、
//! HTTPメソッドやパスに基づいて適切なハンドラーにルーティングします。
//!
//! ## アーキテクチャ概要
//!
//! ```text
//! クライアント
//!   └─▶ Amazon API Gateway HTTP API (JWT Authorizer)
//!         └─▶ AWS Lambda (このモジュール)
//!               └─▶ Amazon Aurora DSQL (SeaORM経由)
//! ```
//!
//! ## 認証フロー
//!
//! 1. クライアントは Amazon Cognito からJWT IDトークンを取得する
//! 2. JWT IDトークンを `Authorization: Bearer <token>` ヘッダーに付与してリクエストを送信する
//! 3. API Gateway の JWT Authorizer がトークンを検証する
//! 4. 検証済みのJWTクレーム（`email`、`sub`など）が `authorizer.jwt.claims` に格納される
//! 5. このクレームを読み取り、ユーザーを識別する
//!
//! ## 見どころ
//!
//! Cold Start 中にログ初期化と DSQL 接続を完了させ、INVOKE フェーズでは
//! 共有済みコネクションを用いて HTTP ハンドラだけを実行する構成にしています。
//!
//! ## 環境変数
//!
//! | 変数名 | 必須 | 説明 |
//! |--------|------|------|
//! | `DSQL_ENDPOINT` | ✓ | Aurora DSQLクラスターのエンドポイント |
//! | `AWS_REGION` | ✓ | Aurora DSQLクラスターのAWSリージョン |

use std::env;

use core_infrastructure_lambda::lambda_executor;
use lambda_runtime::Error;
use sns_system_api_lambda::function_handler;

#[tokio::main(flavor = "current_thread")]
/// DSQL 接続を初期化し、Lambda ランタイムへ HTTP ハンドラを登録する。
async fn main() -> Result<(), Error> {
    // INITフェーズここから
    // INITフェーズは最大10000ms。vCPUが2コア与えられるので、この間に重い処理を済ませておく。
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = env::var("DSQL_ENDPOINT")?;
    let dsql_region = env::var("AWS_REGION")?;

    use core_infrastructure_db::{AuroraDSQLConnectionInfo, DBType, create_db};
    let db = create_db(DBType::AuroraDSQL(AuroraDSQLConnectionInfo {
        role: "lambda",
        endpoint: &dsql_endpoint,
        region: &dsql_region,
    }))
    .await?;

    // INITフェーズここまで
    // INITフェーズが10000ms以内に終わらなかった場合、初期化に失敗したと判断され再起動してしまうので注意。

    // INVOKEフェーズここから
    // INVOKEフェーズではvCPUが1/12コア(約0.08コア)しか割り当てられないため、シングルスレッドで処理する。
    lambda_executor(|event| {
        // INVOKEフェーズでは、Cold Startで作成した共有済みコネクションを使ってHTTPハンドラを実行する。
        function_handler(&db, event)
    })
    .await
}
