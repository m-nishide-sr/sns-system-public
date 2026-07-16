//! インフラ層のLambda実行モジュール
//!
//! このモジュールは、AWS Lambdaの実行環境での共通処理を提供します。

pub mod lambda_auth_repository;
pub mod lambda_executor;

pub use lambda_auth_repository::LambdaAuthRepository;
pub use lambda_executor::lambda_executor;
