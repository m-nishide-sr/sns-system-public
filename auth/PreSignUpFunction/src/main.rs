//! Cognito Pre Sign-Up トリガーでメールドメイン制約を適用する Lambda 関数。
//!
//! この関数は、ユーザー登録時の `request.userAttributes.email` を検査し、
//! 環境変数 `ALLOWED_EMAIL_DOMAINS` に列挙されたドメイン以外からの登録を拒否します。
//! Cognito の正式なイベント型を受け取ることで、トリガー入力の前提を Rust の型で表現し、
//! `cargo doc` 上でも参照しやすい実装にします。

use aws_lambda_events::cognito::CognitoEventUserPoolsPreSignup;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use std::env;

/// メールアドレスのドメイン部が許可リストに含まれているか検証する。
///
/// # 引数
///
/// * `email` - 検証対象のメールアドレス
/// * `allowed_domains` - カンマ区切りの許可ドメインリスト
///
/// # 戻り値
///
/// 許可されている場合は `Ok(true)`、許可されていない場合は `Ok(false)`、
/// メールアドレスの形式が不正な場合は `Err`。
fn is_allowed_domain(email: &str, allowed_domains: &str) -> Result<bool, Error> {
    // '@' でメールアドレスを分割し、ドメイン部を取得する
    let (_, domain) = email.rsplit_once('@').ok_or_else(|| -> Error {
        String::from("メールアドレスの形式が不正です").into()
    })?;
    let domain = domain.to_ascii_lowercase();

    Ok(allowed_domains
        .split(',')
        .any(|d| d.trim().to_ascii_lowercase() == domain))
}

/// Cognito の Pre Sign-Up トリガー要求を検証し、そのまま返却する。
///
/// Lambda は登録リクエスト自体を書き換えず、許可判定のみを担当します。
/// そのため、検証に成功した場合は受け取ったイベントをそのまま返し、
/// Cognito 側の後続処理へ引き渡します。
async fn handler(
    event: LambdaEvent<CognitoEventUserPoolsPreSignup>,
) -> Result<CognitoEventUserPoolsPreSignup, Error> {
    let allowed_domains = env::var("ALLOWED_EMAIL_DOMAINS").map_err(|_| -> Error {
        // 環境変数が未設定の場合はデプロイ設定の問題であるため、明示的なエラーを返す
        String::from("ALLOWED_EMAIL_DOMAINS 環境変数が設定されていません").into()
    })?;

    let email = event
        .payload
        .request
        .user_attributes
        .get("email")
        .map(String::as_str)
        .ok_or_else(|| -> Error {
            String::from("リクエストにメールアドレスが含まれていません").into()
        })?;

    if is_allowed_domain(email, &allowed_domains)? {
        Ok(event.payload)
    } else {
        let domain = email.rsplit_once('@').map(|(_, d)| d).unwrap_or("");
        // ドメインが許可リストに含まれていないためサインアップを拒否する
        Err(format!("メールアドレスのドメイン '{}' は許可されていません", domain).into())
    }
}

#[tokio::main(flavor = "current_thread")]
/// Lambda ランタイムへ型付きハンドラを登録するエントリーポイント。
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
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
        let previous = env::var_os("ALLOWED_EMAIL_DOMAINS");
        unsafe {
            env::set_var("ALLOWED_EMAIL_DOMAINS", "example.com");
        }

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

        let result = handler(LambdaEvent::new(event, Context::default())).await;

        match previous {
            Some(value) => unsafe {
                env::set_var("ALLOWED_EMAIL_DOMAINS", value);
            },
            None => unsafe {
                env::remove_var("ALLOWED_EMAIL_DOMAINS");
            },
        }

        assert!(result.is_ok());
    }
}
