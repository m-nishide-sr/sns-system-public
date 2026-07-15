use chrono::{DateTime, Utc};

/// 現在時刻を取得するための抽象。
///
/// ユースケースがシステム時刻へ直接依存するとテスト時の再現性が下がるため、
/// 本トレイトを介して差し替え可能にする。
pub trait Clock: Send + Sync {
    /// 現在のUTC時刻を返す。
    fn now(&self) -> DateTime<Utc>;
}

/// 実運用で利用するシステム時計。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn system_clockは現在時刻に近い値を返す() {
        let before = Utc::now();
        let now = SystemClock.now();
        let after = Utc::now();

        assert!(now >= before - Duration::seconds(1));
        assert!(now <= after + Duration::seconds(1));
    }
}
