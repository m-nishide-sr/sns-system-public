use aws_lambda_events::{
    apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse},
    encodings::Body,
    http::{HeaderMap, HeaderName, HeaderValue},
};
use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult, SystemClock};
use core_infrastructure_sea_orm::SeaOrmMessageRepository;
use core_usecase::{
    GetTimelineInput, GetTimelineUseCase, PostMessageInput, PostMessageUseCase, TimelineItem,
};
use lambda_runtime::{Error, LambdaEvent};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

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
    path = "/api/v1/message",
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "投稿成功", body = CreateMessageResponse),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 401, description = "認証エラー", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse)
    ),
    security(
        ("CognitoJwtAuth" = [])
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
    )),
    security(
        ("CognitoJwtAuth" = [])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// `CognitoJwtAuth` の詳細定義を設定する構造体
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("componentsが存在する想定");
        components.add_security_scheme(
            "CognitoJwtAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some(
                        "Cognitoから発行されたIDトークンまたはアクセストークンを入力してください",
                    ))
                    .build(),
            ),
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// API Gateway の生イベントをユースケース単位へ振り分ける内部ルート種別。
enum Route {
    GetTimeline,
    PostMessageNew,
    MethodNotAllowed,
    NotFound,
}

/// API Gateway のパスとメソッドから処理対象ルートを判定する。
fn route_request(event: &ApiGatewayV2httpRequest) -> Route {
    let path = event
        .raw_path
        .as_deref()
        .or(event.request_context.http.path.as_deref())
        .unwrap_or("");

    let method = event.request_context.http.method.as_str();

    match path {
        "/api/v1/timeline" => {
            if method == "GET" {
                Route::GetTimeline
            } else {
                Route::MethodNotAllowed
            }
        }
        "/api/v1/message" => {
            if method == "POST" {
                Route::PostMessageNew
            } else {
                Route::MethodNotAllowed
            }
        }
        _ => Route::NotFound,
    }
}

/// API Gateway イベントから機微情報をマスクしたJSON文字列を生成する
fn serde_to_sanitized_log_string(event: &ApiGatewayV2httpRequest) -> String {
    let mut sanitized_event = event.clone();
    let sensitive_keys = ["authorization", "cookie", "set-cookie", "x-api-key"];

    for (key, value) in sanitized_event.headers.iter_mut() {
        if sensitive_keys.contains(&key.as_str()) {
            *value = HeaderValue::from_static("[MASKED]");
        }
    }

    serde_json::to_string(&sanitized_event).unwrap_or_else(|_| {
        tracing::warn!("Failed to serialize API Gateway event for row_log");
        "{}".to_string()
    })
}

fn extract_auth_info(event: &ApiGatewayV2httpRequest) -> Option<(String, uuid::Uuid)> {
    let auth = event.request_context.authorizer.as_ref()?;
    let jwt = auth.jwt.as_ref()?;

    let email = jwt.claims.get("email")?.to_string();
    if email.is_empty() {
        return None;
    }

    let cognito_sub = jwt.claims.get("sub")?.to_string();
    if cognito_sub.is_empty() {
        return None;
    }

    match uuid::Uuid::parse_str(&cognito_sub) {
        Ok(cognito_sub) => Some((email, cognito_sub)),
        Err(err) => {
            tracing::warn!("Invalid cognito_sub in JWT claims: {}", err);
            None
        }
    }
}

/// Lambdaエントリポイントから呼び出されるHTTPハンドラ。
pub async fn function_handler(
    db: &DatabaseConnection,
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let (event, _context) = event.into_parts();
    let route = route_request(&event);

    let row_log = serde_to_sanitized_log_string(&event);
    let repository = SeaOrmMessageRepository::new(db.clone());
    let auth_info = extract_auth_info(&event);

    if auth_info.is_none() {
        return json_response(
            401,
            &ErrorResponse {
                message: "Unauthorized".to_string(),
            },
        );
    }

    match route {
        Route::GetTimeline => {
            let before = match parse_before(event.query_string_parameters.first("before")) {
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

            json_response(200, &response)
        }
        Route::PostMessageNew => {
            let (email, cognito_sub) = match auth_info {
                Some((email, cognito_sub)) => (email, cognito_sub),
                None => {
                    return json_response(
                        401,
                        &ErrorResponse {
                            message: "Unauthorized".to_string(),
                        },
                    );
                }
            };

            let raw_body = match event.body {
                Some(body) => body,
                None => {
                    return json_response(
                        400,
                        &ErrorResponse {
                            message: "Bad Request".to_string(),
                        },
                    );
                }
            };

            let request: CreateMessageRequest = match serde_json::from_str(&raw_body) {
                Ok(value) => value,
                Err(_) => {
                    return json_response(
                        400,
                        &ErrorResponse {
                            message: "Bad Request".to_string(),
                        },
                    );
                }
            };
            let Some(name) = email.split('@').next() else {
                return json_response(
                    400,
                    &ErrorResponse {
                        message: "登録しているemailアドレスが不正です".to_string(),
                    },
                );
            };
            let user_name = name.to_string();

            let usecase = PostMessageUseCase::new(repository, SystemClock);
            let output = match usecase
                .execute(PostMessageInput {
                    user_id: cognito_sub,
                    user_name,
                    body: request.body,
                    row_log,
                    is_from_user: true,
                })
                .await
            {
                Ok(output) => output,
                Err(error) => return Ok(error_response(error)),
            };

            json_response(
                201,
                &CreateMessageResponse {
                    status: output.status,
                    message: output.message,
                },
            )
        }
        Route::NotFound => json_response(
            404,
            &ErrorResponse {
                message: "not found".to_string(),
            },
        ),
        Route::MethodNotAllowed => json_response(
            405,
            &ErrorResponse {
                message: "method not allowed".to_string(),
            },
        ),
    }
}

/// `before` クエリ文字列を ISO8601 日時として検証し、UTC に正規化する。
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

/// 指定したステータスコードと JSON ペイロードから API Gateway 応答を組み立てる。
fn json_response<T: Serialize>(
    status: i64,
    payload: &T,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let body = serde_json::to_string(payload)?;
    let headers = HeaderMap::from_iter(vec![(
        HeaderName::from_static("content-type"),
        "application/json"
            .parse()
            .expect("固定値ヘッダーの変換に失敗しない想定"),
    )]);

    let mut response: ApiGatewayV2httpResponse = ApiGatewayV2httpResponse::default();
    response.body = Some(Body::Text(body));
    response.status_code = status;
    response.headers = headers;
    Ok(response)
}

/// 共通エラー型を HTTP ステータスと公開用 JSON ボディへ変換する。
fn error_response(error: CoreError) -> ApiGatewayV2httpResponse {
    let (status, body) = match error {
        CoreError::Validation(_) => (400, r#"{"message":"Bad Request"}"#.to_string()),
        CoreError::Unauthorized => (401, r#"{"message":"Unauthorized"}"#.to_string()),
        CoreError::Infrastructure(_) => (500, r#"{"message":"Internal Server Error"}"#.to_string()),
    };

    let mut response: ApiGatewayV2httpResponse = ApiGatewayV2httpResponse::default();
    response.body = Some(Body::Text(body));
    response.headers = HeaderMap::from_iter(vec![(
        HeaderName::from_static("content-type"),
        "application/json"
            .parse()
            .expect("固定値ヘッダーの変換に失敗しない想定"),
    )]);
    response.status_code = status;
    response
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
    use aws_lambda_events::http::Method;
    use chrono::TimeZone;
    use utoipa::OpenApi;

    fn request_with(path: &str, method: Method) -> ApiGatewayV2httpRequest {
        let mut req = ApiGatewayV2httpRequest::default();
        req.raw_path = Some(path.to_string());
        req.request_context.http.method = method;
        req
    }

    #[test]
    fn route_requestは_get_timelineを判定できる() {
        let req = request_with("/api/v1/timeline", Method::GET);
        assert_eq!(route_request(&req), Route::GetTimeline);
    }

    #[test]
    fn route_requestは_messageの不正メソッドを405扱いにする() {
        let req = request_with("/api/v1/message", Method::GET);
        assert_eq!(route_request(&req), Route::MethodNotAllowed);
    }

    #[test]
    fn route_requestは未知パスを_not_foundにする() {
        let req = request_with("/api/unknown", Method::GET);
        assert_eq!(route_request(&req), Route::NotFound);
    }

    #[test]
    fn parse_beforeは未指定で_noneを返す() {
        assert_eq!(parse_before(None).unwrap(), None);
    }

    #[test]
    fn parse_beforeは正常な_rfc3339を_utcへ変換する() {
        let actual = parse_before(Some("2026-07-01T00:00:00+09:00")).unwrap();
        let expected = Utc
            .with_ymd_and_hms(2026, 6, 30, 15, 0, 0)
            .single()
            .expect("固定時刻の生成に失敗しない想定");
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn parse_beforeは不正値で_validationエラーを返す() {
        let err = parse_before(Some("invalid")).expect_err("不正な日時はエラー想定");
        assert!(matches!(err, CoreError::Validation(_)));
    }

    #[test]
    fn serde_to_sanitized_log_stringは機微ヘッダーをマスクする() {
        let mut req = request_with("/api/v1/message", Method::POST);
        req.headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("******"),
        );
        req.headers.insert(
            HeaderName::from_static("x-api-key"),
            HeaderValue::from_static("secret"),
        );
        req.headers.insert(
            HeaderName::from_static("x-trace-id"),
            HeaderValue::from_static("trace-1"),
        );

        let serialized = serde_to_sanitized_log_string(&req);
        let value: serde_json::Value =
            serde_json::from_str(&serialized).expect("JSON化されたログ文字列の想定");

        assert_eq!(value["headers"]["authorization"], "[MASKED]");
        assert_eq!(value["headers"]["x-api-key"], "[MASKED]");
        assert_eq!(value["headers"]["x-trace-id"], "trace-1");
    }

    #[test]
    fn json_responseは_jsonヘッダーと本文を返す() {
        let response = json_response(
            200,
            &ErrorResponse {
                message: "ok".to_string(),
            },
        )
        .expect("正常系で失敗しない想定");

        assert_eq!(response.status_code, 200);
        assert_eq!(
            response
                .headers
                .get("content-type")
                .expect("content-typeが存在する想定"),
            "application/json"
        );
        assert_eq!(
            response.body,
            Some(Body::Text(r#"{"message":"ok"}"#.to_string()))
        );
    }

    #[test]
    fn error_responseは_validationを400へ変換する() {
        let response = error_response(CoreError::Validation("x".to_string()));
        assert_eq!(response.status_code, 400);
    }

    #[test]
    fn error_responseは_unauthorizedを401へ変換する() {
        let response = error_response(CoreError::Unauthorized);
        assert_eq!(response.status_code, 401);
    }

    #[test]
    fn error_responseは_infrastructureを500へ変換する() {
        let response = error_response(CoreError::Infrastructure("db".to_string()));
        assert_eq!(response.status_code, 500);
    }

    #[test]
    fn security_addonは_bearer認証を追加する() {
        let mut openapi = ApiDoc::openapi();
        SecurityAddon.modify(&mut openapi);
        let components = openapi.components.expect("componentsが存在する想定");
        assert!(components.security_schemes.contains_key("CognitoJwtAuth"));
    }

    #[test]
    fn extract_auth_infoは_claimsからemailとsubを取得する() {
        let mut req = request_with("/api/v1/timeline", Method::GET);
        let sub = uuid::Uuid::now_v7().to_string();
        req.request_context.authorizer = Some(
            serde_json::from_value(serde_json::json!({
                "jwt": {
                    "claims": {
                        "email": "user@example.com",
                        "sub": sub
                    },
                    "scopes": ["read"]
                }
            }))
            .expect("authorizer生成に失敗しない想定"),
        );

        let actual = extract_auth_info(&req).expect("claimsが揃っている想定");
        assert_eq!(actual.0, "user@example.com");
    }

    #[test]
    fn extract_auth_infoはsubがuuid不正なら_noneを返す() {
        let mut req = request_with("/api/v1/timeline", Method::GET);
        req.request_context.authorizer = Some(
            serde_json::from_value(serde_json::json!({
                "jwt": {
                    "claims": {
                        "email": "user@example.com",
                        "sub": "not-uuid"
                    },
                    "scopes": ["read"]
                }
            }))
            .expect("authorizer生成に失敗しない想定"),
        );

        assert!(extract_auth_info(&req).is_none());
    }
}
