//! ドメイン層の公開API。
//!
//! ここでは業務上の不変条件を型として表現し、
//! 不正状態をコンパイル時・生成時の両方で抑止する。

pub mod message;
pub mod repository;

pub use message::{MessageBody, NewMessage, TimelineMessage, UserId, UserName};
pub use repository::MessageRepository;
