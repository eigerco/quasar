pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_ledgers;
mod m20231012_071003_create_accounts;
mod m20231018_125257_create_events_table;
mod m20231013_065827_create_contract_table;
mod m20231016_094252_create_transactions;
mod m20231016_094302_create_operations;
mod m20231026_092149_created_at_cursor_on_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_ledgers::Migration),
            Box::new(m20231012_071003_create_accounts::Migration),
            Box::new(m20231018_125257_create_events_table::Migration),
            Box::new(m20231013_065827_create_contract_table::Migration),
            Box::new(m20231016_094252_create_transactions::Migration),
            Box::new(m20231016_094302_create_operations::Migration),
            Box::new(m20231026_092149_created_at_cursor_on_tables::Migration),
        ]
    }
}
