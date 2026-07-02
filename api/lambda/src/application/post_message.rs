//! メッセージ投稿ユースケース。
//!
//! `public.messages` テーブルに新規メッセージを挿入します。

use crate::application::MessageRepository;
use crate::domain::{NewMessage, RepositoryError};

/// メッセージ投稿ユースケース。
pub struct PostMessageUseCase<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> PostMessageUseCase<R> {
    /// 新しいユースケースインスタンスを生成します。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// メッセージを投稿します。
    ///
    /// # 引数
    ///
    /// * `new_message` - 投稿するメッセージのデータ。
    pub async fn execute(&self, new_message: NewMessage) -> Result<(), RepositoryError> {
        self.repository.post_message(new_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use crate::domain::Message;

    /// テスト用のモックRepository（投稿されたメッセージを記録する）。
    struct MockMessageRepository {
        posted: Arc<Mutex<Vec<NewMessage>>>,
    }

    #[async_trait]
    impl MessageRepository for MockMessageRepository {
        async fn get_timeline(
            &self,
            _before: Option<DateTime<Utc>>,
        ) -> Result<Vec<Message>, RepositoryError> {
            Ok(vec![])
        }

        async fn post_message(&self, new_message: NewMessage) -> Result<(), RepositoryError> {
            self.posted.lock().unwrap().push(new_message);
            Ok(())
        }
    }

    #[tokio::test]
    async fn メッセージを投稿できる() {
        let posted = Arc::new(Mutex::new(Vec::new()));
        let repo = MockMessageRepository {
            posted: Arc::clone(&posted),
        };
        let use_case = PostMessageUseCase::new(repo);

        let new_message = NewMessage {
            body: "テストメッセージ".to_string(),
            user_name: "testuser".to_string(),
            cognito_id: Uuid::new_v4(),
            row_log: r#"{"test": true}"#.to_string(),
            is_from_user: true,
        };

        use_case.execute(new_message).await.unwrap();
        assert_eq!(posted.lock().unwrap().len(), 1);
    }
}
