//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod db;
pub mod db_dsql;
pub mod db_postgres;
pub mod sea_orm_message_repository;

pub use crate::db_dsql::AuroraDSQLConnectionInfo;
pub use crate::db_postgres::PostgreSQLConnectionInfo;
pub use db::{DBType, create_db};
pub use sea_orm_message_repository::SeaOrmMessageRepository;

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
        let _repo_ctor: fn(sea_orm::prelude::DatabaseConnection) -> SeaOrmMessageRepository =
            SeaOrmMessageRepository::new;
    }
}
