//! Lambda HTTPハンドラー。
//!
//! API Gatewayからのリクエストを受け取り、適切なユースケースを呼び出してレスポンスを返す。
//! JWTの検証はAPI GatewayのJWTオーソライザーが行い、Lambdaではクレームから`sub`を取得する。

use lambda_http::RequestExt;
use lambda_http::{Body, Error, Request, Response, http::StatusCode, request::RequestContext};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::use_cases::{get_messages::GetMessages, post_message::PostMessage};
use crate::domain::message::{CognitoId, MessageBody};
use crate::error::HandlerError;
use crate::infrastructure::db::{connection::connect, message_repository::MessageRepositoryImpl};

/// チャット投稿APIのリクエストボディ。
#[derive(Debug, Deserialize, ToSchema)]
pub struct PostChatRequest {
    /// 投稿するメッセージ本文
    pub body: String,
}

/// チャット取得APIのレスポンスボディ（メッセージ1件）。
#[derive(Debug, Serialize, ToSchema)]
pub struct ChatMessage {
    /// ユーザーからのメッセージかどうか（true: ユーザー, false: システム）
    pub is_from_user: bool,
    /// メッセージ本文
    pub body: String,
    /// 作成日時（ISO 8601形式）
    pub created_at: String,
}

/// エラーレスポンスボディ。
#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

/// リクエストコンテキストからCognitoの`sub`クレームを取得する。
///
/// API GatewayのJWTオーソライザーによって認証済みのリクエストには、
/// JWTクレームがリクエストコンテキストに含まれる。
fn extract_cognito_id(request: &Request) -> Result<CognitoId, HandlerError> {
    let sub = match request.request_context() {
        RequestContext::ApiGatewayV2(ctx) => ctx
            .authorizer
            .and_then(|a| a.jwt)
            .and_then(|j| j.claims.get("sub").cloned()),
        _ => None,
    };

    let sub = sub.ok_or(HandlerError::Unauthorized)?;
    sub.parse::<CognitoId>()
        .map_err(|_| HandlerError::Unauthorized)
}

/// JSONレスポンスを生成するヘルパー関数。
fn json_response<T: Serialize>(status: StatusCode, body: &T) -> Result<Response<Body>, Error> {
    let json = serde_json::to_string(body)?;
    let response = Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::Text(json))?;
    Ok(response)
}

/// エラーレスポンスを生成するヘルパー関数。
fn error_response(status: StatusCode, message: &str) -> Result<Response<Body>, Error> {
    json_response(
        status,
        &ErrorResponse {
            message: message.to_string(),
        },
    )
}

/// チャット取得ハンドラー（GET /chat）。
///
/// 認証済みユーザーのチャットメッセージ一覧を返す。
pub async fn handle_get_chat(request: Request) -> Result<Response<Body>, Error> {
    let cognito_id = match extract_cognito_id(&request) {
        Ok(id) => id,
        Err(HandlerError::Unauthorized) => {
            return error_response(StatusCode::UNAUTHORIZED, "認証が必要です");
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("内部エラー: {e}"),
            );
        }
    };

    let db = match connect().await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("データベース接続エラー: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "データベース接続に失敗しました",
            );
        }
    };

    let repository = MessageRepositoryImpl::new(db);
    let use_case = GetMessages::new(repository);

    match use_case.execute(cognito_id).await {
        Ok(messages) => {
            let chat_messages: Vec<ChatMessage> = messages
                .into_iter()
                .map(|m| ChatMessage {
                    is_from_user: m.is_from_user,
                    body: m.body,
                    created_at: m.created_at.to_rfc3339(),
                })
                .collect();
            json_response(StatusCode::OK, &chat_messages)
        }
        Err(e) => {
            tracing::error!("チャット取得エラー: {e}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "メッセージの取得に失敗しました",
            )
        }
    }
}

/// チャット投稿ハンドラー（POST /chat）。
///
/// 認証済みユーザーのチャットメッセージを投稿する。
pub async fn handle_post_chat(request: Request) -> Result<Response<Body>, Error> {
    let cognito_id = match extract_cognito_id(&request) {
        Ok(id) => id,
        Err(HandlerError::Unauthorized) => {
            return error_response(StatusCode::UNAUTHORIZED, "認証が必要です");
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("内部エラー: {e}"),
            );
        }
    };

    let body = match request.body() {
        Body::Text(s) => s.clone(),
        Body::Binary(b) => String::from_utf8_lossy(b).into_owned(),
        Body::Empty => String::new(),
        _ => String::new(),
    };

    let post_request: PostChatRequest = match serde_json::from_str(&body) {
        Ok(r) => r,
        Err(_) => {
            return error_response(StatusCode::BAD_REQUEST, "リクエストボディのJSONが不正です");
        }
    };

    let message_body = match MessageBody::new(post_request.body) {
        Ok(b) => b,
        Err(_) => {
            return error_response(StatusCode::BAD_REQUEST, "メッセージ本文が空です");
        }
    };

    let db = match connect().await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("データベース接続エラー: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "データベース接続に失敗しました",
            );
        }
    };

    let repository = MessageRepositoryImpl::new(db);
    let use_case = PostMessage::new(repository);

    match use_case.execute(cognito_id, message_body).await {
        Ok(()) => json_response(
            StatusCode::CREATED,
            &serde_json::json!({"message": "投稿しました"}),
        ),
        Err(e) => {
            tracing::error!("チャット投稿エラー: {e}");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "メッセージの投稿に失敗しました",
            )
        }
    }
}
