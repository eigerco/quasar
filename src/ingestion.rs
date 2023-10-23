use log::{debug, error};
use prometheus::{IntCounter, Registry};
use quasar_entities::account::AccountError;
use sea_orm::DbErr;
use thiserror::Error;

use crate::{
    configuration::Ingestion,
    databases::{NodeDatabase, QuasarDatabase},
    ingestion::ledgers::{ingest_ledgers, new_ledgers_available, IngestionNeeded},
    metrics::IngestionMetrics,
};

mod accounts;
mod contracts;
mod ledgers;
mod operations;
mod transactions;
mod events;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
    #[error("Missing ledger sequence")]
    MissingLedgerSequence,
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] stellar_xdr::Error),
    #[error("Account error: {0}")]
    AccountError(#[from] AccountError),
}

pub(super) async fn ingest(
    node_database: NodeDatabase,
    quasar_database: QuasarDatabase,
    ingestion: Ingestion,
    metrics: Registry,
) {
    let ingestion_metrics = setup_ingestion_metrics(&metrics);

    loop {
        sleep(&ingestion).await;

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
                    &ingestion_metrics,
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

pub async fn sleep(ingestion: &Ingestion) {
    tokio::time::sleep(tokio::time::Duration::from_secs(ingestion.polling_interval)).await;
}

fn setup_ingestion_metrics(metrics: &Registry) -> IngestionMetrics {
    let ingested_ledgers =
        IntCounter::new("ingested_ledgers", "Number of ingested ledgers").unwrap();
    metrics
        .register(Box::new(ingested_ledgers.clone()))
        .expect("Failed to register counter");

    let ingested_contracts =
        IntCounter::new("ingested_contracts", "Number of ingested contracts").unwrap();
    metrics
        .register(Box::new(ingested_contracts.clone()))
        .expect("Failed to register counter");

    IngestionMetrics {
        ingested_ledgers,
        ingested_contracts,
    }
}
