//! チャットメッセージ取得ユースケース。

use crate::application::repository::MessageRepository;
use crate::domain::message::{CognitoId, Message};
use crate::error::UseCaseError;

/// 指定ユーザーのチャットメッセージを取得するユースケース。
pub struct GetMessages<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> GetMessages<R> {
    /// 新しいユースケースを生成する。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 指定ユーザーのチャットメッセージを取得する。
    ///
    /// メッセージは作成日時の降順で返される。
    pub async fn execute(&self, cognito_id: CognitoId) -> Result<Vec<Message>, UseCaseError> {
        let messages = self.repository.get_messages(&cognito_id).await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::message::{Message, MessageBody};
    use crate::error::RepositoryError;
    use chrono::Utc;
    use uuid::Uuid;

    struct MockRepository {
        messages: Vec<Message>,
    }

    impl MessageRepository for MockRepository {
        async fn get_messages(
            &self,
            _cognito_id: &CognitoId,
        ) -> Result<Vec<Message>, RepositoryError> {
            Ok(self.messages.clone())
        }

        async fn save_message(
            &self,
            _id: crate::domain::message::MessageId,
            _cognito_id: &CognitoId,
            _body: &MessageBody,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn returns_messages_for_valid_cognito_id() {
        let cognito_id_str = "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d";
        let cognito_id = cognito_id_str.parse::<CognitoId>().unwrap();

        let msg = Message {
            id: Uuid::now_v7(),
            is_from_user: true,
            body: "テストメッセージ".to_string(),
            created_at: Utc::now(),
        };
        let mock = MockRepository {
            messages: vec![msg.clone()],
        };
        let use_case = GetMessages::new(mock);
        let result = use_case.execute(cognito_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].body, "テストメッセージ");
    }

    #[tokio::test]
    async fn returns_empty_list_when_no_messages() {
        let cognito_id = "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d"
            .parse::<CognitoId>()
            .unwrap();
        let mock = MockRepository { messages: vec![] };
        let use_case = GetMessages::new(mock);
        let result = use_case.execute(cognito_id).await.unwrap();
        assert!(result.is_empty());
    }
}
