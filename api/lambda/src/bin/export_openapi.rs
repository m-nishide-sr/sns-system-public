//! `utoipa` から OpenAPI YAML を生成し、リポジトリ管理用の `openapi.yaml` を更新するツール。

use std::{error::Error, fs, path::Path};

use sns_system_api_lambda::ApiDoc;
use utoipa::OpenApi;

/// OpenAPI 定義を YAML へ変換し、`api/openapi.yaml` へ保存する。
fn main() -> Result<(), Box<dyn Error>> {
    let openapi = ApiDoc::openapi().to_yaml()?;
    let output_path = Path::new("..")
        .join("openapi.yaml")
        .canonicalize()
        .unwrap_or_else(|_| Path::new("../openapi.yaml").to_path_buf());

    fs::write(output_path, openapi)?;
    Ok(())
}
