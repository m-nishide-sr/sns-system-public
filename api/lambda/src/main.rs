use std::env;

use lambda_http::{Error, run, service_fn};
use sea_orm::{Database, DatabaseConnection};
use sns_system_api_lambda::function_handler;

/// Aurora DSQLへ接続し、SeaORMの`DatabaseConnection`を返す。
async fn create_db(role: &str, endpoint: &str, region: &str) -> Result<DatabaseConnection, Error> {
    let url = format!("postgres://{role}@{endpoint}/postgres?region={region}");
    Database::connect(&url).await.map_err(Into::into)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    let dsql_endpoint = env::var("DSQL_ENDPOINT")?;
    let dsql_region = env::var("AWS_REGION")?;

    let db = create_db("lambda", &dsql_endpoint, &dsql_region).await?;

    run(service_fn(|event| {
        let db = db.clone();
        async move { function_handler(&db, event).await }
    }))
    .await
}
