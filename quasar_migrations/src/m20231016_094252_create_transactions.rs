use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transaction::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Transaction::LedgerSequence)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::ApplicationOrder)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transaction::AccountId).string().not_null())
                    .col(
                        ColumnDef::new(Transaction::AccountSequence)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::OperationCount)
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transaction {
    #[sea_orm(iden = "transactions")]
    Table,
    Id,
    LedgerSequence,
    ApplicationOrder,
    AccountId,
    AccountSequence,
    OperationCount,
}
