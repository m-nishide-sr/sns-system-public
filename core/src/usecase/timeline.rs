use chrono::{DateTime, Utc};

use crate::{
    common::error::CoreError,
    domain::{message::TimelineMessage, repository::MessageRepository},
};

const TIMELINE_LIMIT: u64 = 50;

/// タイムライン取得ユースケース。
pub struct GetTimelineUsecase<'a, R: MessageRepository> {
    repository: &'a R,
}

impl<'a, R: MessageRepository> GetTimelineUsecase<'a, R> {
    /// ユースケースを初期化する。
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// before条件付きで最新50件を取得する。
    pub async fn execute(
        &self,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<TimelineMessage>, CoreError> {
        self.repository.get_timeline(before, TIMELINE_LIMIT).await
    }
}
