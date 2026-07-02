use chrono::{DateTime, Utc};
use lambda_http::{Body, Error, Request, RequestPayloadExt, Response, http::StatusCode};
use serde::{Deserialize, Serialize};
use sns_core::{
    common::error::CoreError,
    domain::repository::MessageRepository,
    usecase::{
        post_message::PostMessageInput, post_message::PostMessageUsecase,
        timeline::GetTimelineUsecase,
    },
};
use url::form_urlencoded;
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::{
        Components,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};
use uuid::Uuid;

use crate::auth::extract_authenticated_user;

#[derive(Debug, Deserialize, ToSchema)]
struct CreateMessageRequest {
    body: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct CreateMessageResponse {
    status: &'static str,
    message: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
struct TimelineMessageResponse {
    user_name: String,
    #[schema(format = DateTime)]
    created_at: String,
    body: String,
    is_from_user: bool,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    message: String,
}

struct BearerSecurity;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "SNS API", version = "1.0.0"),
    paths(get_timeline_doc, post_message_doc),
    components(schemas(
        CreateMessageRequest,
        CreateMessageResponse,
        TimelineMessageResponse,
        ErrorResponse
    )),
    modifiers(&BearerSecurity)
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/api/v1/timeline",
    params(
        ("before" = Option<String>, Query, format = DateTime, description = "取得対象の上限日時（ISO8601）")
    ),
    responses(
        (status = 200, description = "OK", body = [TimelineMessageResponse]),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 500, description = "内部エラー", body = ErrorResponse)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[allow(dead_code)]
async fn get_timeline_doc() {}

#[utoipa::path(
    post,
    path = "/api/v1/messages",
    request_body(
        content = CreateMessageRequest,
        content_type = "application/json",
        description = "投稿メッセージ"
    ),
    responses(
        (status = 201, description = "Created", body = CreateMessageResponse),
        (status = 400, description = "不正なリクエスト", body = ErrorResponse),
        (status = 500, description = "内部エラー", body = ErrorResponse)
    ),
    security(
        ("bearerAuth" = [])
    )
)]
#[allow(dead_code)]
async fn post_message_doc() {}

/// API Gatewayイベントを処理する。
pub async fn function_handler<R: MessageRepository>(
    repository: &R,
    request: Request,
) -> Result<Response<Body>, Error> {
    match (request.method().as_str(), request.uri().path()) {
        ("GET", "/api/v1/timeline") => handle_get_timeline(repository, request).await,
        ("POST", "/api/v1/messages") => handle_post_message(repository, request).await,
        _ => json_response(
            StatusCode::NOT_FOUND,
            serde_json::json!({"message": "Not Found"}),
        ),
    }
}

async fn handle_get_timeline<R: MessageRepository>(
    repository: &R,
    request: Request,
) -> Result<Response<Body>, Error> {
    let _ = extract_authenticated_user(&request).map_err(core_error_to_http)?;

    let before = request
        .uri()
        .query()
        .and_then(parse_before_query)
        .transpose()
        .map_err(core_error_to_http)?;

    let usecase = GetTimelineUsecase::new(repository);
    let timeline = usecase.execute(before).await.map_err(core_error_to_http)?;

    json_response(StatusCode::OK, timeline)
}

async fn handle_post_message<R: MessageRepository>(
    repository: &R,
    request: Request,
) -> Result<Response<Body>, Error> {
    let user = extract_authenticated_user(&request).map_err(core_error_to_http)?;

    let cognito_id = Uuid::parse_str(&user.cognito_id)
        .map_err(|_| {
            core_error_to_http(CoreError::BadRequest(
                "subクレームがUUID形式ではありません".to_owned(),
            ))
        })?
        .to_string();

    let payload = request.payload::<CreateMessageRequest>().map_err(|e| {
        core_error_to_http(CoreError::BadRequest(format!(
            "JSONの解析に失敗しました: {e}"
        )))
    })?;

    let payload = payload.ok_or_else(|| {
        core_error_to_http(CoreError::BadRequest(
            "リクエストボディが必要です".to_owned(),
        ))
    })?;

    let row_log = serde_json::json!({
        "path": request.uri().path(),
        "method": request.method().as_str(),
        "cognito_id": cognito_id,
    })
    .to_string();

    let usecase = PostMessageUsecase::new(repository);
    let output = usecase
        .execute(PostMessageInput {
            body: payload.body,
            user_name: user.user_name,
            cognito_id,
            row_log,
        })
        .await
        .map_err(core_error_to_http)?;

    json_response(
        StatusCode::CREATED,
        CreateMessageResponse {
            status: output.status,
            message: output.message,
        },
    )
}

fn parse_before_query(query: &str) -> Option<Result<DateTime<Utc>, CoreError>> {
    form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "before")
        .map(|(_, value)| {
            DateTime::parse_from_rfc3339(&value)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    CoreError::BadRequest(
                        "beforeはISO8601形式（例: 2026-07-02T02:24:00Z）で指定してください"
                            .to_owned(),
                    )
                })
        })
}

fn core_error_to_http(error: CoreError) -> Error {
    match error {
        CoreError::Validation(message) | CoreError::BadRequest(message) => {
            anyhow::anyhow!(message).into()
        }
        CoreError::Repository(message) => {
            tracing::error!("Repository error: {message}");
            anyhow::anyhow!("内部エラーが発生しました").into()
        }
    }
}

fn json_response<T: Serialize>(status: StatusCode, body: T) -> Result<Response<Body>, Error> {
    let response_body = serde_json::to_string(&body)?;

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(response_body))
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beforeクエリがない場合はnone() {
        assert!(parse_before_query("foo=bar").is_none());
    }

    #[test]
    fn beforeクエリが有効な場合はutc日時を返す() {
        let parsed = parse_before_query("before=2026-07-02T02%3A24%3A00Z")
            .expect("beforeは取得できるべき")
            .expect("日時変換は成功するべき");

        assert_eq!(parsed.to_rfc3339(), "2026-07-02T02:24:00+00:00");
    }

    #[test]
    fn beforeクエリが不正な場合はエラー() {
        let result = parse_before_query("before=invalid").expect("beforeは取得できるべき");

        assert!(matches!(result, Err(CoreError::BadRequest(_))));
    }
}
