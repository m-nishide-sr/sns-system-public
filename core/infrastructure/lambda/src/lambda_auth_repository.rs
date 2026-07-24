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

/// Lambda を利用した認証(Pre Sign-Up)用 Repository 実装。
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
    /// 許可ドメイン一覧(カンマ区切り)を受け取りRepositoryを生成する。
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

#[cfg(test)]
mod tests {
    use super::*;

    use lambda_runtime::Context;
    use serde_json::json;

    #[test]
    fn 許可ドメインと一致する場合はtrue() {
        assert!(is_allowed_domain("user@example.com", "example.com").unwrap());
    }

    #[test]
    fn 許可ドメインと一致しない場合はfalse() {
        assert!(!is_allowed_domain("user@other.com", "example.com").unwrap());
    }

    #[test]
    fn カンマ区切りの複数ドメインの先頭に一致する場合はtrue() {
        assert!(is_allowed_domain("user@example.com", "example.com,other.com").unwrap());
    }

    #[test]
    fn カンマ区切りの複数ドメインの末尾に一致する場合はtrue() {
        assert!(is_allowed_domain("user@other.com", "example.com,other.com").unwrap());
    }

    #[test]
    fn スペースを含む複数ドメインに一致する場合はtrue() {
        assert!(is_allowed_domain("user@other.com", "example.com, other.com").unwrap());
    }

    #[test]
    fn 不正なメールアドレス形式の場合はエラー() {
        assert!(is_allowed_domain("invalid-email", "example.com").is_err());
    }

    #[tokio::test]
    async fn handlerは型付きイベントからemailを取得できる() {
        let event: CognitoEventUserPoolsPreSignup = serde_json::from_value(json!({
            "version": "1",
            "triggerSource": "PreSignUp_SignUp",
            "region": "ap-northeast-3",
            "userPoolId": "ap-northeast-3_example",
            "userName": "user@example.com",
            "callerContext": {
                "awsSdkVersion": "aws-sdk-js-3.0.0",
                "clientId": "example-client-id"
            },
            "request": {
                "userAttributes": {
                    "email": "user@example.com"
                },
                "validationData": {},
                "clientMetadata": {}
            },
            "response": {
                "autoConfirmUser": false,
                "autoVerifyEmail": false,
                "autoVerifyPhone": false
            }
        }))
        .unwrap();

        let result = function_handler(LambdaEvent::new(event, Context::default()), "example.com");

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handlerはemail属性が無い場合エラー() {
        let event: CognitoEventUserPoolsPreSignup = serde_json::from_value(json!({
            "version": "1",
            "triggerSource": "PreSignUp_SignUp",
            "region": "ap-northeast-3",
            "userPoolId": "ap-northeast-3_example",
            "userName": "user@example.com",
            "callerContext": {
                "awsSdkVersion": "aws-sdk-js-3.0.0",
                "clientId": "example-client-id"
            },
            "request": {
                "userAttributes": {},
                "validationData": {},
                "clientMetadata": {}
            },
            "response": {
                "autoConfirmUser": false,
                "autoVerifyEmail": false,
                "autoVerifyPhone": false
            }
        }))
        .unwrap();

        let result = function_handler(LambdaEvent::new(event, Context::default()), "example.com");

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn handlerは許可しているドメインのemailで無い場合エラー() {
        let event: CognitoEventUserPoolsPreSignup = serde_json::from_value(json!({
            "version": "1",
            "triggerSource": "PreSignUp_SignUp",
            "region": "ap-northeast-3",
            "userPoolId": "ap-northeast-3_example",
            "userName": "user@example.com",
            "callerContext": {
                "awsSdkVersion": "aws-sdk-js-3.0.0",
                "clientId": "example-client-id"
            },
            "request": {
                "userAttributes": {
                    "email": "user@notallowed.com"
                },
                "validationData": {},
                "clientMetadata": {}
            },
            "response": {
                "autoConfirmUser": false,
                "autoVerifyEmail": false,
                "autoVerifyPhone": false
            }
        }))
        .unwrap();

        let result = function_handler(LambdaEvent::new(event, Context::default()), "example.com");

        assert!(result.is_err());
    }

    #[test]
    fn 許可ドメインならtrueになる() {
        assert!(is_allowed_domain("user@example.com", "example.com").unwrap());
    }
    #[test]
    fn メール形式が不正ならエラーになる() {
        assert!(is_allowed_domain("invalid-email", "example.com").is_err());
    }
}
