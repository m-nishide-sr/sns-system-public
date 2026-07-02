mod auth;
mod db;
mod http;

use lambda_http::{run, service_fn};
use lambda_runtime::Error;
use std::{env, sync::Arc};

use crate::db::{SqlxMessageRepository, create_db};

/// Lambdaエントリポイント。
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = env::var("DSQL_ENDPOINT").map_err(|_| {
        tracing::error!("DSQL_ENDPOINT環境変数が設定されていません");
        anyhow::anyhow!("DSQL_ENDPOINT環境変数が設定されていません")
    })?;
    let dsql_region = env::var("AWS_REGION").map_err(|_| {
        tracing::error!("AWS_REGION環境変数が設定されていません");
        anyhow::anyhow!("AWS_REGION環境変数が設定されていません")
    })?;

    let db = create_db("lambda", &dsql_endpoint, &dsql_region).await?;
    let repository = Arc::new(SqlxMessageRepository::new(db));

    run(service_fn(move |request| {
        let repository = Arc::clone(&repository);
        async move { http::function_handler(repository.as_ref(), request).await }
    }))
    .await
}
