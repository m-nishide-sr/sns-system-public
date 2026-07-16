use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

// ビジネスルールに基づいた、ドメイン層専用のエラー型
#[derive(Debug)]
pub enum DomainError {
    InvalidInput(String),
    NetworkError(String),
}

// 外部の仕様に依存しない純粋なドメインモデル（引数と戻り値）
pub struct PreSignUpInput {
    pub username: String,
    pub email: String,
}

pub struct PreSignUpOutput {
    pub is_success: bool,
    pub message: String,
}

/// 1. 複雑なエラー型とFuture型をエイリアス（type）として切り出す
pub type AuthError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// ドメイン層が定義する「インターフェース」（依存性逆転の原則）
///
/// ユースケースは本traitのみに依存し、Lambdaを知らない状態を維持する。
pub trait AuthRepository: Send + Sync + Debug {
    fn execute(&self) -> BoxFuture<'_, Result<(), AuthError>>;
}
