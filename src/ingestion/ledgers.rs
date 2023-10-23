use crate::databases::{NodeDatabase, QuasarDatabase};
use crate::ingestion::contracts::ingest_contracts;
use crate::ingestion::{accounts::ingest_accounts, transactions::ingest_transactions};
use crate::metrics::IngestionMetrics;
use log::info;
use quasar_entities::{ledger, prelude::Ledger};
use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder};
use stellar_node_entities::ledgerheaders;
use stellar_node_entities::prelude::Ledgerheaders;

use super::IngestionError;

pub enum IngestionNeeded {
    Yes {
        last_ingested_ledger_sequence: Option<i32>,
    },
    No,
}

pub(super) async fn new_ledgers_available(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
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
    quasar_database: &QuasarDatabase,
) -> Result<Option<i32>, DbErr> {
    let last_ingested_ledger = Ledger::find()
        .order_by_desc(ledger::Column::Sequence)
        .one(quasar_database.as_inner())
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

pub(super) async fn ingest_ledgers(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    mut last_ingested_ledger_sequence: Option<i32>,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    while let Some(next_ledger) =
        next_ledger_to_ingest(node_database, last_ingested_ledger_sequence).await?
    {
        let ingested_sequence =
            handle_new_ledger(next_ledger, quasar_database, node_database, metrics).await?;
        last_ingested_ledger_sequence = Some(ingested_sequence);

        metrics.ingested_ledgers.inc();
    }

    Ok(())
}

async fn handle_new_ledger(
    ledger: ledgerheaders::Model,
    quasar_database: &QuasarDatabase,
    node_database: &NodeDatabase,
    metrics: &IngestionMetrics,
) -> Result<i32, IngestionError> {
    let sequence = ledger
        .ledgerseq
        .ok_or(IngestionError::MissingLedgerSequence)?;
    info!("Ingesting ledger {} and associated data", sequence);

    ingest_ledger(ledger, quasar_database).await?;
    ingest_accounts(node_database, quasar_database, sequence).await?;
    ingest_transactions(node_database, quasar_database, sequence).await?;
    ingest_contracts(node_database, quasar_database, metrics).await?;

    Ok(sequence)
}

async fn ingest_ledger(
    ledger: ledgerheaders::Model,
    quasar_database: &DatabaseConnection,
) -> Result<(), IngestionError> {
    let ledger: ledger::ActiveModel = ledger::ActiveModel::try_from(ledger)?;
    ledger.insert(quasar_database).await?;
    Ok(())
}

async fn next_ledger_to_ingest(
    node_database: &DatabaseConnection,
    last_ingested_ledger_sequence: Option<i32>,
) -> Result<Option<ledgerheaders::Model>, IngestionError> {
    let next_ledger = Ledgerheaders::find();

    let next_ledger = match last_ingested_ledger_sequence {
        Some(last_ingested_ledger_sequence) => {
            next_ledger.filter(ledgerheaders::Column::Ledgerseq.gt(last_ingested_ledger_sequence))
        }

        None => next_ledger,
    };

    let next_ledger = next_ledger
        .order_by_asc(ledgerheaders::Column::Ledgerseq)
        .one(node_database)
        .await?;

    Ok(next_ledger)
}
