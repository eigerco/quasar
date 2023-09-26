use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ledger::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ledger::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ledger::Hash).string().not_null())
                    .col(
                        ColumnDef::new(Ledger::PreviousLedgerHash)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ledger::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Ledger {
    Table,
    Id,
    Hash,
    PreviousLedgerHash,
}
