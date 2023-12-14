use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContractSpec::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContractSpec::Address)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ContractSpec::Address).string().not_null())
                    .col(ColumnDef::new(ContractSpec::Spec).json().not_null())
                    .col(
                        ColumnDef::new(ContractSpec::LastModified)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ContractSpec::CreatedAt)
                            .timestamp_with_time_zone()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContractSpec::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContractSpec {
    #[sea_orm(iden = "contract_spec")]
    Table,
    Address,
    Spec,
    LastModified,
    CreatedAt,
}
