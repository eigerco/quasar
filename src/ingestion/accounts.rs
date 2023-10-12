use log::info;
use quasar_entities::account;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use stellar_node_entities::{accounts, prelude::Accounts};

use super::IngestionError;

pub async fn ingest_accounts(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    ledger_sequence: i32,
) -> Result<(), IngestionError> {
    // Query all accounts with lastmodified = ledger_sequence
    let updated_accounts = Accounts::find()
        .filter(stellar_node_entities::accounts::Column::Lastmodified.eq(ledger_sequence))
        .all(node_database)
        .await?;

    let count = updated_accounts.len();

    // Ingest all updated accounts
    for account in updated_accounts {
        ingest_account(quasar_database, account).await?;
    }

    info!("Ingested {} updated accounts", count);

    Ok(())
}

pub async fn ingest_account(
    db: &DatabaseConnection,
    account: accounts::Model,
) -> Result<(), IngestionError> {
    let account: account::ActiveModel = account::ActiveModel::try_from(account)?;

    account.insert(db).await?;

    Ok(())
}
