use async_trait::async_trait;
use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult};
use core_domain::{MessageRepository, NewMessage, TimelineMessage};
use sea_orm::entity::prelude::Uuid;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, QueryResult, Statement};

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
            tracing::error!("タイムライン取得に失敗しました: {e}");
            CoreError::Infrastructure(format!("タイムライン取得に失敗しました: {e}"))
        })?;

        rows.into_iter().map(to_timeline_message).collect()
    }

    async fn create(&self, input: NewMessage) -> CoreResult<()> {
        // Statementの組み立て
        let statement = build_create_message_statement(input);

        self.db.query_all(statement).await.map_err(|e| {
            tracing::error!("メッセージ作成に失敗しました: {e}");
            CoreError::Infrastructure(format!("メッセージ作成に失敗しました: {e}"))
        })?;

        Ok(())
    }
}

fn build_create_message_statement(input: NewMessage) -> Statement {
    // 挿入する値を準備
    let id = Uuid::now_v7();
    let user_name = input.user_name.as_str().to_string();
    let cognito_id = input.user_id.into_inner();
    let created_at: chrono::DateTime<chrono::FixedOffset> = input.created_at.into(); // 型は環境に合わせて調整してください
    let body = input.body.as_str().to_string();
    let row_log = input.row_log;
    let is_from_user = input.is_from_user;

    Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"
                INSERT INTO public.messages (id, user_name, cognito_id, created_at, body, row_log, is_from_user)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        [
            id.into(),
            user_name.into(),
            cognito_id.into(),
            created_at.into(),
            body.into(),
            row_log.into(),
            is_from_user.into(),
        ],
    )
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
        .map_err(|e| {
            tracing::error!("created_atの変換に失敗しました: {e}");
            CoreError::Infrastructure(format!("created_atの変換に失敗しました: {e}"))
        })?;

    Ok(TimelineMessage {
        user_name: row.try_get("", "user_name").map_err(|e| {
            tracing::error!("user_nameの変換に失敗しました: {e}");
            CoreError::Infrastructure(format!("user_nameの変換に失敗しました: {e}"))
        })?,
        created_at,
        body: row.try_get("", "body").map_err(|e| {
            tracing::error!("bodyの変換に失敗しました: {e}");
            CoreError::Infrastructure(format!("bodyの変換に失敗しました: {e}"))
        })?,
        is_from_user: row.try_get("", "is_from_user").map_err(|e| {
            tracing::error!("is_from_userの変換に失敗しました: {e}");
            CoreError::Infrastructure(format!("is_from_userの変換に失敗しました: {e}"))
        })?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_db_postgres;
    use chrono::Utc;
    use core_common::Clock;
    // use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};
    // use sea_orm_entities::entity::messages::{self, Column, Entity as Messages};
    use core_usecase::{
        GetTimelineInput, GetTimelineOutput, GetTimelineUseCase, PostMessageInput,
        PostMessageUseCase, TimelineItem,
    };
    use serde::{Deserialize, Serialize};

    use sea_orm_entities::entity::messages;

    // テスト用の時刻固定モック構造体
    struct MockClock {
        fixed_time: DateTime<Utc>,
    }

    impl MockClock {
        /// 指定した初期時刻でモックを作成する
        pub fn new(initial_time: DateTime<Utc>) -> Self {
            Self {
                fixed_time: initial_time,
            }
        }

        /// テストの途中で任意の時刻に変更する
        #[allow(unused)]
        pub fn set_time(&mut self, new_time: DateTime<Utc>) {
            self.fixed_time = new_time;
        }

        /// テストの途中で任意の時刻を加算する
        #[allow(unused)]
        pub fn add_time(&mut self, duration: chrono::Duration) {
            self.fixed_time += duration;
        }
    }

    // Clock トレイトを実装
    impl Clock for MockClock {
        fn now(&self) -> DateTime<Utc> {
            self.fixed_time
        }
    }
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
        assert!(debug.contains("LIMIT $1"));
        assert!(debug.contains("50"));
    }

    #[tokio::test]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn 実際にローカルの_postgre_sqlに接続するテスト() {
        let db = create_db_postgres().await.unwrap();
        let repository = SeaOrmMessageRepository::new(db.clone());
        let now = chrono::Utc::now().fixed_offset();

        let first_uuid = uuid::Uuid::new_v4();
        let email = format!("local-get-{}@example.com", first_uuid);
        let clock = MockClock::new(now.into());
        let usecase = PostMessageUseCase::new(repository.clone(), clock);
        usecase
            .execute(PostMessageInput {
                user_id: first_uuid,
                user_name: email.clone(),
                body: "older body".to_string(),
                row_log: "older row log".to_string(),
                is_from_user: true,
            })
            .await
            .expect("older test message insert should succeed");

        let second_uuid = uuid::Uuid::new_v4();
        let second_email = format!("local-get-{}@example.com", second_uuid);
        let clock = MockClock::new((now + chrono::Duration::seconds(1)).into());
        let usecase = PostMessageUseCase::new(repository.clone(), clock);
        usecase
            .execute(PostMessageInput {
                user_id: second_uuid,
                user_name: second_email.clone(),
                body: "second body".to_string(),
                row_log: "second row log".to_string(),
                is_from_user: false,
            })
            .await
            .expect("newer test message insert should succeed");

        let third_uuid = uuid::Uuid::new_v4();
        let third_email = format!("local-get-{}@example.com", third_uuid);
        let clock = MockClock::new((now + chrono::Duration::seconds(2)).into());
        let usecase = PostMessageUseCase::new(repository.clone(), clock);
        usecase
            .execute(PostMessageInput {
                user_id: third_uuid,
                user_name: third_email.clone(),
                body: "third body".to_string(),
                row_log: "third row log".to_string(),
                is_from_user: true,
            })
            .await
            .expect("newer test message insert should succeed");

        let usecase = GetTimelineUseCase::new(repository.clone());
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
            (now + chrono::Duration::seconds(2))
                .format("%Y-%m-%dT%H:%M:%S.%6fZ")
                .to_string()
                .parse::<DateTime<Utc>>()
                .unwrap()
        );
        assert_eq!(
            response[1].created_at,
            (now + chrono::Duration::seconds(1))
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
