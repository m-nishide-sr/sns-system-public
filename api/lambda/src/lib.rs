//! API-LambdaのHTTPハンドラとOpenAPI定義。

use aws_lambda_events::{
    apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse},
    encodings::Body,
    http::{HeaderMap, HeaderName},
};
use chrono::{DateTime, Utc};
use core_common::{CoreError, CoreResult, SystemClock};
use core_infrastructure::SeaOrmMessageRepository;
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
        let components = openapi.components.as_mut().unwrap();
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
enum Route {
    GetTimeline,
    PostMessageNew,
    MethodNotAllowed,
    NotFound,
}

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

/// Lambdaエントリポイントから呼び出されるHTTPハンドラ。
pub async fn function_handler(
    db: &DatabaseConnection,
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let (event, _context) = event.into_parts();
    let route = route_request(&event);

    // API Gateway のイベント全体をシリアライズして `row_log` に永続化します。
    //
    // # ⚠️ セキュリティに関する注意 (Security Notice)
    // 本関数は、Authorization ヘッダや Cookie などの機微情報（PII/認証情報）を
    // マスクせずにそのままシリアライズして保存します。これは一般的なセキュリティベスト
    // プラクティスに反しますが、**非常時の監査およびトラブルシューティングにおける
    // リクエストの完全性（Integrity）を担保するため**に意図された仕様です。
    //
    // 認証情報等が除外されると、有事の際（不正アクセス調査や認証不良の再現など）に
    // 厳密な原因究明や証拠保全（フォレンジック）が不可能になるリスクがあります。
    let row_log = serde_json::to_string(&event).unwrap_or_else(|_| {
        tracing::warn!("Failed to serialize API Gateway event for row_log");
        "{}".to_string()
    });

    let repository = SeaOrmMessageRepository::new(db.clone());

    // authorizer 内の JWT claims から値を取得
    let auth_info = event.request_context.authorizer.as_ref().and_then(|auth| {
        // email と cognito_sub (または sub) を取得
        // (クレートのバージョンによって値が String か serde_json::Value か異なるため、安全に文字列化します)
        let email = auth.jwt.as_ref()?.claims.get("email").cloned()?;
        if email.is_empty() {
            return None;
        }
        let cognito_sub = auth.jwt.as_ref()?.claims.get("sub").cloned()?;
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
    });

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

            let body = match event.body {
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

            let usecase = PostMessageUseCase::new(repository, SystemClock);
            let output = match usecase
                .execute(PostMessageInput {
                    user_id: cognito_sub,
                    user_name: email.to_string(),
                    body,
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

fn json_response<T: Serialize>(
    status: i64,
    payload: &T,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let body = serde_json::to_string(payload)?;
    let headers = HeaderMap::from_iter(vec![(
        HeaderName::from_static("content-type"),
        "application/json".parse().unwrap(),
    )]);

    let mut response: ApiGatewayV2httpResponse = ApiGatewayV2httpResponse::default();
    response.body = Some(Body::Text(body));
    response.status_code = status;
    response.headers = headers;
    Ok(response)
}

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
        "application/json".parse().unwrap(),
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
