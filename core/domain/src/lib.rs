//! ドメイン層の公開API。
//!
//! ここでは業務上の不変条件を型として表現し、
//! 不正状態をコンパイル時・生成時の両方で抑止する。

pub mod auth;
pub mod message;
pub mod repository;

pub use auth::{AuthRepository, DomainError, PreSignUpInput, PreSignUpOutput};
pub use message::{MessageBody, NewMessage, TimelineMessage, UserId, UserName};
pub use repository::MessageRepository;

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn 公開re_exportを利用できる() {
        let body = MessageBody::new("hello").expect("正常系で失敗しない想定");
        let user_name = UserName::new("taro").expect("正常系で失敗しない想定");
        let user_id = UserId::new(Uuid::now_v7());

        assert_eq!(body.as_str(), "hello");
        assert_eq!(user_name.as_str(), "taro");
        assert_ne!(user_id.into_inner(), Uuid::nil());
    }
}
