use crate::{
    common::error::CoreError,
    domain::{
        message::{CreateMessageCommand, MessageBody},
        repository::MessageRepository,
    },
};

/// 投稿ユースケースの入力DTO。
#[derive(Debug, Clone)]
pub struct PostMessageInput {
    /// 本文。
    pub body: String,
    /// ユーザー名。
    pub user_name: String,
    /// CognitoサブジェクトID。
    pub cognito_id: String,
    /// 調査用途ログ。
    pub row_log: String,
}

/// 投稿ユースケースの出力DTO。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostMessageOutput {
    /// APIレスポンスの状態。
    pub status: &'static str,
    /// APIレスポンスのメッセージ。
    pub message: &'static str,
}

/// 投稿ユースケース。
pub struct PostMessageUsecase<'a, R: MessageRepository> {
    repository: &'a R,
}

impl<'a, R: MessageRepository> PostMessageUsecase<'a, R> {
    /// ユースケースを初期化する。
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// 入力値を検証し、投稿を保存する。
    pub async fn execute(&self, input: PostMessageInput) -> Result<PostMessageOutput, CoreError> {
        let body = MessageBody::new(input.body)?;
        let command = CreateMessageCommand {
            user_name: input.user_name,
            cognito_id: input.cognito_id,
            body,
            row_log: input.row_log,
            is_from_user: true,
        };

        self.repository.create_message(&command).await?;

        Ok(PostMessageOutput {
            status: "success",
            message: "Message created successfully",
        })
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::{
        common::error::CoreError,
        domain::{message::TimelineMessage, repository::MessageRepository},
    };

    struct DummyRepository;

    #[async_trait]
    impl MessageRepository for DummyRepository {
        async fn get_timeline(
            &self,
            _before: Option<chrono::DateTime<chrono::Utc>>,
            _limit: u64,
        ) -> Result<Vec<TimelineMessage>, CoreError> {
            Ok(vec![])
        }

        async fn create_message(
            &self,
            command: &crate::domain::message::CreateMessageCommand,
        ) -> Result<(), CoreError> {
            if command.user_name.is_empty() {
                return Err(CoreError::Validation("user_nameは必須です".to_owned()));
            }
            Ok(())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn 有効な入力で成功レスポンスを返す() {
        let repository = DummyRepository;
        let usecase = PostMessageUsecase::new(&repository);

        let output = usecase
            .execute(PostMessageInput {
                body: "テスト投稿".to_owned(),
                user_name: "tanaka".to_owned(),
                cognito_id: "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d".to_owned(),
                row_log: "{}".to_owned(),
            })
            .await
            .expect("正常系は成功するべき");

        assert_eq!(
            output,
            PostMessageOutput {
                status: "success",
                message: "Message created successfully"
            }
        );
    }
}
