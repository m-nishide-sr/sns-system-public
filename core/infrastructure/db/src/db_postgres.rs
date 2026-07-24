use anyhow::Error;
use sea_orm::{Database, sqlx::PgPool};

fn resolve_database_url(role: &str, password: &str) -> String {
    format!(
        "postgres://{}:{}@localhost:5432/postgres?sslmode=disable",
        role, password
    )
}

/// # PostgreSQLConnectionInfo
///
/// * `role` - PostgreSQL のデータベースロール名（例: `"lambda"`)
/// * `password` - PostgreSQL のデータベースパスワード（例: `"lambda"`)
pub struct PostgreSQLConnectionInfo<T>
where
    T: AsRef<str>,
{
    /// データベースロール名（例: `"lambda"`, `"selectview"`）
    pub role: T,
    /// データベースパスワード
    pub password: T,
}

/// PostgreSQL への SeaORM データベース接続を作成する(テスト用)
pub async fn create_db_postgres<T>(
    database_connection_info: &PostgreSQLConnectionInfo<T>,
) -> Result<PgPool, Error>
where
    T: AsRef<str>,
{
    let database_url = resolve_database_url(
        database_connection_info.role.as_ref(),
        database_connection_info.password.as_ref(),
    );
    let connection = Database::connect(database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect local PostgreSQL test DB: {}", e))?;
    // SeaORM の接続から sqlx の PgPool を取り出す
    let pool: &PgPool = connection.get_postgres_connection_pool();

    // 所有権を持った Pool を返すためにクローン（sqlx::Pool のクローンは安価です）
    Ok(pool.clone())
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

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn 明示的にローカル_postgre_sqlへ接続できること() {
        assert!(
            create_db_postgres(&PostgreSQLConnectionInfo {
                role: "lambda",
                password: "lambda",
            })
            .await
            .is_ok()
        );
    }
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn ローカル_postgre_sqlへ接続できない時にエラーになること() {
        assert!(
            create_db_postgres(&PostgreSQLConnectionInfo {
                role: "invalid_role",
                password: "invalid_password",
            })
            .await
            .is_err()
        );
    }
}
