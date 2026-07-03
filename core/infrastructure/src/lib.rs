//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod db;
pub mod sea_orm_message_repository;

pub use db::create_db;
pub use sea_orm_message_repository::SeaOrmMessageRepository;
