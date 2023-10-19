use log::{debug, error};
use prometheus::Registry;
use quasar_entities::account::AccountError;
use sea_orm::{DatabaseConnection, DbErr};
use thiserror::Error;

use crate::{
    configuration::Ingestion,
    ingestion::{
        accounts::ingest_accounts,
        contracts::{ingest_contracts, new_contracts_available, IngestNextContract},
        ledgers::{ingest_ledgers, new_ledgers_available, IngestionNeeded},
    },
};

use self::{ledgers::setup_ledger_metrics, contracts::setup_contract_metrics};

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
    let ingested_ledgers = setup_ledger_metrics(&metrics);
    let ingested_contracts = setup_contract_metrics(&metrics);
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
                    &ingested_ledgers,
                )
                .await;

                if let Err(error) = ingestion_result {
                    error!("Error while ingesting ledgers: {:?}", error);
                }
                // Accounts
                {
                    let account_ingestion = ingest_accounts(
                        &node_database,
                        &quasar_database,
                        last_ingested_ledger_sequence.unwrap_or_default(),
                    )
                    .await;

                    if let Err(error) = account_ingestion {
                        error!("Error while ingesting accounts: {:?}", error);
                    }
                }

                // Contracts
                {
                    let ingestion_needed =
                        new_contracts_available(&node_database, &quasar_database).await;

                    match ingestion_needed {
                        Ok(IngestNextContract::Yes {
                            last_ingested_contract_sequence,
                        }) => {
                            debug!("New contracts available");
                            let ingestion_result = ingest_contracts(
                                &node_database,
                                &quasar_database,
                                last_ingested_contract_sequence,
                                &ingested_contracts,
                            )
                            .await;

                            if let Err(error) = ingestion_result {
                                error!("Error while ingesting contracts: {:?}", error);
                            }
                        }
                        Ok(IngestNextContract::No) => {}
                        Err(error) => {
                            error!("Error while checking for new contracts: {}", error);
                        }
                    }
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
