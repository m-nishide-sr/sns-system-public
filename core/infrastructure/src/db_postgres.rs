use anyhow::Error;
use sea_orm::{Database, DatabaseConnection};

/// PostgreSQL への SeaORM データベース接続を作成する(テスト用)
pub async fn create_db_postgres() -> Result<DatabaseConnection, Error> {
    let database_url = std::env::var("LOCAL_TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/postgres?sslmode=disable".to_string()
    });
    let connection = Database::connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect local PostgreSQL test DB: {}", e))?;
    Ok(connection)
}
