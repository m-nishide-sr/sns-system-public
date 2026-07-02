//! インターフェースレイヤーモジュール。
//!
//! HTTPハンドラ・リクエスト/レスポンスのDTOを定義します。

pub mod handler;

pub use handler::{AppError, AppState};
