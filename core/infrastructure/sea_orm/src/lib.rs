//! インフラ層の公開API。
//!
//! Domainで定義したRepository traitを、SeaORMとDBスキーマに接続して実装する。

pub mod sea_orm_message_repository;

pub use sea_orm_message_repository::SeaOrmMessageRepository;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 公開re_exportが参照できる() {
        let _repo_ctor: fn(sea_orm::prelude::DatabaseConnection) -> SeaOrmMessageRepository =
            SeaOrmMessageRepository::new;
    }
}
