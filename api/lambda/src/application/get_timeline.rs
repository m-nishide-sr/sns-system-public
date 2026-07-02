//! タイムライン取得ユースケース。
//!
//! `public.messages_latest` ビューから最新メッセージを最大50件取得します。

use chrono::{DateTime, Utc};

use crate::application::MessageRepository;
use crate::domain::{Message, RepositoryError};

/// タイムライン取得ユースケース。
///
/// メッセージを `created_at` の降順で最大50件取得します。
/// `before` パラメータを指定した場合、その日時より前のメッセージのみ取得します。
pub struct GetTimelineUseCase<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> GetTimelineUseCase<R> {
    /// 新しいユースケースインスタンスを生成します。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// タイムラインを取得します。
    ///
    /// # 引数
    ///
    /// * `before` - この日時より前のメッセージを取得します（ページネーション用）。
    ///   `None` の場合は最新のメッセージから取得します。
    ///
    /// # 戻り値
    ///
    /// `created_at` の降順で最大50件のメッセージリスト。
    pub async fn execute(
        &self,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, RepositoryError> {
        self.repository.get_timeline(before).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::TimeZone;

    /// テスト用のモックRepository。
    struct MockMessageRepository {
        messages: Vec<Message>,
    }

    #[async_trait]
    impl MessageRepository for MockMessageRepository {
        async fn get_timeline(
            &self,
            before: Option<DateTime<Utc>>,
        ) -> Result<Vec<Message>, RepositoryError> {
            let filtered: Vec<Message> = self
                .messages
                .iter()
                .filter(|m| before.is_none() || m.created_at < before.unwrap())
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn post_message(
            &self,
            _new_message: crate::domain::NewMessage,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    fn make_message(user_name: &str, secs: i64, body: &str) -> Message {
        Message {
            user_name: user_name.to_string(),
            created_at: Utc.timestamp_opt(secs, 0).unwrap(),
            body: body.to_string(),
            is_from_user: true,
        }
    }

    #[tokio::test]
    async fn beforeなしで全メッセージを取得できる() {
        let messages = vec![
            make_message("user1", 1000, "メッセージ1"),
            make_message("user2", 2000, "メッセージ2"),
        ];
        let repo = MockMessageRepository {
            messages: messages.clone(),
        };
        let use_case = GetTimelineUseCase::new(repo);
        let result = use_case.execute(None).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn beforeを指定すると指定日時より前のメッセージのみ取得できる() {
        let messages = vec![
            make_message("user1", 1000, "メッセージ1"),
            make_message("user2", 2000, "メッセージ2"),
            make_message("user3", 3000, "メッセージ3"),
        ];
        let repo = MockMessageRepository { messages };
        let use_case = GetTimelineUseCase::new(repo);
        let before = Utc.timestamp_opt(2500, 0).unwrap();
        let result = use_case.execute(Some(before)).await.unwrap();
        assert_eq!(result.len(), 2);
    }
}
