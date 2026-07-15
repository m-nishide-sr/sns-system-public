//! 共通的で技術非依存なユーティリティを提供するクレート。
//!
//! この層にはビジネスルールを置かず、各レイヤーで共有しても
//! ドメイン知識が汚染されない最小限の基盤要素だけを保持する。

pub mod clock;
pub mod error;

pub use clock::{Clock, SystemClock};
pub use error::{CoreError, CoreResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 公開re_exportを利用できる() {
        let now = SystemClock.now();
        let result: CoreResult<i32> = Ok(1);

        assert!(now.timestamp() > 0);
        assert_eq!(result, Ok(1));
    }
}
