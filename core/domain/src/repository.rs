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
