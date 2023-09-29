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
                        ColumnDef::new(Ledger::Hash)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Ledger::PreviousLedgerHash)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Ledger::ProtocolVersion)
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Ledger::Sequence).integer().not_null())
                    .col(ColumnDef::new(Ledger::TotalCoins).big_integer().not_null())
                    .col(ColumnDef::new(Ledger::FeePool).big_integer().not_null())
                    .col(
                        ColumnDef::new(Ledger::InflationSeq)
                            .integer()
                            .unsigned()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Ledger::IdPool).big_unsigned().not_null())
                    .col(ColumnDef::new(Ledger::BaseFee).unsigned().not_null())
                    .col(ColumnDef::new(Ledger::BaseReserve).unsigned().not_null())
                    .col(ColumnDef::new(Ledger::MaxTxSetSize).unsigned().not_null())
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
    #[sea_orm(iden = "ledgers")]
    Table,
    Hash,
    PreviousLedgerHash,
    ProtocolVersion,
    Sequence,
    TotalCoins,
    FeePool,
    InflationSeq,
    IdPool,
    BaseFee,
    BaseReserve,
    MaxTxSetSize,
}
