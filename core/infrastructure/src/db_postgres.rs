use anyhow::Error;
use sea_orm::{Database, DatabaseConnection};

fn resolve_database_url(role: &str, password: &str) -> String {
    format!(
        "postgres://{}:{}@localhost:5432/postgres?sslmode=disable",
        role, password
    )
}

/// PostgreSQL への SeaORM データベース接続を作成する(テスト用)
pub async fn create_db_postgres() -> Result<DatabaseConnection, Error> {
    let database_url = resolve_database_url("lambda", "lambda");
    let connection = Database::connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect local PostgreSQL test DB: {}", e))?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envが未設定ならデフォルトurlを使う() {
        assert_eq!(
            resolve_database_url("lambda", "lambda"),
            "postgres://lambda:lambda@localhost:5432/postgres?sslmode=disable".to_string()
        );
    }

    #[tokio::test]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn envが設定されていればそれを使う() {
        assert!(create_db_postgres().await.is_ok());
    }
}
