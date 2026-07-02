use sns_api_lambda::http::ApiDoc;
use std::{fs, path::Path};
use utoipa::OpenApi;

fn main() {
    let api_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("api/lambdaの親ディレクトリが存在するはずです");
    let openapi_path = api_root.join("openapi.yaml");

    let yaml = ApiDoc::openapi()
        .to_yaml()
        .expect("OpenAPIのYAML変換に失敗しました");

    fs::write(&openapi_path, yaml).expect("openapi.yamlの出力に失敗しました");
    println!("{}", openapi_path.display());
}
