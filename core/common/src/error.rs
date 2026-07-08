use thiserror::Error;

/// コア層全体で共有するエラー。
///
/// 外部技術依存の詳細は文字列へ集約し、呼び出し元ではカテゴリごとに
/// HTTPレスポンスや再試行制御へマッピングできるようにする。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// 入力値やドメイン制約に反したエラー。
    #[error("入力値が不正です: {0}")]
    Validation(String),
    /// 認証情報が不足または不正なエラー。
    #[error("認証に失敗しました")]
    Unauthorized,
    /// DBやネットワーク等の外部I/O起因のエラー。
    #[error("インフラ層でエラーが発生しました: {0}")]
    Infrastructure(String),
}

/// コア層の戻り値型。
pub type CoreResult<T> = Result<T, CoreError>;
