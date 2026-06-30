//! メッセージリポジトリのトレイト定義。
//!
//! アプリケーション層はこのトレイトに依存し、具体的な実装はインフラストラクチャ層で提供する。

use crate::domain::message::{CognitoId, Message, MessageBody, MessageId};
use crate::error::RepositoryError;

/// メッセージの永続化操作を抽象化するトレイト。
pub trait MessageRepository {
    /// 指定ユーザーのメッセージを作成日時の降順で取得する。
    fn get_messages(
        &self,
        cognito_id: &CognitoId,
    ) -> impl Future<Output = Result<Vec<Message>, RepositoryError>> + Send;

    /// 新しいメッセージを保存する。
    fn save_message(
        &self,
        id: MessageId,
        cognito_id: &CognitoId,
        body: &MessageBody,
    ) -> impl Future<Output = Result<(), RepositoryError>> + Send;
}
