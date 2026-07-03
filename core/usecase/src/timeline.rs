use chrono::{DateTime, Utc};
use core_common::CoreResult;
use core_domain::MessageRepository;
use serde::{Deserialize, Serialize};

/// タイムライン取得ユースケースの入力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetTimelineInput {
    /// 指定時刻より前の投稿だけを取得するための境界。
    pub before: Option<DateTime<Utc>>,
    /// 取得件数。未指定相当の`0`は最大件数として扱う。
    pub limit: u64,
}

/// タイムライン1件分の出力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineItem {
    /// 投稿者名。
    pub user_name: String,
    /// 投稿日時(UTC)。
    pub created_at: DateTime<Utc>,
    /// 本文。
    pub body: String,
    /// ユーザー投稿かどうか。
    pub is_from_user: bool,
}

/// タイムライン取得ユースケースの出力。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetTimelineOutput {
    /// 取得結果一覧。
    pub items: Vec<TimelineItem>,
}

/// タイムラインを取得するアプリケーションサービス。
pub struct GetTimelineUseCase<R: MessageRepository> {
    repository: R,
}

impl<R: MessageRepository> GetTimelineUseCase<R> {
    /// リポジトリ実装を注入してユースケースを生成する。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// タイムラインを取得する。
    pub async fn execute(&self, input: GetTimelineInput) -> CoreResult<GetTimelineOutput> {
        let limit = normalize_limit(input.limit);
        let items = self
            .repository
            .list_latest(input.before, limit)
            .await?
            .into_iter()
            .map(|m| TimelineItem {
                user_name: m.user_name,
                created_at: m.created_at,
                body: m.body,
                is_from_user: m.is_from_user,
            })
            .collect();

        Ok(GetTimelineOutput { items })
    }
}

fn normalize_limit(limit: u64) -> u64 {
    if limit == 0 || limit > 50 { 50 } else { limit }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use core_common::CoreResult;
    use core_domain::{MessageRepository, NewMessage, TimelineMessage};

    struct FakeRepository;

    #[async_trait]
    impl MessageRepository for FakeRepository {
        async fn list_latest(
            &self,
            _before: Option<DateTime<Utc>>,
            limit: u64,
        ) -> CoreResult<Vec<TimelineMessage>> {
            Ok(vec![TimelineMessage {
                user_name: format!("user-{limit}"),
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
    async fn limitが0の場合は50件扱いになる() {
        let usecase = GetTimelineUseCase::new(FakeRepository);
        let output = usecase
            .execute(GetTimelineInput {
                before: None,
                limit: 0,
            })
            .await
            .expect("正常系で失敗しない想定");

        assert_eq!(output.items[0].user_name, "user-50");
    }
}
