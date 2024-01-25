use log::{debug, error};
use prometheus::{IntCounter, Registry};
use quasar_entities::{account::AccountError, event::EventError};
use sea_orm::DbErr;
use thiserror::Error;

use crate::{
    configuration::Ingestion,
    // databases::{NodeDatabase, QuasarDatabase},
    // ingestion::ledgers::{ingest_ledgers, new_ledgers_available, IngestionNeeded},
};

mod accounts;
mod contracts;
mod events;
mod ledgers;
mod operations;
mod transactions;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
    #[error("Missing ledger sequence")]
    MissingLedgerSequence,
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] stellar_xdr::curr::Error),
    #[error("Account error: {0}")]
    AccountError(#[from] AccountError),
    #[error("Event error: {0}")]
    EventError(#[from] EventError),
}
pub(super) struct IngestionMetrics {
    pub ledgers: IntCounter,
    pub accounts: IntCounter,
    pub contracts: IntCounter,
    pub transactions: IntCounter,
    pub operations: IntCounter,
    pub events: IntCounter,
}

// pub(super) async fn ingest(
//     node_database: NodeDatabase,
//     quasar_database: QuasarDatabase,
//     ingestion: Ingestion,
//     metrics: Registry,
// ) {
//     let ingestion_metrics = setup_ingestion_metrics(&metrics);

//     loop {
//         sleep(&ingestion).await;

//         let ingestion_needed = new_ledgers_available(&node_database, &quasar_database).await;

//         match ingestion_needed {
//             Ok(IngestionNeeded::Yes {
//                 last_ingested_ledger_sequence,
//             }) => {
//                 debug!("New ledgers available");
//                 let ingestion_result = ingest_ledgers(
//                     &node_database,
//                     &quasar_database,
//                     last_ingested_ledger_sequence,
//                     &ingestion_metrics,
//                 )
//                 .await;

//                 if let Err(error) = ingestion_result {
//                     error!("Error while ingesting ledgers: {:?}", error);
//                 }
//             }
//             Ok(IngestionNeeded::No) => {}
//             Err(error) => {
//                 error!("Error while checking for new ledgers: {}", error);
//             }
//         }
//     }
// }

pub async fn sleep(ingestion: &Ingestion) {
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}

fn setup_ingestion_metrics(metrics: &Registry) -> IngestionMetrics {
    let ledgers = create_ingestion_counter(metrics, "ledgers");
    let contracts = create_ingestion_counter(metrics, "contracts");
    let accounts = create_ingestion_counter(metrics, "accounts");
    let transactions = create_ingestion_counter(metrics, "transactions");
    let operations = create_ingestion_counter(metrics, "operations");
    let events = create_ingestion_counter(metrics, "events");

    IngestionMetrics {
        ledgers,
        contracts,
        accounts,
        transactions,
        operations,
        events,
    }
}

fn create_ingestion_counter(
    metrics: &Registry,
    name: &str,
) -> prometheus::core::GenericCounter<prometheus::core::AtomicU64> {
    let counter = IntCounter::new(
        format!("ingested_{}", name),
        format!("Number of ingested {}", name),
    )
    .unwrap();
    metrics
        .register(Box::new(counter.clone()))
        .expect("Failed to register counter");
    counter
}
