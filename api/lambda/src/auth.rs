use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use lambda_http::Request;
use sns_core::common::error::CoreError;
use std::collections::HashMap;

/// JWTから抽出した認証済みユーザー情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// Cognitoのsubject。
    pub cognito_id: String,
    /// 画面表示に利用するユーザー名。
    pub user_name: String,
}

/// API Gatewayで検証済みのBearerトークンからクレームを取り出す。
pub fn extract_authenticated_user(request: &Request) -> Result<AuthenticatedUser, CoreError> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| CoreError::BadRequest("Authorizationヘッダーが必要です".to_owned()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| CoreError::BadRequest("Authorization形式が不正です".to_owned()))?;

    let claims = decode_claims(token)?;

    let cognito_id = claims
        .get("sub")
        .cloned()
        .ok_or_else(|| CoreError::BadRequest("JWTのsubクレームが不足しています".to_owned()))?;

    let user_name = claims
        .get("email")
        .and_then(|email| email.split('@').next())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| claims.get("cognito:username").cloned())
        .ok_or_else(|| {
            CoreError::BadRequest("JWTにユーザー名を判断できるクレームがありません".to_owned())
        })?;

    Ok(AuthenticatedUser {
        cognito_id,
        user_name,
    })
}

fn decode_claims(token: &str) -> Result<HashMap<String, String>, CoreError> {
    let mut segments = token.split('.');
    let _header = segments.next();
    let payload = segments
        .next()
        .ok_or_else(|| CoreError::BadRequest("JWT形式が不正です".to_owned()))?;

    let bytes = URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .map_err(|_| CoreError::BadRequest("JWTペイロードのデコードに失敗しました".to_owned()))?;

    let claims = serde_json::from_slice::<HashMap<String, serde_json::Value>>(&bytes)
        .map_err(|_| CoreError::BadRequest("JWTクレームのJSON解析に失敗しました".to_owned()))?
        .into_iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_owned())))
        .collect::<HashMap<_, _>>();

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{Body, Request, http};

    fn build_request(token: &str) -> Request {
        let authorization = ["Bearer", token].join(" ");
        http::Request::builder()
            .uri("https://example.com/api/v1/messages")
            .header("authorization", authorization)
            .body(Body::Empty)
            .expect("リクエスト生成は成功するべき")
    }

    #[test]
    fn emailクレームからユーザー名を抽出できる() {
        let payload =
            r#"{"sub":"12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d","email":"tanaka@example.com"}"#;
        let token = format!("aaa.{}.bbb", URL_SAFE_NO_PAD.encode(payload.as_bytes()));
        let request = build_request(&token);

        let user = extract_authenticated_user(&request).expect("認証情報が抽出できるべき");

        assert_eq!(user.user_name, "tanaka");
        assert_eq!(user.cognito_id, "12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d");
    }

    #[test]
    fn authorizationヘッダーがない場合はエラー() {
        let request = http::Request::builder()
            .uri("https://example.com/api/v1/messages")
            .body(Body::Empty)
            .expect("リクエスト生成は成功するべき");

        let result = extract_authenticated_user(&request);

        assert!(matches!(result, Err(CoreError::BadRequest(_))));
    }
}
