//! `messages` テーブルのORM定義。
//!
//! `sea-orm-cli generate entity` によって自動生成されます。
//! このファイルは手動で編集しないでください。

use sea_orm::entity::prelude::*;

/// `messages` テーブルのエンティティモデル。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    /// メッセージID（UUID v7）
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// CognitoユーザーのサブジェクトID
    pub cognito_id: Uuid,
    /// メッセージ作成日時
    pub created_at: DateTimeWithTimeZone,
    /// メッセージ本文
    pub body: String,
    /// ユーザーからのメッセージかどうか（true: ユーザー, false: システム）
    pub is_from_user: bool,
}

/// `messages` テーブルのリレーション定義。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
