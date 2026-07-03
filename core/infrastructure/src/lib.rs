//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod sea_orm_message_repository;

pub use sea_orm_message_repository::SeaOrmMessageRepository;
