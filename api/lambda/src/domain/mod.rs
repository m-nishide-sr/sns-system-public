//! ドメインレイヤーモジュール。
//!
//! エンティティ・値オブジェクト・ドメインルール・ドメイン固有エラーを定義します。

pub mod message;

pub use message::{Message, NewMessage, RepositoryError};
