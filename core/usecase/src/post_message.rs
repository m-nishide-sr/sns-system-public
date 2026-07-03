use chrono::Utc;
use core_common::{Clock, CoreResult};
use core_domain::{MessageBody, MessageRepository, NewMessage, UserId, UserName};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// メッセージ投稿ユースケースの入力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostMessageInput {
    /// JWTのsub。
    pub user_id: Uuid,
    /// JWTから導出した投稿者名。
    pub user_name: String,
    /// 投稿本文。
    pub body: String,
    /// 監査・障害調査用途の生ログ。
    pub row_log: String,
    /// ユーザー投稿かどうか。
    pub is_from_user: bool,
}

/// メッセージ投稿ユースケースの出力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostMessageOutput {
    /// 処理結果。
    pub status: String,
    /// 利用者向けメッセージ。
    pub message: String,
}

/// メッセージ投稿を実行するアプリケーションサービス。
pub struct PostMessageUseCase<R: MessageRepository, C: Clock> {
    repository: R,
    clock: C,
}

impl<R: MessageRepository, C: Clock> PostMessageUseCase<R, C> {
    /// リポジトリと時刻プロバイダを受け取りユースケースを生成する。
    pub fn new(repository: R, clock: C) -> Self {
        Self { repository, clock }
    }

    /// メッセージを作成して保存する。
    pub async fn execute(&self, input: PostMessageInput) -> CoreResult<PostMessageOutput> {
        let user_name = UserName::new(input.user_name)?;
        let body = MessageBody::new(input.body)?;

        self.repository
            .create(NewMessage {
                user_name,
                user_id: UserId::new(input.user_id),
                body,
                created_at: self.clock.now().with_timezone(&Utc),
                is_from_user: input.is_from_user,
                row_log: input.row_log,
            })
            .await?;

        Ok(PostMessageOutput {
            status: "success".to_string(),
            message: "Message created successfully".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::{DateTime, TimeZone};
    use core_common::{Clock, CoreError};
    use core_domain::{MessageRepository, NewMessage, TimelineMessage};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct FixedClock {
        now: DateTime<Utc>,
    }

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.now
        }
    }

    #[derive(Clone, Default)]
    struct RecordingRepository {
        saved: Arc<Mutex<Vec<NewMessage>>>,
    }

    #[async_trait]
    impl MessageRepository for RecordingRepository {
        async fn list_latest(
            &self,
            _before: Option<DateTime<Utc>>,
            _limit: u64,
        ) -> CoreResult<Vec<TimelineMessage>> {
            Ok(Vec::new())
        }

        async fn create(&self, input: NewMessage) -> CoreResult<()> {
            self.saved
                .lock()
                .expect("mutexのロックに失敗しない想定")
                .push(input);
            Ok(())
        }
    }

    #[tokio::test]
    async fn 正常入力なら保存して成功レスポンスを返す() {
        let repository = RecordingRepository::default();
        let usecase = PostMessageUseCase::new(
            repository.clone(),
            FixedClock {
                now: Utc
                    .with_ymd_and_hms(2026, 7, 1, 0, 0, 0)
                    .single()
                    .expect("固定時刻の生成に失敗しない想定"),
            },
        );

        let output = usecase
            .execute(PostMessageInput {
                user_id: Uuid::now_v7(),
                user_name: "taro".to_string(),
                body: "こんにちは".to_string(),
                row_log: "request-id=test".to_string(),
                is_from_user: true,
            })
            .await
            .expect("正常系で失敗しない想定");

        assert_eq!(output.status, "success");
        assert_eq!(
            repository
                .saved
                .lock()
                .expect("mutexのロックに失敗しない想定")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn 本文が空ならバリデーションエラー() {
        let repository = RecordingRepository::default();
        let usecase = PostMessageUseCase::new(repository, FixedClock { now: Utc::now() });

        let err = usecase
            .execute(PostMessageInput {
                user_id: Uuid::now_v7(),
                user_name: "taro".to_string(),
                body: " ".to_string(),
                row_log: "request-id=test".to_string(),
                is_from_user: true,
            })
            .await
            .expect_err("本文空文字はエラー想定");

        assert!(matches!(err, CoreError::Validation(_)));
    }
}
