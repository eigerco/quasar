use log::{info, debug, error};
use quasar_entities::account;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use stellar_node_entities::{accounts, prelude::Accounts};

use crate::configuration::Ingestion;

use super::{ledgers::{new_ledgers_available, IngestionNeeded}, IngestionError, sleep};

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

pub async fn ingest(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    ingestion: &Ingestion,
) {
    loop {
        sleep(ingestion).await;

        // Handle ledgers
        let ingestion_needed = new_ledgers_available(node_database, quasar_database).await;

        match ingestion_needed {
            Ok(IngestionNeeded::Yes {
                last_ingested_ledger_sequence,
            }) => {
                debug!("New ledgers available");
                let ingestion_result = ingest_accounts(
                    node_database,
                    quasar_database,
                    last_ingested_ledger_sequence.unwrap(),
                )
                .await;

                if let Err(error) = ingestion_result {
                    error!("Error while ingesting ledgers: {:?}", error);
                }
            }
            Ok(IngestionNeeded::No) => {}
            Err(error) => {
                error!("Error while checking for new ledgers: {}", error);
            }
        }
    }
}
