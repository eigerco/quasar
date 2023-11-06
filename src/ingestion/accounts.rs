use crate::databases::{NodeDatabase, QuasarDatabase};

use super::{IngestionError, IngestionMetrics};
use log::info;
use migration::OnConflict;
use quasar_entities::account;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use stellar_node_entities::{accounts, prelude::Accounts};

pub(super) async fn ingest_accounts(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    ledger_sequence: i32,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    // Query all accounts with lastmodified = ledger_sequence
    let updated_accounts = Accounts::find()
        .filter(stellar_node_entities::accounts::Column::Lastmodified.eq(ledger_sequence))
        .all(node_database.as_inner())
        .await?;

    let count = updated_accounts.len();

    // Ingest all updated accounts
    for account in updated_accounts {
        ingest_account(quasar_database, account).await?;

        metrics.accounts.inc();
    }

    info!("Ingested {} updated accounts", count);

    Ok(())
}

pub(super) async fn ingest_account(
    db: &QuasarDatabase,
    account: accounts::Model,
) -> Result<(), IngestionError> {
    let account: account::ActiveModel = account::ActiveModel::try_from(account)?;

    account::Entity::insert(account)
        .on_conflict(
            OnConflict::column(account::Column::Id)
                .update_columns([
                    account::Column::LastModified,
                    account::Column::Balance,
                    account::Column::BuyingLiabilities,
                    account::Column::HomeDomain,
                    account::Column::InflationDestination,
                    account::Column::MasterWeight,
                    account::Column::NumberOfSubentries,
                    account::Column::SellingLiabilities,
                    account::Column::SequenceNumber,
                ])
                .to_owned(),
        )
        .exec(db.as_inner())
        .await?;
    Ok(())
}
