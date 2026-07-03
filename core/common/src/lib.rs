//! 共通的で技術非依存なユーティリティを提供するクレート。
//!
//! この層にはビジネスルールを置かず、各レイヤーで共有しても
//! ドメイン知識が汚染されない最小限の基盤要素だけを保持する。

pub mod clock;
pub mod error;

pub use clock::{Clock, SystemClock};
pub use error::{CoreError, CoreResult};
