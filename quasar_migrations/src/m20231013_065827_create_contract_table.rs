use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contract::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contract::Address)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Contract::Hash)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Contract::Key).string().not_null())
                    .col(ColumnDef::new(Contract::Type).string().not_null())
                    .col(ColumnDef::new(Contract::LastModified).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contract::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Contract {
    #[sea_orm(iden = "contracts")]
    Table,
    Address,
    Hash,
    Key,
    Type,
    LastModified,
    // Methods
}
