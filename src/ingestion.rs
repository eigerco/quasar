use log::{debug, error};
use prometheus::{IntCounter, Registry};
use quasar_entities::{ledger, prelude::Ledger};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder};
use stellar_node_entities::ledgerheaders;
use thiserror::Error;

use crate::{configuration::Ingestion, ingestion::ledgers::ingest_ledgers};

mod ledgers;
mod contracts;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
    #[error("Missing ledger sequence")]
    MissingLedgerSequence,
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] stellar_xdr::Error),
}

enum IngestionNeeded {
    Yes {
        last_ingested_ledger_sequence: Option<i32>,
    },
    No,
}

async fn new_ledgers_available(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
) -> Result<IngestionNeeded, DbErr> {
    let last_ingested_ledger_sequence = last_ingested_ledger_sequence(quasar_database).await?;
    let last_stellar_ledger_sequence = last_stellar_ledger_sequence(node_database).await?;

    let ingestion_needed = if last_ingested_ledger_sequence != last_stellar_ledger_sequence {
        IngestionNeeded::Yes {
            last_ingested_ledger_sequence,
        }
    } else {
        IngestionNeeded::No
    };

    Ok(ingestion_needed)
}

async fn last_ingested_ledger_sequence(
    quasar_database: &DatabaseConnection,
) -> Result<Option<i32>, DbErr> {
    let last_ingested_ledger = Ledger::find()
        .order_by_desc(ledger::Column::Sequence)
        .one(quasar_database)
        .await?;
    let last_ingested_ledger_sequence = last_ingested_ledger.map(|ledger| ledger.sequence);
    Ok(last_ingested_ledger_sequence)
}

async fn last_stellar_ledger_sequence(
    node_database: &DatabaseConnection,
) -> Result<Option<i32>, DbErr> {
    let last_stellar_ledger = ledgerheaders::Entity::find()
        .order_by_desc(ledgerheaders::Column::Ledgerseq)
        .one(node_database)
        .await?;
    let last_stellar_ledger_sequence = last_stellar_ledger.and_then(|ledger| ledger.ledgerseq);
    Ok(last_stellar_ledger_sequence)
}

fn setup_metrics(
    metrics: Registry,
) -> prometheus::core::GenericCounter<prometheus::core::AtomicU64> {
    let ingested_ledgers_counter =
        IntCounter::new("ingested_ledgers", "Number of ingested ledgers").unwrap();
    metrics
        .register(Box::new(ingested_ledgers_counter.clone()))
        .expect("Failed to register counter");
    ingested_ledgers_counter
}

pub async fn ingest(
    node_database: DatabaseConnection,
    quasar_database: DatabaseConnection,
    ingestion: &Ingestion,
    metrics: Registry,
) {
    let ingested_ledgers = setup_metrics(metrics);

    loop {
        sleep(ingestion).await;

        let ingestion_needed = new_ledgers_available(&node_database, &quasar_database).await;

        match ingestion_needed {
            Ok(IngestionNeeded::Yes {
                last_ingested_ledger_sequence,
            }) => {
                debug!("New ledgers available");
                let ingestion_result = ingest_ledgers(
                    &node_database,
                    &quasar_database,
                    last_ingested_ledger_sequence,
                    &ingested_ledgers,
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

async fn sleep(ingestion: &Ingestion) {
    tokio::time::sleep(tokio::time::Duration::from_secs(ingestion.polling_interval)).await;
}
