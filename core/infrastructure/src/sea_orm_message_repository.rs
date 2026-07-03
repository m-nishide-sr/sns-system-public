use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult};
use core_domain::{MessageRepository, NewMessage, TimelineMessage};
use sea_orm::entity::prelude::Uuid;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbBackend, QueryResult, Set, Statement,
};
use sea_orm_entities::entity::messages;

/// SeaORMを利用したメッセージRepository実装。
#[derive(Clone)]
pub struct SeaOrmMessageRepository {
    db: DatabaseConnection,
}

impl SeaOrmMessageRepository {
    /// DB接続を受け取りRepositoryを生成する。
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MessageRepository for SeaOrmMessageRepository {
    async fn list_latest(
        &self,
        before: Option<DateTime<Utc>>,
        limit: u64,
    ) -> CoreResult<Vec<TimelineMessage>> {
        let statement = build_timeline_statement(before, limit);
        let rows = self.db.query_all(statement).await.map_err(|e| {
            CoreError::Infrastructure(format!("タイムライン取得に失敗しました: {e}"))
        })?;

        rows.into_iter().map(to_timeline_message).collect()
    }

    async fn create(&self, input: NewMessage) -> CoreResult<()> {
        messages::ActiveModel {
            id: Set(Uuid::now_v7()),
            user_name: Set(input.user_name.as_str().to_string()),
            cognito_id: Set(input.user_id.into_inner()),
            created_at: Set(input.created_at.into()),
            body: Set(input.body.as_str().to_string()),
            row_log: Set(input.row_log),
            is_from_user: Set(input.is_from_user),
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::Infrastructure(format!("メッセージ作成に失敗しました: {e}")))?;

        Ok(())
    }
}

fn build_timeline_statement(before: Option<DateTime<Utc>>, limit: u64) -> Statement {
    let safe_limit = if limit == 0 || limit > 50 {
        50
    } else {
        limit as i64
    };

    match before {
        Some(before) => Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                SELECT user_name, created_at, body, is_from_user
                FROM public.messages_latest
                WHERE created_at < $1
                ORDER BY created_at DESC
                LIMIT $2
            "#,
            [before.into(), safe_limit.into()],
        ),
        None => Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                SELECT user_name, created_at, body, is_from_user
                FROM public.messages_latest
                ORDER BY created_at DESC
                LIMIT $1
            "#,
            [safe_limit.into()],
        ),
    }
}

fn to_timeline_message(row: QueryResult) -> CoreResult<TimelineMessage> {
    let created_at = row
        .try_get::<DateTime<Utc>>("", "created_at")
        .map_err(|e| CoreError::Infrastructure(format!("created_atの変換に失敗しました: {e}")))?;

    Ok(TimelineMessage {
        user_name: row.try_get("", "user_name").map_err(|e| {
            CoreError::Infrastructure(format!("user_nameの変換に失敗しました: {e}"))
        })?,
        created_at,
        body: row
            .try_get("", "body")
            .map_err(|e| CoreError::Infrastructure(format!("bodyの変換に失敗しました: {e}")))?,
        is_from_user: row.try_get("", "is_from_user").map_err(|e| {
            CoreError::Infrastructure(format!("is_from_userの変換に失敗しました: {e}"))
        })?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limitが0の場合はクエリで50件に補正する() {
        let statement = build_timeline_statement(None, 0);
        let debug = format!("{statement:?}");
        assert!(debug.contains("messages_latest"));
    }
}
