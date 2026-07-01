use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use serde_json::Value;
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
    let domain = email.split('@').nth(1).ok_or_else(|| -> Error {
        String::from("メールアドレスの形式が不正です").into()
    })?;

    Ok(allowed_domains.split(',').any(|d| d.trim() == domain))
}

/// Cognitoサインアップ前トリガーのハンドラ。
///
/// メールアドレスのドメイン部が `ALLOWED_EMAIL_DOMAINS` 環境変数に含まれているか検証する。
/// 許可されていないドメインの場合はエラーを返し、Cognitoのサインアップを拒否する。
async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let allowed_domains = env::var("ALLOWED_EMAIL_DOMAINS").map_err(|_| -> Error {
        // 環境変数が未設定の場合はデプロイ設定の問題であるため、明示的なエラーを返す
        String::from("ALLOWED_EMAIL_DOMAINS 環境変数が設定されていません").into()
    })?;

    let email = event.payload["request"]["userAttributes"]["email"]
        .as_str()
        .ok_or_else(|| -> Error {
            String::from("リクエストにメールアドレスが含まれていません").into()
        })?;

    if is_allowed_domain(email, &allowed_domains)? {
        Ok(event.payload)
    } else {
        let domain = email.split('@').nth(1).unwrap_or("");
        // ドメインが許可リストに含まれていないためサインアップを拒否する
        Err(format!("メールアドレスのドメイン '{}' は許可されていません", domain).into())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
