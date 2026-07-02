//! HTTPハンドラ実装。
//!
//! タイムライン取得API（GET /api/v1/timeline）と
//! メッセージ投稿API（POST /api/v1/messages）のaxumハンドラを定義します。
//!
//! 認証はAPI Gateway HTTP APIのJWTオーソライザーで行い、
//! Lambda呼び出し時にリクエストコンテキスト経由でJWTクレームを受け取ります。

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use chrono::{DateTime, Utc};
use lambda_http::request::RequestContext;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;
use utoipa::ToSchema;

use crate::application::{GetTimelineUseCase, MessageRepository, PostMessageUseCase};
use crate::domain::{NewMessage, RepositoryError};

/// axumルーターで共有するアプリケーション状態。
#[derive(Clone)]
pub struct AppState {
    /// メッセージRepositoryの共有参照。
    pub repository: Arc<dyn MessageRepository>,
}

/// HTTPレスポンス用エラー型。
///
/// RepositoryエラーをHTTPステータスコードと共に返します。
#[derive(Debug, Error)]
pub enum AppError {
    /// JWT認証情報が取得できない場合（401 Unauthorized）。
    #[error("認証情報が取得できません: {0}")]
    Unauthorized(String),
    /// リクエストのバリデーションエラー（400 Bad Request）。
    #[error("リクエストが不正です: {0}")]
    BadRequest(String),
    /// Repositoryエラー（500 Internal Server Error）。
    #[error("サーバーエラー: {0}")]
    Repository(#[from] RepositoryError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Repository(e) => {
                eprintln!("Repositoryエラー: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部エラーが発生しました".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

/// JWTクレームから取得したユーザー情報。
struct JwtClaims {
    /// CognitoサブジェクトID（UUID形式）。
    sub: String,
    /// Cognitoユーザー名（emailアドレス）。
    username: String,
}

/// API GatewayのリクエストコンテキストからJWTクレームを取得します。
///
/// API Gateway HTTP APIのJWTオーソライザーが検証済みのクレームを
/// `requestContext.authorizer.jwt.claims` に格納します。
fn extract_jwt_claims(req_ctx: &RequestContext) -> Result<JwtClaims, AppError> {
    let claims_map = match req_ctx {
        RequestContext::ApiGatewayV2(ctx) => ctx
            .authorizer
            .as_ref()
            .and_then(|a| a.jwt.as_ref())
            .map(|j| &j.claims)
            .ok_or_else(|| AppError::Unauthorized("JWTクレームが見つかりません".to_string()))?,
        _ => {
            return Err(AppError::Unauthorized(
                "API Gateway HTTP APIのリクエストコンテキストが必要です".to_string(),
            ));
        }
    };

    let sub = claims_map
        .get("sub")
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("JWTにsubクレームがありません".to_string()))?;
    let username = claims_map.get("cognito:username").cloned().ok_or_else(|| {
        AppError::Unauthorized("JWTにcognito:usernameクレームがありません".to_string())
    })?;

    Ok(JwtClaims { sub, username })
}

/// emailアドレスの@以前の文字列を取得します。
///
/// DB定義の `user_name` カラムは「emailアドレスの@以前の文字列」を格納します。
fn extract_local_part(email: &str) -> &str {
    email.split('@').next().unwrap_or(email)
}

// ─── タイムライン取得API ─────────────────────────────────────────────────────

/// タイムライン取得APIのクエリパラメータ。
#[derive(Debug, Deserialize, ToSchema)]
pub struct TimelineQuery {
    /// この日時より前のメッセージを取得します（ISO8601形式のUTC）。
    pub before: Option<DateTime<Utc>>,
}

/// タイムライン取得APIのレスポンスアイテム。
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimelineItem {
    /// 投稿ユーザー名。
    pub user_name: String,
    /// 投稿日時（UTCのZ形式）。
    pub created_at: DateTime<Utc>,
    /// メッセージ本文。
    pub body: String,
    /// `true`: ユーザー投稿, `false`: システム投稿。
    pub is_from_user: bool,
}

/// タイムライン取得ハンドラ（GET /api/v1/timeline）。
///
/// `public.messages_latest` ビューから最新50件を取得します。
#[utoipa::path(
    get,
    path = "/api/v1/timeline",
    params(
        ("before" = Option<DateTime<Utc>>, Query, description = "この日時より前のメッセージを取得（ISO8601 UTC）")
    ),
    responses(
        (status = 200, description = "タイムライン取得成功", body = Vec<TimelineItem>),
        (status = 401, description = "認証エラー"),
        (status = 500, description = "サーバーエラー")
    ),
    security(("jwt" = []))
)]
pub async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<impl IntoResponse, AppError> {
    let use_case = GetTimelineUseCase::new(state.repository);
    let messages = use_case.execute(query.before).await?;
    let items: Vec<TimelineItem> = messages
        .into_iter()
        .map(|m| TimelineItem {
            user_name: m.user_name,
            created_at: m.created_at,
            body: m.body,
            is_from_user: m.is_from_user,
        })
        .collect();
    Ok((StatusCode::OK, Json(items)))
}

// ─── メッセージ投稿API ───────────────────────────────────────────────────────

/// メッセージ投稿APIのリクエストボディ。
#[derive(Debug, Deserialize, ToSchema)]
pub struct PostMessageRequest {
    /// メッセージ本文。
    pub body: String,
}

/// メッセージ投稿APIのレスポンス。
#[derive(Debug, Serialize, ToSchema)]
pub struct PostMessageResponse {
    /// 処理結果。
    pub status: String,
    /// メッセージ。
    pub message: String,
}

/// メッセージ投稿ハンドラ（POST /api/v1/messages）。
///
/// `public.messages` テーブルに新規メッセージを挿入します。
/// JWTクレームから `cognito_id` と `user_name` を取得します。
#[utoipa::path(
    post,
    path = "/api/v1/messages",
    request_body = PostMessageRequest,
    responses(
        (status = 201, description = "メッセージ投稿成功", body = PostMessageResponse),
        (status = 400, description = "リクエストエラー"),
        (status = 401, description = "認証エラー"),
        (status = 500, description = "サーバーエラー")
    ),
    security(("jwt" = []))
)]
pub async fn post_message(
    State(state): State<AppState>,
    req_ctx_ext: axum::Extension<RequestContext>,
    Json(body): Json<PostMessageRequest>,
) -> Result<impl IntoResponse, AppError> {
    // JWTクレームからユーザー情報を取得する
    let claims = extract_jwt_claims(&req_ctx_ext.0)?;
    let cognito_id = claims
        .sub
        .parse::<uuid::Uuid>()
        .map_err(|e| AppError::BadRequest(format!("cognito_idのUUID変換に失敗: {e}")))?;
    let user_name = extract_local_part(&claims.username).to_string();

    let row_log = serde_json::json!({
        "cognito_id": claims.sub,
        "username": claims.username,
        "body_length": body.body.len(),
    })
    .to_string();

    let new_message = NewMessage {
        body: body.body,
        user_name,
        cognito_id,
        row_log,
        is_from_user: true,
    };

    let use_case = PostMessageUseCase::new(state.repository);
    use_case.execute(new_message).await?;

    Ok((
        StatusCode::CREATED,
        Json(PostMessageResponse {
            status: "success".to_string(),
            message: "Message created successfully".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::apigw::{
        ApiGatewayRequestAuthorizer, ApiGatewayRequestAuthorizerJwtDescription,
        ApiGatewayV2httpRequestContext, ApiGatewayV2httpRequestContextHttpDescription,
    };
    use std::collections::HashMap;

    fn make_jwt_request_context(sub: &str, username: &str) -> RequestContext {
        let mut claims = HashMap::new();
        claims.insert("sub".to_string(), sub.to_string());
        claims.insert("cognito:username".to_string(), username.to_string());

        let mut jwt = ApiGatewayRequestAuthorizerJwtDescription::default();
        jwt.claims = claims;

        let mut authorizer = ApiGatewayRequestAuthorizer::default();
        authorizer.jwt = Some(jwt);

        let mut http = ApiGatewayV2httpRequestContextHttpDescription::default();
        http.method = http::Method::POST;
        http.path = Some("/api/v1/messages".to_string());

        let mut ctx = ApiGatewayV2httpRequestContext::default();
        ctx.authorizer = Some(authorizer);
        ctx.http = http;

        RequestContext::ApiGatewayV2(ctx)
    }

    #[test]
    fn jwtクレームを正常に取得できる() {
        let ctx = make_jwt_request_context(
            "12345678-1234-1234-1234-123456789012",
            "testuser@example.com",
        );
        let claims = extract_jwt_claims(&ctx).unwrap();
        assert_eq!(claims.sub, "12345678-1234-1234-1234-123456789012");
        assert_eq!(claims.username, "testuser@example.com");
    }

    #[test]
    fn emailのlocal_partを抽出できる() {
        assert_eq!(extract_local_part("user@example.com"), "user");
        assert_eq!(extract_local_part("testuser@company.co.jp"), "testuser");
        // @がない場合はそのまま返す
        assert_eq!(extract_local_part("noatsign"), "noatsign");
    }
}
