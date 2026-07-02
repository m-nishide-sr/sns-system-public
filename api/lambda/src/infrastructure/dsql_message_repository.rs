//! Aurora DSQL（PostgreSQL互換）を使ったメッセージRepositoryの実装。
//!
//! AWS環境ではIAM認証トークンを使用してDSQLに接続します。
//! ローカル環境では `DB_PASSWORD` 環境変数のパスワードを使用します。
//!
//! ## 環境変数
//!
//! - `DSQL_ENDPOINT` : DSQLのエンドポイント（ホスト名）。
//!   ローカル開発時は `localhost` を指定します。
//! - `DB_PASSWORD` : ローカル開発時のDBパスワード。
//!   この環境変数が設定されている場合はパスワード認証を使用し、
//!   未設定の場合はIAM認証トークンを生成してDSQLに接続します。

use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config as AuthTokenConfig};
use chrono::{DateTime, Utc};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::NoTls;

use crate::application::MessageRepository;
use crate::domain::{Message, NewMessage, RepositoryError};

/// Aurora DSQL（PostgreSQL互換）を使ったメッセージRepositoryの実装。
pub struct DsqlMessageRepository {
    /// DSQLのエンドポイント（ホスト名）。
    dsql_endpoint: String,
    /// ローカル開発用のDBパスワード（設定されていない場合はIAM認証を使用）。
    db_password: Option<String>,
}

impl DsqlMessageRepository {
    /// 環境変数から設定を読み込んで新しいインスタンスを生成します。
    ///
    /// 環境変数 `DSQL_ENDPOINT` でエンドポイントを指定します。
    /// 環境変数 `DB_PASSWORD` が設定されている場合はパスワード認証を使用します。
    pub fn from_env() -> Self {
        let dsql_endpoint =
            std::env::var("DSQL_ENDPOINT").unwrap_or_else(|_| "localhost".to_string());
        let db_password = std::env::var("DB_PASSWORD").ok();
        Self {
            dsql_endpoint,
            db_password,
        }
    }

    /// DSQLへの接続を確立し、 `tokio_postgres::Client` を返します。
    async fn connect(&self) -> Result<tokio_postgres::Client, RepositoryError> {
        if let Some(ref password) = self.db_password {
            // ローカル環境: パスワード認証でSSLなし接続
            self.connect_without_ssl(password).await
        } else {
            // AWS環境: IAM認証トークンを生成してSSL接続
            let token = self.generate_iam_token().await?;
            self.connect_with_ssl(&token).await
        }
    }

    /// SSLなしでPostgreSQLに接続します（ローカル開発用）。
    async fn connect_without_ssl(
        &self,
        password: &str,
    ) -> Result<tokio_postgres::Client, RepositoryError> {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host(&self.dsql_endpoint)
            .port(5432)
            .user("lambda")
            .password(password)
            .dbname("postgres");

        let (client, connection) = pg_config
            .connect(NoTls)
            .await
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;

        // 接続タスクをバックグラウンドで実行する
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("DB接続エラー: {e}");
            }
        });

        Ok(client)
    }

    /// SSL（TLS）でDSQLに接続します（AWS環境用）。
    async fn connect_with_ssl(
        &self,
        password: &str,
    ) -> Result<tokio_postgres::Client, RepositoryError> {
        let mut ssl_builder = SslConnector::builder(SslMethod::tls())
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;
        // DSQLではシステムCAを信頼することでサーバー証明書を検証する
        ssl_builder.set_verify(SslVerifyMode::PEER);
        let tls = MakeTlsConnector::new(ssl_builder.build());

        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host(&self.dsql_endpoint)
            .port(5432)
            .user("lambda")
            .password(password)
            .dbname("postgres");

        let (client, connection) = pg_config
            .connect(tls)
            .await
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;

        // 接続タスクをバックグラウンドで実行する
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("DB接続エラー: {e}");
            }
        });

        Ok(client)
    }

    /// AWS DSQLのIAM認証トークンを生成します。
    async fn generate_iam_token(&self) -> Result<String, RepositoryError> {
        let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let auth_config = AuthTokenConfig::builder()
            .hostname(&self.dsql_endpoint)
            .build()
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;
        let generator = AuthTokenGenerator::new(auth_config);
        let token = generator
            .db_connect_auth_token(&sdk_config)
            .await
            .map_err(|e| RepositoryError::Connection(e.to_string()))?;
        Ok(token.to_string())
    }
}

#[async_trait]
impl MessageRepository for DsqlMessageRepository {
    /// `public.messages_latest` ビューからタイムラインを取得します。
    ///
    /// `before` パラメータを指定した場合、その日時より前のメッセージのみ取得します。
    /// `created_at` の降順で最大50件を返します。
    async fn get_timeline(
        &self,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, RepositoryError> {
        let client = self.connect().await?;

        let rows = if let Some(before) = before {
            let stmt = "SELECT user_name, created_at, body, is_from_user \
                        FROM public.messages_latest \
                        WHERE created_at < $1 \
                        ORDER BY created_at DESC \
                        LIMIT 50";
            client.query(stmt, &[&before]).await?
        } else {
            let stmt = "SELECT user_name, created_at, body, is_from_user \
                        FROM public.messages_latest \
                        ORDER BY created_at DESC \
                        LIMIT 50";
            client.query(stmt, &[]).await?
        };

        let messages = rows
            .into_iter()
            .map(|row| Message {
                user_name: row.get("user_name"),
                created_at: row.get("created_at"),
                body: row.get("body"),
                is_from_user: row.get("is_from_user"),
            })
            .collect();

        Ok(messages)
    }

    /// `public.messages` テーブルに新規メッセージを挿入します。
    async fn post_message(&self, new_message: NewMessage) -> Result<(), RepositoryError> {
        let client = self.connect().await?;

        let stmt = "INSERT INTO public.messages \
                    (body, user_name, cognito_id, row_log, is_from_user) \
                    VALUES ($1, $2, $3, $4, $5)";
        client
            .execute(
                stmt,
                &[
                    &new_message.body,
                    &new_message.user_name,
                    &new_message.cognito_id,
                    &new_message.row_log,
                    &new_message.is_from_user,
                ],
            )
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ローカルのDBが必要なため、デフォルトでは実行しない統合テスト。
    /// 実行時は `DSQL_ENDPOINT=localhost DB_PASSWORD=lambda` を設定してください。
    #[tokio::test]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn タイムラインを取得できる() {
        let repo = DsqlMessageRepository {
            dsql_endpoint: "localhost".to_string(),
            db_password: Some("lambda".to_string()),
        };
        let result = repo.get_timeline(None).await;
        assert!(result.is_ok(), "タイムライン取得に失敗: {:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn メッセージを投稿できる() {
        use uuid::Uuid;

        let repo = DsqlMessageRepository {
            dsql_endpoint: "localhost".to_string(),
            db_password: Some("lambda".to_string()),
        };
        let new_message = NewMessage {
            body: "統合テスト投稿".to_string(),
            user_name: "testuser".to_string(),
            cognito_id: Uuid::new_v4(),
            row_log: r#"{"test": true}"#.to_string(),
            is_from_user: true,
        };
        let result = repo.post_message(new_message).await;
        assert!(result.is_ok(), "メッセージ投稿に失敗: {:?}", result.err());
    }
}
