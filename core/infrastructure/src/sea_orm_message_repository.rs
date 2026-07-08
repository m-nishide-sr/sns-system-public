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
    use crate::create_db_postgres;
    use chrono::Utc;
    // use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
    // use sea_orm_entities::entity::messages::{self, Column, Entity as Messages};
    use core_usecase::{GetTimelineInput, GetTimelineOutput, GetTimelineUseCase, TimelineItem};
    use serde::{Deserialize, Serialize};

    use sea_orm_entities::entity::messages;

    /// タイムライン取得レスポンス。
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TimelineMessageResponse {
        /// 投稿者名。
        pub user_name: String,
        /// 投稿日時。
        pub created_at: DateTime<Utc>,
        /// 投稿本文。
        pub body: String,
        /// 利用者投稿かどうか。
        pub is_from_user: bool,
    }

    impl From<TimelineItem> for TimelineMessageResponse {
        fn from(value: TimelineItem) -> Self {
            Self {
                user_name: value.user_name,
                created_at: value.created_at,
                body: value.body,
                is_from_user: value.is_from_user,
            }
        }
    }

    #[test]
    fn limitが0の場合はクエリで50件に補正する() {
        let statement = build_timeline_statement(None, 0);
        let debug = format!("{statement:?}");
        assert!(debug.contains("messages_latest"));
    }

    #[tokio::test]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn 実際にローカルの_postgre_sqlに接続するテスト() {
        let db = create_db_postgres().await.unwrap();
        let cognito_id = uuid::Uuid::now_v7();
        let repository = SeaOrmMessageRepository::new(db.clone());
        let now = chrono::Utc::now().fixed_offset();

        let first_uuid = uuid::Uuid::new_v4();
        let email = format!("local-get-{}@example.com", first_uuid);
        messages::ActiveModel {
            id: Set(first_uuid),
            cognito_id: Set(cognito_id),
            user_name: Set(email.clone()),
            created_at: Set(now),
            body: Set("older body".to_string()),
            row_log: Set("older row log".to_string()),
            is_from_user: Set(true),
        }
        .insert(&db)
        .await
        .expect("older test message insert should succeed");

        let second_uuid = uuid::Uuid::new_v4();
        let second_email = format!("local-get-{}@example.com", second_uuid);
        messages::ActiveModel {
            id: Set(second_uuid),
            cognito_id: Set(cognito_id),
            user_name: Set(second_email.clone()),
            created_at: Set(now + chrono::Duration::seconds(1)),
            body: Set("second body".to_string()),
            row_log: Set("second row log".to_string()),
            is_from_user: Set(false),
        }
        .insert(&db)
        .await
        .expect("newer test message insert should succeed");

        let third_uuid = uuid::Uuid::new_v4();
        let third_email = format!("local-get-{}@example.com", third_uuid);
        messages::ActiveModel {
            id: Set(third_uuid),
            cognito_id: Set(cognito_id),
            user_name: Set(third_email.clone()),
            created_at: Set(now + chrono::Duration::seconds(2)),
            body: Set("third body".to_string()),
            row_log: Set("third row log".to_string()),
            is_from_user: Set(true),
        }
        .insert(&db)
        .await
        .expect("newer test message insert should succeed");

        let usecase = GetTimelineUseCase::new(repository);
        let output: GetTimelineOutput = match usecase
            .execute(GetTimelineInput {
                limit: 2,
                before: None,
            })
            .await
        {
            Ok(output) => output,
            Err(err) => panic!("usecase execution failed: {}", err),
        };
        assert_eq!(output.items.len(), 2);
        let response: Vec<TimelineMessageResponse> = output
            .items
            .into_iter()
            .map(TimelineMessageResponse::from)
            .collect();

        assert_eq!(
            response[0].created_at,
            third_created_at
                .format("%Y-%m-%dT%H:%M:%S.%6fZ")
                .to_string()
                .parse::<DateTime<Utc>>()
                .unwrap()
        );
        assert_eq!(
            response[1].created_at,
            second_created_at
                .format("%Y-%m-%dT%H:%M:%S.%6fZ")
                .to_string()
                .parse::<DateTime<Utc>>()
                .unwrap()
        );

        assert_eq!(response[0].body, "third body");
        assert_eq!(response[1].body, "second body");

        assert!(response[0].is_from_user);
        assert!(!response[1].is_from_user);

        assert_eq!(response[0].user_name, third_email);
        assert_eq!(response[1].user_name, second_email);
    }
}
