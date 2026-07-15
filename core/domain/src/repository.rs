use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_common::CoreResult;

use crate::{NewMessage, TimelineMessage};

/// メッセージ永続化の抽象インターフェース。
///
/// ユースケースは本traitのみに依存し、SeaORMやSQL文を知らない状態を維持する。
#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// タイムラインを新しい順に取得する。
    async fn list_latest(
        &self,
        before: Option<DateTime<Utc>>,
        limit: u64,
    ) -> CoreResult<Vec<TimelineMessage>>;

    /// メッセージを永続化する。
    async fn create(&self, input: NewMessage) -> CoreResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageBody, UserId, UserName};
    use uuid::Uuid;

    struct InMemoryRepository;

    #[async_trait]
    impl MessageRepository for InMemoryRepository {
        async fn list_latest(
            &self,
            _before: Option<DateTime<Utc>>,
            _limit: u64,
        ) -> CoreResult<Vec<TimelineMessage>> {
            Ok(vec![TimelineMessage {
                user_name: "taro".to_string(),
                created_at: Utc::now(),
                body: "hello".to_string(),
                is_from_user: true,
            }])
        }

        async fn create(&self, _input: NewMessage) -> CoreResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn trait実装経由で一覧取得できる() {
        let repo = InMemoryRepository;
        let messages = repo
            .list_latest(None, 10)
            .await
            .expect("正常系で失敗しない想定");
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn trait実装経由で作成できる() {
        let repo = InMemoryRepository;
        let result = repo
            .create(NewMessage {
                user_name: UserName::new("taro").expect("正常系で失敗しない想定"),
                user_id: UserId::new(Uuid::now_v7()),
                body: MessageBody::new("hello").expect("正常系で失敗しない想定"),
                created_at: Utc::now(),
                is_from_user: true,
                row_log: "{}".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }
}
