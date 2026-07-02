//! インフラストラクチャレイヤーモジュール。
//!
//! DBなど外部依存の具体実装を提供します。

pub mod dsql_message_repository;

pub use dsql_message_repository::DsqlMessageRepository;
