//! メッセージドメインエンティティ。
//!
//! SNSシステムのチャットメッセージを表すエンティティと、
//! メッセージ投稿時に必要なデータ構造を定義します。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

/// タイムラインから取得したメッセージ。
///
/// `public.messages_latest` ビューのカラムに対応します。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Message {
    /// 投稿ユーザーのemailアドレスの@以前の文字列。
    pub user_name: String,
    /// メッセージ作成日時（UTC）。
    pub created_at: DateTime<Utc>,
    /// メッセージ本文。
    pub body: String,
    /// `true`: ユーザー投稿, `false`: システム投稿。
    pub is_from_user: bool,
}

/// メッセージ新規作成時のデータ。
///
/// `public.messages` テーブルへの挿入に使用します。
#[derive(Debug, Clone)]
pub struct NewMessage {
    /// メッセージ本文。
    pub body: String,
    /// 投稿ユーザーのemailアドレスの@以前の文字列。
    pub user_name: String,
    /// 投稿ユーザーのCognitoサブジェクトID。
    pub cognito_id: Uuid,
    /// 生ログ（不具合調査用）。リクエストのJSON表現を格納します。
    pub row_log: String,
    /// `true`: ユーザー投稿, `false`: システム投稿。
    pub is_from_user: bool,
}

/// Repositoryで発生するエラー。
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// データベース接続・クエリエラー。
    #[error("データベースエラー: {0}")]
    Database(#[from] tokio_postgres::Error),
    /// データベース接続エラー。
    #[error("データベース接続エラー: {0}")]
    Connection(String),
    /// UUID解析エラー。
    #[error("UUID解析エラー: {0}")]
    Uuid(#[from] uuid::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn メッセージを生成できる() {
        let msg = Message {
            user_name: "testuser".to_string(),
            created_at: DateTime::from_timestamp(0, 0).unwrap(),
            body: "こんにちは".to_string(),
            is_from_user: true,
        };
        assert_eq!(msg.user_name, "testuser");
        assert!(msg.is_from_user);
    }

    #[test]
    fn 新規メッセージを生成できる() {
        let new_msg = NewMessage {
            body: "テストメッセージ".to_string(),
            user_name: "testuser".to_string(),
            cognito_id: Uuid::new_v4(),
            row_log: "{}".to_string(),
            is_from_user: true,
        };
        assert_eq!(new_msg.body, "テストメッセージ");
    }
}
