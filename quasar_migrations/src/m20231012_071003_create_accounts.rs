use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Account::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Account::Balance).big_integer().not_null())
                    .col(ColumnDef::new(Account::BuyingLiabilities).big_integer())
                    .col(ColumnDef::new(Account::SellingLiabilities).big_integer())
                    .col(
                        ColumnDef::new(Account::SequenceNumber)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::NumberOfSubentries)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Account::InflationDestination).string())
                    .col(ColumnDef::new(Account::HomeDomain).string().not_null())
                    .col(
                        ColumnDef::new(Account::MasterWeight)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::ThresholdLow)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::ThresholdMedium)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Account::ThresholdHigh)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Account::LastModified).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Account {
    #[sea_orm(iden = "accounts")]
    Table,
    Id,
    Balance,
    BuyingLiabilities,
    SellingLiabilities,
    SequenceNumber,
    NumberOfSubentries,
    InflationDestination,
    HomeDomain,
    MasterWeight,
    ThresholdLow,
    ThresholdMedium,
    ThresholdHigh,
    LastModified,
}
