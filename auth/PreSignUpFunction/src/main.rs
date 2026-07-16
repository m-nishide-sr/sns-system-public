use core_domain::AuthRepository;
use core_infrastructure_lambda::LambdaAuthRepository;
use lambda_runtime::{Error, tracing};
use std::env;
use std::sync::OnceLock;

static ALLOWED_EMAIL_DOMAINS: OnceLock<String> = OnceLock::new();

static EXECUTOR: OnceLock<Box<dyn AuthRepository>> = OnceLock::new();

#[tokio::main(flavor = "current_thread")]
/// Lambda ランタイムへ型付きハンドラを登録するエントリーポイント。
async fn main() -> Result<(), Error> {
    executor().await
}

/// Lambda 処理の実態。
async fn executor() -> Result<(), Error> {
    let allowed_domains = ALLOWED_EMAIL_DOMAINS.get_or_init(|| {
        match env::var("ALLOWED_EMAIL_DOMAINS") {
            Ok(value) => value,
            Err(e) => {
                // 取得に失敗したら tracing でエラーログを出力
                tracing::error!(
                    "ALLOWED_EMAIL_DOMAINS 環境変数の取得に失敗しました: {:?}",
                    e
                );
                // panic! で Lambda を終了させる(Fail-Fast)
                panic!("Internal Server Error");
            }
        }
    });

    let executor = EXECUTOR.get_or_init(|| Box::new(LambdaAuthRepository::new(allowed_domains)));

    executor.execute().await
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, pin::Pin};

    use super::*;

    /// メールアドレスのドメイン部が許可リストに含まれているか検証する。
    fn is_allowed_domain(
        email: &str,
        allowed_domains: &str,
    ) -> Result<bool, lambda_runtime::Error> {
        let (_, domain) = email
            .rsplit_once('@')
            .ok_or_else(|| -> lambda_runtime::Error {
                String::from("メールアドレスの形式が不正です").into()
            })?;
        let domain = domain.to_ascii_lowercase();

        Ok(allowed_domains
            .split(',')
            .any(|d| d.trim().to_ascii_lowercase() == domain))
    }

    /// Cognito の Pre Sign-Up トリガー要求を検証し、そのまま返却する。
    fn function_handler(email: &str, allowed_domains: &str) -> Result<(), lambda_runtime::Error> {
        if is_allowed_domain(email, allowed_domains)? {
            Ok(())
        } else {
            let domain = email.rsplit_once('@').map(|(_, d)| d).unwrap_or("");
            Err(format!("メールアドレスのドメイン '{}' は許可されていません", domain).into())
        }
    }

    /// Lambdaを利用したメッセージRepository実装。
    #[derive(Debug)]
    pub struct MockAuthRepository<T>
    where
        T: AsRef<str> + Send + Sync + Debug,
    {
        allowed_domains: T,
    }

    impl<T> MockAuthRepository<T>
    where
        T: AsRef<str> + Send + Sync + Debug,
    {
        /// DB接続を受け取りRepositoryを生成する。
        pub fn new(allowed_domains: T) -> Self {
            Self { allowed_domains }
        }
    }

    impl<T> AuthRepository for MockAuthRepository<T>
    where
        T: AsRef<str> + Send + Sync + Debug,
    {
        // 戻り値の Future に引数のライフタイム（'_）を関連付ける
        fn execute(
            &self, // selfを消費(move)せず参照で受け取る場合
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                    + Send
                    + '_,
            >,
        > {
            let allowed_domains = self.allowed_domains.as_ref();
            Box::pin(async move { function_handler("test@example.com", allowed_domains) })
        }
    }
    #[tokio::test]
    async fn mainのロジックが成功すること() {
        // 環境変数のモックなどが必要な場合はここで行う
        ALLOWED_EMAIL_DOMAINS
            .set("example.com".to_string())
            .unwrap();

        EXECUTOR
            .set(Box::new(MockAuthRepository::new(
                ALLOWED_EMAIL_DOMAINS.get().unwrap().clone(),
            )))
            .unwrap();

        // #[tokio::main] を経由せず、ロジックを直接テストする
        let result = executor().await;

        // lambda_runtime::run はテスト環境だと別のエラー（接続失敗など）に
        // なる可能性があるため、アサーションは実際の検証目的に合わせて調整してください
        assert!(result.is_ok());
    }
}
