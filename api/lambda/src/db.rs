use async_trait::async_trait;
use aurora_dsql_sqlx_connector::pool;
use chrono::{DateTime, Utc};
use sns_core::{
    common::error::CoreError,
    domain::{
        message::{CreateMessageCommand, TimelineMessage},
        repository::MessageRepository,
    },
};
use sqlx::{Pool, Postgres};

/// Aurora DSQLに接続した`sqlx::Pool`を生成する。
pub async fn create_db(
    role: &str,
    endpoint: &str,
    region: &str,
) -> Result<Pool<Postgres>, lambda_runtime::Error> {
    let url = format!("postgres://{role}@{endpoint}/postgres?region={region}");
    pool::connect(&url)
        .await
        .map_err(|e| anyhow::anyhow!("DB接続に失敗しました: {e}").into())
}

/// sqlxを利用したメッセージリポジトリ実装。
pub struct SqlxMessageRepository {
    db: Pool<Postgres>,
}

impl SqlxMessageRepository {
    /// 新しいリポジトリを生成する。
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MessageRepository for SqlxMessageRepository {
    async fn get_timeline(
        &self,
        before: Option<DateTime<Utc>>,
        limit: u64,
    ) -> Result<Vec<TimelineMessage>, CoreError> {
        let records = if let Some(before) = before {
            sqlx::query_as::<_, (String, DateTime<Utc>, String, bool)>(
                "SELECT user_name, created_at, body, is_from_user FROM public.messages_latest WHERE created_at < $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(before)
            .bind(i64::try_from(limit).map_err(|e| CoreError::Repository(e.to_string()))?)
            .fetch_all(&self.db)
            .await
        } else {
            sqlx::query_as::<_, (String, DateTime<Utc>, String, bool)>(
                "SELECT user_name, created_at, body, is_from_user FROM public.messages_latest ORDER BY created_at DESC LIMIT $1",
            )
            .bind(i64::try_from(limit).map_err(|e| CoreError::Repository(e.to_string()))?)
            .fetch_all(&self.db)
            .await
        }
        .map_err(|e| CoreError::Repository(e.to_string()))?;

        Ok(records
            .into_iter()
            .map(
                |(user_name, created_at, body, is_from_user)| TimelineMessage {
                    user_name,
                    created_at,
                    body,
                    is_from_user,
                },
            )
            .collect())
    }

    async fn create_message(&self, command: &CreateMessageCommand) -> Result<(), CoreError> {
        sqlx::query(
            "INSERT INTO public.messages (user_name, cognito_id, body, row_log, is_from_user) VALUES ($1, $2::uuid, $3, $4, $5)",
        )
        .bind(&command.user_name)
        .bind(&command.cognito_id)
        .bind(command.body.as_str())
        .bind(&command.row_log)
        .bind(command.is_from_user)
        .execute(&self.db)
        .await
        .map_err(|e| CoreError::Repository(e.to_string()))?;

        Ok(())
    }
}
