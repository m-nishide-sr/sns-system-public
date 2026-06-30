//! データベース接続を管理するモジュール。
//!
//! ローカル環境では環境変数`DATABASE_URL`を使用し、
//! AWS環境では環境変数`DSQL_ENDPOINT`を使用してAurora DSQLに接続する。

use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use std::env;

use crate::error::RepositoryError;

/// データベース接続を確立する。
///
/// 環境変数`DSQL_ENDPOINT`が設定されている場合はAurora DSQLに接続し、
/// そうでない場合は環境変数`DATABASE_URL`を使用して接続する。
pub async fn connect() -> Result<DatabaseConnection, RepositoryError> {
    if let Ok(dsql_endpoint) = env::var("DSQL_ENDPOINT") {
        connect_dsql(&dsql_endpoint).await
    } else {
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            RepositoryError::Database(sea_orm::DbErr::Custom(
                "環境変数 DATABASE_URL または DSQL_ENDPOINT が設定されていません".into(),
            ))
        })?;
        connect_postgres(&database_url).await
    }
}

/// Aurora DSQLに接続する。
///
/// `aurora-dsql-sqlx-connector`を使用してIAM認証でAurora DSQLに接続する。
async fn connect_dsql(endpoint: &str) -> Result<DatabaseConnection, RepositoryError> {
    let region = env::var("AWS_REGION").unwrap_or_else(|_| "ap-northeast-3".to_string());
    let conn_str = format!("postgres://admin@{endpoint}/postgres?region={region}&sslmode=require");
    let pool = aurora_dsql_sqlx_connector::pool::connect(&conn_str)
        .await
        .map_err(|e| {
            RepositoryError::Database(sea_orm::DbErr::Custom(format!(
                "Aurora DSQL接続エラー: {e}"
            )))
        })?;
    Ok(SqlxPostgresConnector::from_sqlx_postgres_pool(pool))
}

/// 標準PostgreSQLに接続する（ローカル開発・テスト用）。
async fn connect_postgres(database_url: &str) -> Result<DatabaseConnection, RepositoryError> {
    sea_orm::Database::connect(database_url)
        .await
        .map_err(RepositoryError::Database)
}
