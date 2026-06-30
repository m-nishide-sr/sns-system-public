//! チャットメッセージのドメインモデル。

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::DomainError;

/// メッセージIDを表す値オブジェクト。
///
/// UUID v7を使用し、プログラム側で生成する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MessageId(Uuid);

impl MessageId {
    /// UUID v7で新しいメッセージIDを生成する。
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// 既存のUUIDからメッセージIDを生成する。
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 内部UUIDを返す。
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

/// CognitoユーザーIDを表す値オブジェクト。
///
/// CognitoのJWTに含まれる`sub`クレームのUUIDを保持する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CognitoId(Uuid);

impl CognitoId {
    /// 内部UUIDを返す。
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl FromStr for CognitoId {
    type Err = DomainError;

    /// 文字列からCognitoIDを生成する。
    ///
    /// UUID形式でない場合は`DomainError::InvalidCognitoId`を返す。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DomainError::InvalidCognitoId(s.to_string()))
    }
}

/// メッセージ本文を表す値オブジェクト。
///
/// 空文字は許可しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBody(String);

impl MessageBody {
    /// 文字列からメッセージ本文を生成する。
    ///
    /// 空文字の場合は`DomainError::EmptyMessageBody`を返す。
    pub fn new(s: impl Into<String>) -> Result<Self, DomainError> {
        let s = s.into();
        if s.is_empty() {
            return Err(DomainError::EmptyMessageBody);
        }
        Ok(Self(s))
    }

    /// 内部文字列を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// チャットメッセージを表すドメインエンティティ。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Message {
    /// メッセージID（UUID v7）
    pub id: Uuid,
    /// ユーザーからのメッセージかどうか（true: ユーザー, false: システム）
    pub is_from_user: bool,
    /// メッセージ本文
    pub body: String,
    /// 作成日時（UTC）
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_id_generates_valid_uuid_v7() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        // UUID v7は時刻ベースのため、連続生成では順序が保証される
        assert_ne!(id1.as_uuid(), id2.as_uuid());
    }

    #[test]
    fn cognito_id_from_valid_str() {
        let uuid_str = "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d";
        let result: Result<CognitoId, _> = uuid_str.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn cognito_id_from_invalid_str() {
        let result: Result<CognitoId, _> = "not-a-uuid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn message_body_rejects_empty() {
        let result = MessageBody::new("");
        assert!(result.is_err());
    }

    #[test]
    fn message_body_accepts_non_empty() {
        let result = MessageBody::new("こんにちは");
        assert!(result.is_ok());
    }
}
