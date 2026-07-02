use thiserror::Error;

/// コア層で扱う共通エラー。
#[derive(Debug, Error)]
pub enum CoreError {
    /// 外部入力が業務ルールを満たさない場合。
    #[error("入力値が不正です: {0}")]
    Validation(String),
    /// API入力フォーマットが不正な場合。
    #[error("リクエストが不正です: {0}")]
    BadRequest(String),
    /// 永続化や参照処理が失敗した場合。
    #[error("データアクセスに失敗しました: {0}")]
    Repository(String),
}
