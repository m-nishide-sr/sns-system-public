use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::common::error::CoreError;

const MAX_BODY_LENGTH: usize = 500;

/// 投稿本文を表す値オブジェクト。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBody(String);

impl MessageBody {
    /// 業務ルールを満たす本文を生成する。
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(CoreError::Validation("bodyは必須です".to_owned()));
        }

        if trimmed.chars().count() > MAX_BODY_LENGTH {
            return Err(CoreError::Validation(format!(
                "bodyは{MAX_BODY_LENGTH}文字以内で指定してください"
            )));
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// 永続化に利用する内部文字列を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// タイムライン表示用のメッセージ。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimelineMessage {
    /// 投稿ユーザー名。
    pub user_name: String,
    /// 投稿日時。
    pub created_at: DateTime<Utc>,
    /// 本文。
    pub body: String,
    /// 投稿元がユーザーかどうか。
    pub is_from_user: bool,
}

/// 新規投稿時の業務入力データ。
#[derive(Debug, Clone)]
pub struct CreateMessageCommand {
    /// 投稿ユーザー名。
    pub user_name: String,
    /// CognitoのユーザーサブジェクトID。
    pub cognito_id: String,
    /// 投稿本文。
    pub body: MessageBody,
    /// 調査用途の生ログ。
    pub row_log: String,
    /// 投稿元がユーザーかどうか。
    pub is_from_user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bodyが空白のみの場合はエラー() {
        let result = MessageBody::new("   ");

        assert!(matches!(result, Err(CoreError::Validation(_))));
    }

    #[test]
    fn bodyが500文字を超える場合はエラー() {
        let value = "a".repeat(501);
        let result = MessageBody::new(value);

        assert!(matches!(result, Err(CoreError::Validation(_))));
    }

    #[test]
    fn bodyが有効な場合はtrim済みで生成される() {
        let result = MessageBody::new("  こんにちは  ").expect("有効なbodyは生成されるべき");

        assert_eq!(result.as_str(), "こんにちは");
    }
}
