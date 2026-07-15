use lambda_runtime::Error;
use pre_sign_up::run_lambda;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    run_lambda().await
}
