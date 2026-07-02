use std::{fs, path::Path};

const OPENAPI_YAML: &str = r#"openapi: 3.0.3
info:
  title: SNS API
  version: 1.0.0
paths:
  /api/v1/timeline:
    get:
      summary: タイムライン取得
      security:
        - bearerAuth: []
      parameters:
        - in: query
          name: before
          required: false
          schema:
            type: string
            format: date-time
      responses:
        '200':
          description: OK
  /api/v1/messages:
    post:
      summary: メッセージ投稿
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [body]
              properties:
                body:
                  type: string
      responses:
        '201':
          description: Created
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
"#;

fn main() {
    let api_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("api/lambdaの親ディレクトリが存在するはずです");
    let openapi_path = api_root.join("openapi.yaml");

    fs::write(&openapi_path, OPENAPI_YAML).expect("openapi.yamlの出力に失敗しました");
    println!("{}", openapi_path.display());
}
