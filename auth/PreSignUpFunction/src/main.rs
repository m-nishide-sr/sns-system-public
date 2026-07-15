use lambda_runtime::{Error, run, service_fn};
use pre_sign_up::function_handler;
use std::env;

#[tokio::main(flavor = "current_thread")]
/// Lambda ランタイムへ型付きハンドラを登録するエントリーポイント。
async fn main() -> Result<(), Error> {
    let allowed_domains: String = env::var("ALLOWED_EMAIL_DOMAINS").map_err(|_| -> Error {
        // 環境変数が未設定の場合はデプロイ設定の問題であるため、明示的なエラーを返す
        String::from("ALLOWED_EMAIL_DOMAINS 環境変数が設定されていません").into()
    })?;

    run(service_fn(|event| async {
        function_handler(event, &allowed_domains)
    }))
    .await
}
