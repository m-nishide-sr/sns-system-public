//! ユースケース層。
//!
//! 入出力DTOと処理手順を定義し、ドメインモデルを呼び出して業務要件を実現する。

pub mod post_message;
pub mod timeline;

pub use post_message::{PostMessageInput, PostMessageOutput, PostMessageUseCase};
pub use timeline::{GetTimelineInput, GetTimelineOutput, GetTimelineUseCase, TimelineItem};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn 公開re_exportのdtoを生成できる() {
        let post = PostMessageInput {
            user_id: Uuid::now_v7(),
            user_name: "taro".to_string(),
            body: "hello".to_string(),
            row_log: "{}".to_string(),
            is_from_user: true,
        };
        let timeline = GetTimelineInput {
            before: Some(Utc::now()),
            limit: 10,
        };

        assert_eq!(post.user_name, "taro");
        assert_eq!(timeline.limit, 10);
    }
}
