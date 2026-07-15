use aws_lambda_events::cognito::CognitoEventUserPoolsPreSignup;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use std::env;

/// メールアドレスのドメイン部が許可リストに含まれているか検証する。
fn is_allowed_domain(email: &str, allowed_domains: &str) -> Result<bool, Error> {
    let (_, domain) = email.rsplit_once('@').ok_or_else(|| -> Error {
        String::from("メールアドレスの形式が不正です").into()
    })?;
    let domain = domain.to_ascii_lowercase();

    Ok(allowed_domains
        .split(',')
        .any(|d| d.trim().to_ascii_lowercase() == domain))
}

/// Cognito の Pre Sign-Up トリガー要求を検証し、そのまま返却する。
async fn handler(
    event: LambdaEvent<CognitoEventUserPoolsPreSignup>,
    allowed_domains: &str,
) -> Result<CognitoEventUserPoolsPreSignup, Error> {
    let email = event
        .payload
        .request
        .user_attributes
        .get("email")
        .map(String::as_str)
        .ok_or_else(|| -> Error {
            String::from("リクエストにメールアドレスが含まれていません").into()
        })?;

    if is_allowed_domain(email, allowed_domains)? {
        Ok(event.payload)
    } else {
        let domain = email.rsplit_once('@').map(|(_, d)| d).unwrap_or("");
        Err(format!("メールアドレスのドメイン '{}' は許可されていません", domain).into())
    }
}

/// Lambda ランタイムを起動する。
pub async fn run_lambda() -> Result<(), Error> {
    let allowed_domains: String = env::var("ALLOWED_EMAIL_DOMAINS").map_err(|_| -> Error {
        String::from("ALLOWED_EMAIL_DOMAINS 環境変数が設定されていません").into()
    })?;

    run(service_fn(|event| handler(event, &allowed_domains))).await
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

        let result = handler(
            LambdaEvent::new(event, Context::default()),
            "example.com",
        )
        .await;

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

        let result = handler(
            LambdaEvent::new(event, Context::default()),
            "example.com",
        )
        .await;

        assert!(result.is_err());
    }
}
