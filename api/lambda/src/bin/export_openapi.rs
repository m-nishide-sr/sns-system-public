//! OpenAPI定義ファイル生成バイナリ。
//!
//! このバイナリを実行すると、`utoipa` で生成したOpenAPI定義を
//! 標準出力にJSON形式で出力します。
//!
//! ## 使用方法
//!
//! ```bash
//! cargo run --bin export_openapi > ../openapi.yaml
//! ```

use utoipa::OpenApi;
use utoipa::openapi::ComponentsBuilder;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

use sns_api::interface::handler::{
    PostMessageRequest, PostMessageResponse, TimelineItem, TimelineQuery,
};

/// OpenAPIドキュメントの定義。
#[derive(OpenApi)]
#[openapi(
    info(
        title = "SNSシステム API",
        version = "0.1.0",
        description = "SNSシステムのタイムライン取得・メッセージ投稿APIです。"
    ),
    paths(
        sns_api::interface::handler::get_timeline,
        sns_api::interface::handler::post_message,
    ),
    components(schemas(TimelineItem, TimelineQuery, PostMessageRequest, PostMessageResponse,))
)]
struct ApiDoc;

fn main() {
    let mut openapi = ApiDoc::openapi();

    // JWT BearerトークンのセキュリティスキームをComponentsに追加する
    let security_scheme = SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
    );
    let components = openapi
        .components
        .get_or_insert_with(|| ComponentsBuilder::new().build());
    components
        .security_schemes
        .insert("jwt".to_string(), security_scheme);

    // JSON形式でOpenAPI定義を出力する（YAML形式として保存する場合も有効なYAML）
    let json = serde_json::to_string_pretty(&openapi)
        .expect("OpenAPI定義のJSONシリアライズに失敗しました");
    println!("{json}");
}
