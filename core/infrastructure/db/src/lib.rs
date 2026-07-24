//! インフラ層(DB)の公開API。
//!
//! DB接続(SeaORMのDatabaseConnection)生成など、DB周りの共通機能を提供する。

pub mod db;
pub mod db_dsql;
pub mod db_postgres;

pub use crate::db_dsql::AuroraDSQLConnectionInfo;
pub use crate::db_postgres::PostgreSQLConnectionInfo;
pub use db::{DBType, create_db};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 公開re_exportが参照できる() {
        let _create_db = create_db::<&str>;
        let _db_type_auroradsql = DBType::AuroraDSQL(AuroraDSQLConnectionInfo {
            role: "",
            endpoint: "",
            region: "",
        });
        let _db_type_postgresql = DBType::PostgreSQL(PostgreSQLConnectionInfo {
            role: "",
            password: "",
        });
    }
}
