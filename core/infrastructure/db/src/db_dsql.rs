//! # データベース接続モジュール
//!
//! このモジュールは、Amazon Aurora DSQL への接続管理を担当します。
//! [`aurora_dsql_sqlx_connector`] クレートを使用して IAM 認証付きの接続プールを構築し、
//! [`sea_orm`] の [`PgPool`] として提供します。
//!
//! ## Aurora DSQL について
//!
//! Amazon Aurora DSQL は、IAM ロールベースの認証を使用するサーバーレス分散 SQL データベースです。
//! 通常の PostgreSQL と異なり、ユーザー名はロール名、パスワードは IAM 認証トークンで自動生成されます。
//! このモジュールでは [`aurora_dsql_sqlx_connector`] がトークン生成と更新を自動的に処理します。
//!
//! ## 接続文字列の形式
//!
//! ```text
//! postgres://<role>@<endpoint>/postgres?region=<region>
//! ```
//!
//! - `<role>`: データベースロール名（例: `lambda`, `selectview`）
//! - `<endpoint>`: Aurora DSQL クラスターのエンドポイント（例: `abc123.dsql.ap-northeast-1.on.aws`）
//! - `<region>`: AWSリージョン（例: `ap-northeast-3`）
//!
//! ## 使用するロール
//!
//! Lambda 関数では以下のロールを使用します：
//! - `lambda`: `messages` テーブルの SELECT / INSERT に使用

use anyhow::Error;
use aurora_dsql_sqlx_connector::pool;
use sea_orm::sqlx::PgPool;

/// Aurora DSQL への接続文字列を構築する
///
/// 指定されたロール名、エンドポイント、リージョンから PostgreSQL 形式の接続文字列を生成します。
/// [`aurora_dsql_sqlx_connector`] がこの文字列を解析して IAM 認証トークンを取得し、
/// 実際のデータベース接続を確立します。
///
/// # Arguments
///
/// * `role` - データベースロール名（例: `"lambda"`, `"selectview"`)
/// * `endpoint` - Aurora DSQL クラスターのエンドポイントホスト名
///   （例: `"abc123.dsql.ap-northeast-1.on.aws"`）
/// * `region` - Aurora DSQL クラスターが存在する AWS リージョン
///   （例: `"ap-northeast-3"`）
///
/// # Returns
///
/// PostgreSQL URI 形式の接続文字列。
/// `aurora_dsql_sqlx_connector` に渡すことで、IAM 認証が自動的に処理されます。
fn build_connection_string(role: &str, endpoint: &str, region: &str) -> String {
    format!("postgres://{role}@{endpoint}/postgres?region={region}")
}

/// # AuroraDSQLConnectionInfo
///
/// * `role` - Aurora DSQL のデータベースロール名（例: `"lambda"`)
/// * `endpoint` - Aurora DSQL クラスターのエンドポイントホスト名
/// * `region` - Aurora DSQL クラスターが存在する AWS リージョン
pub struct AuroraDSQLConnectionInfo<T>
where
    T: AsRef<str>,
{
    /// データベースロール名（例: `"lambda"`, `"selectview"`）
    pub role: T,
    /// Aurora DSQL クラスターのエンドポイントホスト名
    pub endpoint: T,
    /// Aurora DSQL クラスターが存在する AWS リージョン
    pub region: T,
}

/// Aurora DSQL への SeaORM データベース接続を作成する
///
/// 指定されたロール・エンドポイント・リージョンを使用して Aurora DSQL への接続プールを構築し、
/// [`sea_orm::sqlx::PgPool`] として返します。
///
/// 接続確立には IAM 認証が使用されます。[`aurora_dsql_sqlx_connector`] が AWS STS と通信して
/// 認証トークンを自動取得します。そのため、Lambda 関数の実行ロールに
/// `dsql:DbConnectAdmin` または `dsql:DbConnect` 権限が必要です。
///
/// # Arguments
///
/// * `role` - Aurora DSQL のデータベースロール名（例: `"lambda"`)
/// * `endpoint` - Aurora DSQL クラスターのエンドポイントホスト名
/// * `region` - Aurora DSQL クラスターが存在する AWS リージョン
///
/// # Returns
///
/// * `Ok(PgPool)` - 正常に接続が確立された場合
/// * `Err(Error)` - 接続に失敗した場合（IAM権限不足、ネットワークエラー等）
///
/// # Errors
///
/// - `aurora_dsql_sqlx_connector::pool::connect` が失敗した場合（IAM認証エラー、接続拒否等）
///   `"Failed to connect to database: ..."` メッセージを含む [`anyhow::Error`] を返します。
pub async fn create_db_dsql<T>(role: &AuroraDSQLConnectionInfo<T>) -> Result<PgPool, Error>
where
    T: AsRef<str>,
{
    tracing::info!("Creating database connection with Aurora DSQL SQLx connector...");
    let connection_string = build_connection_string(
        role.role.as_ref(),
        role.endpoint.as_ref(),
        role.region.as_ref(),
    );
    let pool = pool::connect(&connection_string)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 接続文字列を正しく組み立てる() {
        let actual = build_connection_string(
            "lambda",
            "example.dsql.ap-northeast-3.on.aws",
            "ap-northeast-3",
        );
        assert_eq!(
            actual,
            "postgres://lambda@example.dsql.ap-northeast-3.on.aws/postgres?region=ap-northeast-3"
                .to_string()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "ローカルのDBが必要なためデフォルトでは実行しない"]
    async fn エラー時に接続できないこと() {
        let connection_info = AuroraDSQLConnectionInfo {
            role: "invalid_role",
            endpoint: "invalid_endpoint",
            region: "invalid_region",
        };
        assert!(create_db_dsql(&connection_info).await.is_err());
    }
}
