use crate::configuration::Ingestion;
use prometheus::Registry;
use sea_orm::{DatabaseConnection, DbErr};
mod contracts;
mod ledgers;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Database error: {0}")]
    DbError(#[from] DbErr),
    #[error("Missing ledger sequence")]
    MissingLedgerSequence,
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] stellar_xdr::curr::Error),
}

pub async fn ingest(
    node_database: DatabaseConnection,
    quasar_database: DatabaseConnection,
    ingestion: Ingestion,
    metrics: Registry,
) {
    let ledgers = ledgers::ingest(&node_database, &quasar_database, &ingestion, &metrics);
    let contracts = contracts::ingest(&node_database, &quasar_database, &ingestion, &metrics);

    tokio::join!(ledgers, contracts);
}

pub async fn sleep(ingestion: &Ingestion) {
    tokio::time::sleep(tokio::time::Duration::from_secs(ingestion.polling_interval)).await;
}
