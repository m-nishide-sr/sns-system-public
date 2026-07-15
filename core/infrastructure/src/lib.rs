//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod db_dsql;
pub mod db_postgres;
pub mod sea_orm_message_repository;

pub use db_dsql::create_db_dsql;
pub use db_postgres::create_db_postgres;
pub use sea_orm_message_repository::SeaOrmMessageRepository;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 公開re_exportが参照できる() {
        let _create_dsql = create_db_dsql;
        let _create_postgres = create_db_postgres;
        let _repo_ctor = SeaOrmMessageRepository::new;
    }
}
