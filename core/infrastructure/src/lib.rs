//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod db;
pub mod db_dsql;
pub mod db_postgres;

pub use crate::db_dsql::AuroraDSQLConnectionInfo;
pub use crate::db_postgres::PostgreSQLConnectionInfo;
pub use db::{DBType, create_db};
pub use sea_orm_message_repository::SeaOrmMessageRepository;
