//! # SNS APIサブシステム
//!
//! このライブラリはSNSシステムのAPIを実装するRustクレートです。
//!
//! ## アーキテクチャ
//!
//! クリーンアーキテクチャに基づき、以下のレイヤーで構成されます。
//!
//! - `domain` : ドメインエンティティ・値オブジェクト・ドメインエラー
//! - `application` : ユースケース・Repositoryトレイト
//! - `infrastructure` : DB等の外部依存の具体実装
//! - `interface` : HTTPハンドラ等の入出力境界

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;
