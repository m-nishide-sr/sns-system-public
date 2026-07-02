//! メッセージRepositoryトレイト。
//!
//! DBアクセスを抽象化し、ユースケースとインフラストラクチャ層を分離します。
//! テスト時はモック実装に差し替えることで、DBなしのユースケーステストが可能です。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{Message, NewMessage, RepositoryError};

/// メッセージのCRUD操作を抽象化するRepositoryトレイト。
#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// タイムライン（メッセージ一覧）を最大50件取得します。
    ///
    /// # 引数
    ///
    /// * `before` - 指定した日時より前のメッセージを取得する場合に使用（ページネーション用）。
    ///
    /// # 戻り値
    ///
    /// `created_at` の降順で最大50件のメッセージリスト。
    async fn get_timeline(
        &self,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, RepositoryError>;

    /// メッセージを新規投稿します。
    ///
    /// # 引数
    ///
    /// * `new_message` - 投稿するメッセージのデータ。
    async fn post_message(&self, new_message: NewMessage) -> Result<(), RepositoryError>;
}

/// `Arc<dyn MessageRepository>` に対してもトレイトを委譲実装します。
///
/// これにより、axumのStateに格納した `Arc<dyn MessageRepository>` を
/// そのままユースケースに渡すことができます。
#[async_trait]
impl MessageRepository for Arc<dyn MessageRepository> {
    async fn get_timeline(
        &self,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>, RepositoryError> {
        self.as_ref().get_timeline(before).await
    }

    async fn post_message(&self, new_message: NewMessage) -> Result<(), RepositoryError> {
        self.as_ref().post_message(new_message).await
    }
}
