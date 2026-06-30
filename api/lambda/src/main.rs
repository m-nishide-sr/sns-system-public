use lambda_http::{Error, run, service_fn, tracing};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(sns_system_api_lambda::handle_request)).await
}
