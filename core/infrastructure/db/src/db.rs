use crate::db_dsql::{AuroraDSQLConnectionInfo, create_db_dsql};
use crate::db_postgres::{PostgreSQLConnectionInfo, create_db_postgres};
use anyhow::Error;
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};

pub enum DBType<T>
where
    T: AsRef<str>,
{
    AuroraDSQL(AuroraDSQLConnectionInfo<T>),
    PostgreSQL(PostgreSQLConnectionInfo<T>),
}

pub async fn create_db<T>(db_type: DBType<T>) -> Result<DatabaseConnection, Error>
where
    T: AsRef<str>,
{
    let pool = match db_type {
        DBType::AuroraDSQL(conn_info) => create_db_dsql(&conn_info)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Aurora DSQL: {}", e))?,
        DBType::PostgreSQL(conn_info) => create_db_postgres(&conn_info)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to PostgreSQL: {}", e))?,
    };

    Ok(SqlxPostgresConnector::from_sqlx_postgres_pool(pool))
}
