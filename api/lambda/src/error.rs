//! アプリケーション全体で使用するエラー型の定義。

use thiserror::Error;

/// ドメイン層のエラー。
#[derive(Debug, Error)]
pub enum DomainError {
    /// CognitoIDの形式が不正な場合のエラー。
    #[error("CognitoIDの形式が不正です: {0}")]
    InvalidCognitoId(String),

    /// メッセージ本文が空の場合のエラー。
    #[error("メッセージ本文が空です")]
    EmptyMessageBody,
}

/// リポジトリ層のエラー。
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// データベース操作に失敗した場合のエラー。
    #[error("データベースエラー: {0}")]
    Database(#[from] sea_orm::DbErr),
}

/// ユースケース層のエラー。
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// ドメインエラーのラッパー。
    #[error("ドメインエラー: {0}")]
    Domain(#[from] DomainError),

    /// リポジトリエラーのラッパー。
    #[error("リポジトリエラー: {0}")]
    Repository(#[from] RepositoryError),
}

/// ハンドラー層のエラー。
#[derive(Debug, Error)]
pub enum HandlerError {
    /// リクエストが不正な場合のエラー。
    #[error("リクエストが不正です: {0}")]
    BadRequest(String),

    /// 認証に失敗した場合のエラー。
    #[error("認証に失敗しました")]
    Unauthorized,

    /// ユースケースエラーのラッパー。
    #[error("ユースケースエラー: {0}")]
    UseCase(#[from] UseCaseError),
}
