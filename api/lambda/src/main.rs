//! API Lambda の本番エントリーポイント。
//!
//! Cold Start 中にログ初期化と DSQL 接続を完了させ、INVOKE フェーズでは
//! 共有済みコネクションを用いて HTTP ハンドラだけを実行する構成にしています。

use std::env;

use core_infrastructure::create_db_dsql;
use lambda_runtime::{Error, run, service_fn};
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

    let db = create_db_dsql("lambda", &dsql_endpoint, &dsql_region).await?;

    // INITフェーズここまで
    // INITフェーズが10000ms以内に終わらなかった場合、初期化に失敗したと判断され再起動してしまうので注意。

    // INVOKEフェーズここから
    // INVOKEフェーズではvCPUが1/12コア(約0.08コア)しか割り当てられないため、シングルスレッドで処理する。
    run(service_fn(|event| {
        let db = db.clone();
        async move { function_handler(&db, event).await }
    }))
    .await
}
