//! チャットメッセージ投稿ユースケース。

use crate::application::repository::MessageRepository;
use crate::domain::message::{CognitoId, MessageBody, MessageId};
use crate::error::UseCaseError;

/// チャットメッセージを投稿するユースケース。
pub struct PostMessage<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> PostMessage<R> {
    /// 新しいユースケースを生成する。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// チャットメッセージを投稿する。
    ///
    /// メッセージIDはUUID v7で生成される。
    pub async fn execute(
        &self,
        cognito_id: CognitoId,
        body: MessageBody,
    ) -> Result<(), UseCaseError> {
        let id = MessageId::new();
        self.repository.save_message(id, &cognito_id, &body).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::{Message, MessageBody};
    use crate::error::RepositoryError;
    use std::sync::{Arc, Mutex};

    struct MockRepository {
        saved: Arc<Mutex<Vec<(MessageId, String)>>>,
    }

    impl MessageRepository for MockRepository {
        async fn get_messages(
            &self,
            _cognito_id: &CognitoId,
        ) -> Result<Vec<Message>, RepositoryError> {
            Ok(vec![])
        }

        async fn save_message(
            &self,
            id: MessageId,
            _cognito_id: &CognitoId,
            body: &MessageBody,
        ) -> Result<(), RepositoryError> {
            self.saved
                .lock()
                .unwrap()
                .push((id, body.as_str().to_string()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn saves_message_with_valid_inputs() {
        let cognito_id = "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d"
            .parse::<CognitoId>()
            .unwrap();
        let body = MessageBody::new("テストメッセージ").unwrap();
        let saved = Arc::new(Mutex::new(vec![]));
        let mock = MockRepository {
            saved: saved.clone(),
        };
        let use_case = PostMessage::new(mock);
        use_case.execute(cognito_id, body).await.unwrap();
        assert_eq!(saved.lock().unwrap().len(), 1);
        assert_eq!(saved.lock().unwrap()[0].1, "テストメッセージ");
    }
}
