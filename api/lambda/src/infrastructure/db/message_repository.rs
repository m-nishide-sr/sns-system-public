//! sea-ORMを使用したメッセージリポジトリの実装。

use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use sea_orm_entities::entity::messages::{ActiveModel, Column, Entity as Messages};

use crate::application::repository::MessageRepository;
use crate::domain::message::{CognitoId, Message, MessageBody, MessageId};
use crate::error::RepositoryError;

/// sea-ORMを使用したメッセージリポジトリの実装。
pub struct MessageRepositoryImpl {
    db: DatabaseConnection,
}

impl MessageRepositoryImpl {
    /// 新しいリポジトリを生成する。
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl MessageRepository for MessageRepositoryImpl {
    /// 指定ユーザーのメッセージを作成日時の降順で取得する。
    async fn get_messages(&self, cognito_id: &CognitoId) -> Result<Vec<Message>, RepositoryError> {
        let models = Messages::find()
            .filter(Column::CognitoId.eq(cognito_id.as_uuid()))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await?;

        let messages = models
            .into_iter()
            .map(|m| {
                let created_at =
                    DateTime::from_timestamp(m.created_at.timestamp(), 0).unwrap_or_default();
                Message {
                    id: m.id,
                    is_from_user: m.is_from_user,
                    body: m.body,
                    created_at,
                }
            })
            .collect();

        Ok(messages)
    }

    /// 新しいメッセージを保存する。
    async fn save_message(
        &self,
        id: MessageId,
        cognito_id: &CognitoId,
        body: &MessageBody,
    ) -> Result<(), RepositoryError> {
        // is_from_user は常に true（ユーザーからの投稿のため）
        let active_model = ActiveModel {
            id: Set(id.as_uuid()),
            cognito_id: Set(cognito_id.as_uuid()),
            body: Set(body.as_str().to_string()),
            is_from_user: Set(true),
            ..Default::default()
        };
        active_model.insert(&self.db).await?;
        Ok(())
    }
}
