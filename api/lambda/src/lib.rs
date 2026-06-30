use std::{cmp::Reverse, sync::Mutex};

use chrono::{DateTime, Utc};
use lambda_http::{
    Body, Error, Request, Response,
    http::{Method, StatusCode},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static STORE: Lazy<Mutex<Vec<Message>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// メッセージ作成APIのリクエスト。
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMessageRequest {
    pub cognito_id: Uuid,
    pub body: String,
    pub is_from_user: bool,
}

/// メッセージAPIのレスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: Uuid,
    pub cognito_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub body: String,
    pub is_from_user: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse<'a> {
    error: &'a str,
}

/// LambdaイベントをHTTP APIとして処理する。
pub async fn handle_request(request: Request) -> Result<Response<Body>, Error> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    match (method, path.as_str()) {
        (Method::POST, "/messages") => create_message(request),
        (Method::GET, p) if p.starts_with("/messages/") => list_messages(p),
        _ => json_response(
            StatusCode::NOT_FOUND,
            &ErrorResponse {
                error: "エンドポイントが存在しません。",
            },
        ),
    }
}

/// OpenAPI仕様(YAML)を返却する。
pub fn openapi_yaml() -> &'static str {
    r#"openapi: 3.0.3
info:
  title: SNSシステム API
  version: 0.1.0
paths:
  /messages:
    post:
      summary: メッセージを作成する
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateMessageRequest'
      responses:
        '201':
          description: 作成成功
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Message'
  /messages/{cognito_id}:
    get:
      summary: 利用者のメッセージ一覧を取得する
      parameters:
        - in: path
          name: cognito_id
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: 取得成功
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Message'
components:
  schemas:
    CreateMessageRequest:
      type: object
      required:
        - cognito_id
        - body
        - is_from_user
      properties:
        cognito_id:
          type: string
          format: uuid
        body:
          type: string
        is_from_user:
          type: boolean
    Message:
      type: object
      required:
        - message_id
        - cognito_id
        - created_at
        - body
        - is_from_user
      properties:
        message_id:
          type: string
          format: uuid
        cognito_id:
          type: string
          format: uuid
        created_at:
          type: string
          format: date-time
        body:
          type: string
        is_from_user:
          type: boolean
"#
}

fn create_message(request: Request) -> Result<Response<Body>, Error> {
    let payload: CreateMessageRequest = serde_json::from_slice(&body_bytes(request.body())?)
        .map_err(|error| format!("リクエストJSONのパースに失敗しました: {error}"))?;

    if payload.body.trim().is_empty() {
        return json_response(
            StatusCode::BAD_REQUEST,
            &ErrorResponse {
                error: "bodyは空文字を許可しません。",
            },
        );
    }

    let message = Message {
        message_id: Uuid::now_v7(),
        cognito_id: payload.cognito_id,
        created_at: Utc::now(),
        body: payload.body,
        is_from_user: payload.is_from_user,
    };

    STORE
        .lock()
        .map_err(|error| format!("メッセージ保存処理でロックに失敗しました: {error}"))?
        .push(message.clone());

    json_response(StatusCode::CREATED, &message)
}

fn list_messages(path: &str) -> Result<Response<Body>, Error> {
    let cognito_id = path
        .strip_prefix("/messages/")
        .ok_or_else(|| "cognito_idの取得に失敗しました。".to_string())?;

    let cognito_id = Uuid::parse_str(cognito_id)
        .map_err(|error| format!("cognito_idはUUID形式で指定してください: {error}"))?;

    let mut messages = STORE
        .lock()
        .map_err(|error| format!("メッセージ取得処理でロックに失敗しました: {error}"))?
        .iter()
        .filter(|message| message.cognito_id == cognito_id)
        .cloned()
        .collect::<Vec<_>>();

    messages.sort_by_key(|message| Reverse(message.created_at));
    json_response(StatusCode::OK, &messages)
}

fn body_bytes(body: &Body) -> Result<Vec<u8>, Error> {
    match body {
        Body::Empty => Ok(Vec::new()),
        Body::Text(text) => Ok(text.as_bytes().to_vec()),
        Body::Binary(binary) => Ok(binary.clone()),
    }
}

fn json_response<T: Serialize>(status: StatusCode, body: &T) -> Result<Response<Body>, Error> {
    let serialized = serde_json::to_string(body)?;
    let response = Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::Text(serialized))
        .map_err(|error| format!("レスポンス生成に失敗しました: {error}"))?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset_store() {
        if let Ok(mut guard) = STORE.lock() {
            guard.clear();
        }
    }

    #[tokio::test]
    async fn メッセージを作成して一覧取得できる() {
        reset_store();
        let cognito_id = Uuid::now_v7();
        let request = lambda_http::http::Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .body(Body::Text(
                serde_json::json!({
                    "cognito_id": cognito_id,
                    "body": "こんにちは",
                    "is_from_user": true
                })
                .to_string(),
            ))
            .expect("POSTリクエストを構築できること");

        let created = handle_request(request)
            .await
            .expect("メッセージ作成が成功すること");
        assert_eq!(created.status(), StatusCode::CREATED);

        let list_request = lambda_http::http::Request::builder()
            .method(Method::GET)
            .uri(format!("/messages/{cognito_id}"))
            .body(Body::Empty)
            .expect("GETリクエストを構築できること");

        let list_response = handle_request(list_request)
            .await
            .expect("メッセージ一覧取得が成功すること");
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = body_bytes(list_response.body()).expect("レスポンスボディを取得できること");
        let messages: Vec<Message> =
            serde_json::from_slice(&body).expect("レスポンスJSONをパースできること");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "こんにちは");
    }

    #[tokio::test]
    async fn 空文字メッセージは作成できない() {
        reset_store();
        let request = lambda_http::http::Request::builder()
            .method(Method::POST)
            .uri("/messages")
            .body(Body::Text(
                serde_json::json!({
                    "cognito_id": Uuid::now_v7(),
                    "body": "  ",
                    "is_from_user": true
                })
                .to_string(),
            ))
            .expect("POSTリクエストを構築できること");

        let response = handle_request(request)
            .await
            .expect("バリデーションエラーでもレスポンスは返ること");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
