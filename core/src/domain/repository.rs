use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    common::error::CoreError, domain::message::CreateMessageCommand,
    domain::message::TimelineMessage,
};

/// メッセージ永続化と参照を抽象化するリポジトリ。
#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// タイムラインを新しい順で取得する。
    async fn get_timeline(
        &self,
        before: Option<DateTime<Utc>>,
        limit: u64,
    ) -> Result<Vec<TimelineMessage>, CoreError>;

    /// 新規投稿を保存する。
    async fn create_message(&self, command: &CreateMessageCommand) -> Result<(), CoreError>;
}
