use std::env;

use core_infrastructure::create_db_dsql;
use lambda_runtime::{Error, run, service_fn};
use sns_system_api_lambda::function_handler;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = env::var("DSQL_ENDPOINT")?;
    let dsql_region = env::var("AWS_REGION")?;

    let db = create_db_dsql("lambda", &dsql_endpoint, &dsql_region).await?;

    run(service_fn(|event| {
        let db = db.clone();
        async move { function_handler(&db, event).await }
    }))
    .await
}
