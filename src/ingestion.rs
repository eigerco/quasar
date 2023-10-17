use log::error;
use prometheus::Registry;
use quasar_entities::account::AccountError;
use sea_orm::{DatabaseConnection, DbErr};
use thiserror::Error;

use crate::configuration::Ingestion;

mod accounts;
mod contracts;
mod ledgers;

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

pub async fn ingest(
    node_database: DatabaseConnection,
    quasar_database: DatabaseConnection,
    ingestion: Ingestion,
    metrics: Registry,
) {
    let ledgers = ledgers::ingest(&node_database, &quasar_database, &ingestion, &metrics);
    let accounts = accounts::ingest(&node_database, &quasar_database, &ingestion);
    let contracts = contracts::ingest(&node_database, &quasar_database, &ingestion, &metrics);
    tokio::join!(ledgers, accounts, contracts);
}

pub async fn sleep(ingestion: &Ingestion) {
    tokio::time::sleep(tokio::time::Duration::from_secs(ingestion.polling_interval)).await;
}
