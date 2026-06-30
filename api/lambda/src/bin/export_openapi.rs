use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("apiディレクトリの取得に失敗しました。")?
        .join("openapi.yaml");

    fs::write(&output_path, sns_system_api_lambda::openapi_yaml())?;
    println!("OpenAPI定義を出力しました: {}", output_path.display());

    Ok(())
}
