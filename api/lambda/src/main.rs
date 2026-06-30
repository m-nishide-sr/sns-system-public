//! Lambda エントリポイント。
//!
//! API Gatewayからのリクエストを受け取り、GETとPOSTのルーティングを行う。

use lambda_http::{Error, Request, http::Method, run, service_fn};
use tracing_subscriber::fmt;

use lambda::interface::handler::{handle_get_chat, handle_post_chat};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 構造化ログの初期化
    fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .json()
        .init();

    run(service_fn(router)).await
}

/// リクエストのHTTPメソッドに応じてハンドラーをルーティングする。
async fn router(request: Request) -> Result<impl lambda_http::IntoResponse, Error> {
    match request.method() {
        &Method::GET => handle_get_chat(request).await,
        &Method::POST => handle_post_chat(request).await,
        method => {
            tracing::warn!("未対応のHTTPメソッド: {method}");
            Ok(lambda_http::Response::builder()
                .status(405)
                .body(lambda_http::Body::Text(
                    r#"{"message":"Method Not Allowed"}"#.to_string(),
                ))?)
        }
    }
}
