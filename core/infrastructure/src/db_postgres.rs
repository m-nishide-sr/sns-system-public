use anyhow::Error;
use sea_orm::{Database, DatabaseConnection};

fn resolve_database_url() -> String {
    std::env::var("LOCAL_TEST_DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "postgres://{}:{}@localhost:5432/postgres?sslmode=disable",
            "lambda", "lambda"
        )
    })
}

/// PostgreSQL への SeaORM データベース接続を作成する(テスト用)
pub async fn create_db_postgres() -> Result<DatabaseConnection, Error> {
    let database_url = resolve_database_url();
    let connection = Database::connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect local PostgreSQL test DB: {}", e))?;
    Ok(connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    fn envが未設定ならデフォルトurlを使う() {
        let _guard = ENV_LOCK.lock().expect("mutexのロックに失敗しない想定");
        unsafe {
            std::env::remove_var("LOCAL_TEST_DATABASE_URL");
        }

        let url = resolve_database_url();
        assert!(url.starts_with("postgres://"));
        assert!(url.contains("@localhost:5432/postgres?sslmode=disable"));
    }

    #[test]
    fn envが設定されていればそれを使う() {
        let _guard = ENV_LOCK.lock().expect("mutexのロックに失敗しない想定");
        unsafe {
            std::env::set_var("LOCAL_TEST_DATABASE_URL", "postgres://example");
        }

        assert_eq!(resolve_database_url(), "postgres://example".to_string());
        unsafe {
            std::env::remove_var("LOCAL_TEST_DATABASE_URL");
        }
    }
}
