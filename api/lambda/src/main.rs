//! Lambda エントリポイント。
//!
//! axumルーターを構成し、`lambda_http` でLambdaとして実行します。
//! Lambdaの制約（128MB, 約0.08コア）に対応するため、シングルスレッドランタイムを使用します。

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use lambda_http::Error;

use sns_api::infrastructure::DsqlMessageRepository;
use sns_api::interface::handler::{AppState, get_timeline, post_message};

/// Lambdaのエントリポイント。
///
/// axumルーターを構成し、lambda_httpでLambdaとして実行します。
/// Lambdaの128MBに割り当てられているCPUは約0.08コア(1/12コア)であるため、
/// シングルスレッドに最適化した `current_thread` フレーバーを使用します。
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let repository = Arc::new(DsqlMessageRepository::from_env());
    let state = AppState {
        repository: repository as Arc<dyn sns_api::application::MessageRepository>,
    };

    let app = Router::new()
        .route("/api/v1/timeline", get(get_timeline))
        .route("/api/v1/messages", post(post_message))
        .with_state(state);

    lambda_http::run(app).await
}
