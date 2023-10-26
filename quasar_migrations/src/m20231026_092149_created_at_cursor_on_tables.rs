use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use CreatedAtIden::*;
        let tables = [
            Accounts,
            Contracts,
            Transactions,
            Operations,
            Events,
            Ledgers,
        ];
        for table in tables {
            manager
                .alter_table(
                    Table::alter()
                        .table(table)
                        .add_column(
                            ColumnDef::new(CreatedAtIden::CreatedAt)
                                .timestamp_with_time_zone()
                                .default(Expr::current_timestamp())
                                .not_null(),
                        )
                        .take(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use CreatedAtIden::*;
        let tables = [
            Accounts,
            Contracts,
            Transactions,
            Operations,
            Events,
            Ledgers,
        ];
        for table in tables {
            manager
                .alter_table(
                    Table::alter()
                        .table(table)
                        .drop_column(CreatedAtIden::CreatedAt)
                        .take(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum CreatedAtIden {
    CreatedAt,
    Accounts,
    Contracts,
    Transactions,
    Operations,
    Events,
    Ledgers,
}
