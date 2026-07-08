use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const MAX_BODY_LENGTH: usize = 500;
const MAX_USER_NAME_LENGTH: usize = 64;

/// メッセージ本文を表す値オブジェクト。
///
/// 空文字列や過度に長い本文は保存前に拒否し、
/// DB整合性エラーではなく業務エラーとして扱う。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageBody(String);

impl MessageBody {
    /// 業務制約を満たす本文を生成する。
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return Err(CoreError::Validation("本文は必須です".to_string()));
        }

        if trimmed.chars().count() > MAX_BODY_LENGTH {
            return Err(CoreError::Validation(format!(
                "本文は{MAX_BODY_LENGTH}文字以下で入力してください"
            )));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// 文字列参照を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 投稿者名を表す値オブジェクト。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserName(String);

impl UserName {
    /// 妥当な投稿者名を生成する。
    pub fn new(value: impl Into<String>) -> CoreResult<Self> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CoreError::Validation("ユーザー名は必須です".to_string()));
        }
        if trimmed.chars().count() > MAX_USER_NAME_LENGTH {
            return Err(CoreError::Validation(format!(
                "ユーザー名は{MAX_USER_NAME_LENGTH}文字以下で入力してください"
            )));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// 文字列参照を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 認証済みユーザーID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    /// UUIDからユーザーIDを作成する。
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    /// 生のUUIDを返す。
    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

/// タイムライン表示用のドメインモデル。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineMessage {
    /// 投稿者名。
    pub user_name: String,
    /// 投稿日時(UTC)。
    pub created_at: DateTime<Utc>,
    /// 本文。
    pub body: String,
    /// ユーザー投稿かどうか。
    pub is_from_user: bool,
}

/// 新規メッセージ作成時に必要なドメイン入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMessage {
    /// 投稿者名。
    pub user_name: UserName,
    /// Cognitoのsub。
    pub user_id: UserId,
    /// 本文。
    pub body: MessageBody,
    /// 作成時刻。
    pub created_at: DateTime<Utc>,
    /// ユーザー投稿かどうか。
    pub is_from_user: bool,
    /// 障害調査用の生ログ。
    pub row_log: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 本文が空文字ならエラー() {
        let result = MessageBody::new("   ");
        assert!(matches!(result, Err(CoreError::Validation(_))));
    }

    #[test]
    fn 本文が上限超過ならエラー() {
        let body = "あ".repeat(501);
        let result = MessageBody::new(body);
        assert!(matches!(result, Err(CoreError::Validation(_))));
    }

    #[test]
    fn ユーザー名が空文字ならエラー() {
        let result = UserName::new(" ");
        assert!(matches!(result, Err(CoreError::Validation(_))));
    }

    #[test]
    fn 正常な本文は前後空白を除去して保持() {
        let result = MessageBody::new("  テスト  ").expect("正常系で失敗しない想定");
        assert_eq!(result.as_str(), "テスト");
    }
}
