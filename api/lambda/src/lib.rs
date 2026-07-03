//! API-LambdaのHTTPハンドラとOpenAPI定義。

use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult, SystemClock};
use core_infrastructure::SeaOrmMessageRepository;
use core_usecase::{
    GetTimelineInput, GetTimelineUseCase, PostMessageInput, PostMessageUseCase, TimelineItem,
};
use lambda_http::{Body, Error, Request, RequestExt, Response, http::StatusCode};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

/// タイムライン取得レスポンス。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TimelineMessageResponse {
    /// 投稿者名。
    pub user_name: String,
    /// 投稿日時。
    pub created_at: DateTime<Utc>,
    /// 投稿本文。
    pub body: String,
    /// 利用者投稿かどうか。
    pub is_from_user: bool,
}

/// メッセージ投稿リクエスト。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct CreateMessageRequest {
    /// 投稿本文。
    pub body: String,
}

/// メッセージ投稿レスポンス。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct CreateMessageResponse {
    /// 処理ステータス。
    pub status: String,
    /// 応答メッセージ。
    pub message: String,
}

/// エラーレスポンス。
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct ErrorResponse {
    /// エラー概要。
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct JwtClaims {
    sub: String,
    email: String,
}

/// タイムライン取得APIのOpenAPI定義。
#[utoipa::path(
    get,
    path = "/api/v1/timeline",
    params(
        ("before" = Option<String>, Query, description = "ISO8601形式の境界日時")
    ),
    responses(
        (status = 200, description = "タイムライン取得成功", body = [TimelineMessageResponse]),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 401, description = "認証エラー", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse)
    )
)]
pub fn timeline_endpoint_doc() {}

/// 投稿APIのOpenAPI定義。
#[utoipa::path(
    post,
    path = "/api/v1/messages",
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "投稿成功", body = CreateMessageResponse),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 401, description = "認証エラー", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse)
    )
)]
pub fn create_message_endpoint_doc() {}

/// OpenAPIドキュメントのエントリポイント。
#[derive(OpenApi)]
#[openapi(
    paths(timeline_endpoint_doc, create_message_endpoint_doc),
    components(schemas(
        TimelineMessageResponse,
        CreateMessageRequest,
        CreateMessageResponse,
        ErrorResponse
    ))
)]
pub struct ApiDoc;

/// Lambdaエントリポイントから呼び出されるHTTPハンドラ。
pub async fn function_handler(
    db: &DatabaseConnection,
    request: Request,
) -> Result<Response<Body>, Error> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let repository = SeaOrmMessageRepository::new(db.clone());

    match (method.as_str(), path.as_str()) {
        ("GET", "/api/v1/timeline") => {
            if let Err(error) = extract_auth_info(&request) {
                return Ok(error_response(error));
            }
            let before = match parse_before(request.query_string_parameters().first("before")) {
                Ok(before) => before,
                Err(error) => return Ok(error_response(error)),
            };
            let usecase = GetTimelineUseCase::new(repository);
            let output = match usecase
                .execute(GetTimelineInput { before, limit: 50 })
                .await
            {
                Ok(output) => output,
                Err(error) => return Ok(error_response(error)),
            };

            let response: Vec<TimelineMessageResponse> = output
                .items
                .into_iter()
                .map(TimelineMessageResponse::from)
                .collect();

            json_response(StatusCode::OK, &response)
        }
        ("POST", "/api/v1/messages") => {
            let auth = match extract_auth_info(&request) {
                Ok(auth) => auth,
                Err(error) => return Ok(error_response(error)),
            };
            let body = match parse_create_message_request(request.body()) {
                Ok(body) => body,
                Err(error) => return Ok(error_response(error)),
            };
            let row_log = build_row_log(&auth.user_id);

            let usecase = PostMessageUseCase::new(repository, SystemClock);
            let output = match usecase
                .execute(PostMessageInput {
                    user_id: auth.user_id,
                    user_name: auth.user_name,
                    body: body.body,
                    row_log,
                    is_from_user: true,
                })
                .await
            {
                Ok(output) => output,
                Err(error) => return Ok(error_response(error)),
            };

            json_response(
                StatusCode::CREATED,
                &CreateMessageResponse {
                    status: output.status,
                    message: output.message,
                },
            )
        }
        _ => json_response(
            StatusCode::NOT_FOUND,
            &ErrorResponse {
                message: "not found".to_string(),
            },
        ),
    }
}

#[derive(Debug, Clone)]
struct AuthInfo {
    user_id: Uuid,
    user_name: String,
}

fn parse_before(before: Option<&str>) -> CoreResult<Option<DateTime<Utc>>> {
    before
        .map(|v| {
            DateTime::parse_from_rfc3339(v)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    CoreError::Validation("beforeはISO8601形式で指定してください".to_string())
                })
        })
        .transpose()
}

fn parse_create_message_request(body: &Body) -> CoreResult<CreateMessageRequest> {
    let body_str = match body {
        Body::Text(text) => text.to_string(),
        Body::Binary(bytes) => String::from_utf8_lossy(bytes).to_string(),
        Body::Empty => String::new(),
    };

    serde_json::from_str(&body_str)
        .map_err(|_| CoreError::Validation("リクエストボディが不正です".to_string()))
}

fn extract_auth_info(request: &Request) -> CoreResult<AuthInfo> {
    let authorization = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(CoreError::Unauthorized)?;

    let token = authorization
        .strip_prefix("Bearer ")
        .ok_or(CoreError::Unauthorized)?;

    let payload = token.split('.').nth(1).ok_or(CoreError::Unauthorized)?;

    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| general_purpose::URL_SAFE.decode(payload))
        .map_err(|_| CoreError::Unauthorized)?;

    let claims: JwtClaims =
        serde_json::from_slice(&decoded).map_err(|_| CoreError::Unauthorized)?;
    let user_name = claims
        .email
        .split('@')
        .next()
        .filter(|v| !v.trim().is_empty())
        .ok_or(CoreError::Unauthorized)?;

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| CoreError::Unauthorized)?;

    Ok(AuthInfo {
        user_id,
        user_name: user_name.to_string(),
    })
}

fn build_row_log(user_id: &Uuid) -> String {
    format!("source=api,user_id={user_id}")
}

fn json_response<T: Serialize>(status: StatusCode, payload: &T) -> Result<Response<Body>, Error> {
    let body = serde_json::to_string(payload)?;

    let response = Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(body))?;

    Ok(response)
}

fn error_response(error: CoreError) -> Response<Body> {
    let status = match error {
        CoreError::Validation(_) => StatusCode::BAD_REQUEST,
        CoreError::Unauthorized => StatusCode::UNAUTHORIZED,
        CoreError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    json_response(
        status,
        &ErrorResponse {
            message: error.to_string(),
        },
    )
    .unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::Text(
                r#"{"message":"internal server error"}"#.to_string(),
            ))
            .expect("固定レスポンスの生成に失敗しない想定")
    })
}

impl From<TimelineItem> for TimelineMessageResponse {
    fn from(value: TimelineItem) -> Self {
        Self {
            user_name: value.user_name,
            created_at: value.created_at,
            body: value.body,
            is_from_user: value.is_from_user,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::header::AUTHORIZATION;

    #[test]
    fn jwtから認証情報を抽出できる() {
        let payload =
            r#"{"sub":"12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d","email":"taro@example.com"}"#;
        let token = format!(
            "header.{}.signature",
            general_purpose::URL_SAFE_NO_PAD.encode(payload.as_bytes())
        );

        let authorization_header = "Bearer ".to_owned() + &token;
        let request: Request = http::Request::builder()
            .header(AUTHORIZATION, authorization_header)
            .body(Body::Empty)
            .expect("リクエスト生成に失敗しない想定");

        let auth = extract_auth_info(&request).expect("認証情報を抽出できる想定");
        assert_eq!(auth.user_name, "taro");
    }
}
