//! OpenAPI仕様書（`openapi.yaml`）を生成するバイナリ。
//!
//! `cargo run --bin export_openapi > api/openapi.yaml` で実行する。

use utoipa::OpenApi;
use utoipa::openapi::Components;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

use lambda::interface::handler::{ChatMessage, PostChatRequest};

/// OpenAPI仕様書の定義。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SNSシステム チャットAPI",
        description = "SNSシステムのチャット機能を提供するAPIです。",
        version = "1.0.0",
        contact(name = "SNSシステム開発チーム"),
        license(name = "MIT")
    ),
    paths(get_chat, post_chat),
    components(schemas(ChatMessage, PostChatRequest)),
    security(("bearer_auth" = []))
)]
struct ApiDoc;

/// チャット取得API。
///
/// 認証済みユーザーのチャットメッセージ一覧を取得する。
#[utoipa::path(
    get,
    path = "/chat",
    responses(
        (status = 200, description = "チャットメッセージ一覧", body = Vec<ChatMessage>),
        (status = 401, description = "認証が必要です"),
        (status = 500, description = "内部サーバーエラー"),
    ),
    security(("bearer_auth" = []))
)]
#[allow(dead_code)]
async fn get_chat() {}

/// チャット投稿API。
///
/// 認証済みユーザーのチャットメッセージを投稿する。
#[utoipa::path(
    post,
    path = "/chat",
    request_body = PostChatRequest,
    responses(
        (status = 201, description = "投稿成功"),
        (status = 400, description = "リクエストが不正です"),
        (status = 401, description = "認証が必要です"),
        (status = 500, description = "内部サーバーエラー"),
    ),
    security(("bearer_auth" = []))
)]
#[allow(dead_code)]
async fn post_chat() {}

fn main() {
    let mut openapi = ApiDoc::openapi();

    // Bearer認証（Cognito JWT）のセキュリティスキームを追加
    let security_scheme = SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .description(Some(
                "Amazon CognitoのJWTトークンをBearerトークンとして指定してください。",
            ))
            .build(),
    );

    if let Some(components) = openapi.components.as_mut() {
        components
            .security_schemes
            .insert("bearer_auth".to_string(), security_scheme);
    } else {
        let mut c = Components::default();
        c.security_schemes
            .insert("bearer_auth".to_string(), security_scheme);
        openapi.components = Some(c);
    }

    let yaml = openapi
        .to_yaml()
        .expect("OpenAPI仕様書のYAML変換に失敗しました");
    print!("{yaml}");
}
