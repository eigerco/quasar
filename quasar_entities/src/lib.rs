use sea_orm::DatabaseConnection;

pub mod prelude;

pub mod account;
pub mod contract;
pub mod event;
pub mod ledger;
pub mod operation;
pub mod transaction;

#[derive(Clone, Debug)]
pub struct QuasarDataLoader {
    pool: DatabaseConnection,
}

impl QuasarDataLoader {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}
