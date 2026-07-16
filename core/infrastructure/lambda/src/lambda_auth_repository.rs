use crate::lambda_executor::lambda_executor;
use aws_lambda_events::cognito::CognitoEventUserPoolsPreSignup;
use core_domain::AuthRepository;
use lambda_runtime::LambdaEvent;
use std::{fmt::Debug, future::Future, pin::Pin};

/// メールアドレスのドメイン部が許可リストに含まれているか検証する。
fn is_allowed_domain(email: &str, allowed_domains: &str) -> Result<bool, lambda_runtime::Error> {
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
fn function_handler(
    event: LambdaEvent<CognitoEventUserPoolsPreSignup>,
    allowed_domains: &str,
) -> Result<CognitoEventUserPoolsPreSignup, lambda_runtime::Error> {
    let email = event
        .payload
        .request
        .user_attributes
        .get("email")
        .map(String::as_str)
        .ok_or_else(|| -> lambda_runtime::Error {
            String::from("リクエストにメールアドレスが含まれていません").into()
        })?;

    if is_allowed_domain(email, allowed_domains)? {
        Ok(event.payload)
    } else {
        let domain = email.rsplit_once('@').map(|(_, d)| d).unwrap_or("");
        Err(format!("メールアドレスのドメイン '{}' は許可されていません", domain).into())
    }
}

/// Lambdaを利用したメッセージRepository実装。
#[derive(Debug)]
pub struct LambdaAuthRepository<T>
where
    T: AsRef<str> + Send + Sync + Debug,
{
    allowed_domains: T,
}

impl<T> LambdaAuthRepository<T>
where
    T: AsRef<str> + Send + Sync + Debug,
{
    /// DB接続を受け取りRepositoryを生成する。
    pub fn new(allowed_domains: T) -> Self {
        Self { allowed_domains }
    }
}

impl<T> AuthRepository for LambdaAuthRepository<T>
where
    T: AsRef<str> + Send + Sync + Debug,
{
    fn execute(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>,
    > {
        // 1. まず String に変換して self の寿命から切り離す
        let allowed_domains = self.allowed_domains.as_ref().to_string();

        // 💡 修正ポイント: 外側の Box::pin は削除し、lambda_executor を直接返す。
        // ただし、型を Pin<Box<dyn Future...>> に合わせるため、全体をラップして返します。
        Box::pin(lambda_executor(move |event| {
            // クロージャが呼ばれるたびにクローンし、任意のライフタイムに対応させる
            let allowed_domains_cloned = allowed_domains.clone();
            async move { function_handler(event, &allowed_domains_cloned) }
        }))
    }
}

// impl<V, F, Fut> AuthRepository for LambdaAuthRepository<V, F, Fut>
// where
//     F: FnOnce(
//         Box<
//             dyn FnMut(
//                     LambdaEvent<CognitoEventUserPoolsPreSignup>,
//                 ) -> Result<CognitoEventUserPoolsPreSignup, Error>
//                 + Send,
//         >,
//     ) -> Fut,
//     Fut: Future<Output = Result<(), Error>>,
//     for<'a> V: 'a + AsRef<str>,
// {
//     pub fn new(lambda_executor: F, allowed_domains: V) -> Self {
//         Self {
//             lambda_executor,
//             allowed_domains,
//         }
//     }

//     pub async fn execute(self) -> Result<(), Error> {
//         let domains = self.allowed_domains;
//         // クロージャをBoxに包んでExecutorに渡す
//         let handler = Box::new(move |event| function_handler(event, domains.as_ref()));

//         (self.lambda_executor)(handler).await
//     }
// }

// /// Lambdaを利用したメッセージRepository実装。
// #[derive(Clone)]
// pub struct LambdaAuthRepository<T, V, F, Fut>
// where
//     T: serde::de::DeserializeOwned,
//     F: FnMut(LambdaEvent<T>) -> Fut,
//     Fut: Future<Output = Result<(), Error>>,
//     for<'a> V: 'a + AsRef<str>,
// {
//     // ハンドラ関数を構造体のフィールドとして保持
//     pub lambda_executor: F,
//     allowed_domains: V,
//     // ジェネリクス T, V を構造体で消費するためのマーカー
//     _marker: std::marker::PhantomData<(T, V)>,
// }

// impl<T, V, F, Fut> LambdaAuthRepository<T, V, F, Fut>
// where
//     T: serde::de::DeserializeOwned,
//     F: FnMut(LambdaEvent<T>) -> Fut,
//     Fut: Future<Output = Result<(), Error>>,
//     for<'a> V: 'a + AsRef<str>,
// {
//     /// コンストラクタ
//     pub fn new(lambda_executor: F, allowed_domains: V) -> Self {
//         Self {
//             lambda_executor,
//             allowed_domains,
//             _marker: std::marker::PhantomData,
//         }
//     }

//     /// Lambdaの実行メソッド（self を受け取る）
//     pub async fn execute(mut self) -> Result<(), Error> {
//         (self.lambda_executor)(|event| async {
//             function_handler(event, self.allowed_domains.as_ref())
//         })
//         .await
//     }
// }

// #[async_trait]
// impl<T, V, F, Fut> AuthRepository for LambdaAuthRepository<T, V, F, Fut>
// where
//     T: serde::de::DeserializeOwned,
//     F: FnMut(LambdaEvent<T>) -> Fut + Send + Sync + Clone,
//     Fut: Future<Output = Result<(), Error>>,
//     for<'a> V: 'a + AsRef<str>,
// {
//     async fn pre_sign_up(
//         &self,
//         input: &PreSignUpInput,
//     ) -> impl std::future::Future<Output = Result<PreSignUpOutput, DomainError>> + Send {
//         let lambda_executor = self.lambda_executor.clone();
//         let input_clone = input.clone();

//         // async move {
//         //     // Lambdaを呼び出す処理をここに実装する
//         //     // 例: lambda_executor.invoke("PreSignUpFunction", input_clone).await
//         //     // ここでは仮の成功レスポンスを返す
//         //     Ok(PreSignUpOutput {
//         //         is_success: true,
//         //         message: "PreSignUp successful".to_string(),
//         //     })
//         // }
//         lambda_executor(|event| async {
//             function_handler(event, ALLOWED_EMAIL_DOMAINS.get().unwrap())
//         })
//         .await
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
