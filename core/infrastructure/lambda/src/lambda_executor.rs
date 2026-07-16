use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use std::future::Future;
// use std::pin::Pin;

// /// ハンドラ関数の型別名（PinとBoxを使った静的関数向け）
// pub type LambdaHandlerFn<T, U> =
//     fn(LambdaEvent<T>) -> Pin<Box<dyn Future<Output = Result<U, Error>> + Send + 'static>>;

/// Lambda 実行モジュール
///
/// このモジュールは、AWS Lambdaの実行環境での共通処理を提供します。
/// この処理に入った時点で、INITフェーズは終わり、INVOKEフェーズに入ります。
/// Lambdaのハンドラは、ここで登録された関数が呼び出されます。
pub async fn lambda_executor<T, U, F, Fut>(invoke: F) -> Result<(), Error>
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
    F: FnMut(LambdaEvent<T>) -> Fut,
    Fut: Future<Output = Result<U, Error>>,
{
    run(service_fn(invoke)).await
}
