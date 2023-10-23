use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Operation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Operation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Operation::TransactionId).string().not_null())
                    .col(
                        ColumnDef::new(Operation::ApplicationOrder)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Operation::Type).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Operation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Operation {
    #[sea_orm(iden = "operations")]
    Table,
    Id,
    TransactionId,
    ApplicationOrder,
    Type,
}
