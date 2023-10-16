use super::{sleep, IngestionError};
use crate::configuration::Ingestion;
use log::{debug, error, info};
use prometheus::{IntCounter, Registry};
use quasar_entities::{ledger, prelude::Ledger};
use sea_orm::{ActiveModelTrait, ColumnTrait, QueryFilter};
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryOrder};
use stellar_node_entities::ledgerheaders;
use stellar_node_entities::prelude::Ledgerheaders;

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

pub async fn ingest_ledgers(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    mut last_ingested_ledger_sequence: Option<i32>,
    counter: &IntCounter,
) -> Result<(), IngestionError> {
    while let Some(next_ledger) =
        next_ledger_to_ingest(node_database, last_ingested_ledger_sequence).await?
    {
        let ingested_sequence = ingest_ledger(next_ledger, quasar_database).await?;
        last_ingested_ledger_sequence = Some(ingested_sequence);

        counter.inc();
    }

    Ok(())
}

async fn ingest_ledger(
    ledger: ledgerheaders::Model,
    db: &DatabaseConnection,
) -> Result<i32, IngestionError> {
    let sequence = ledger
        .ledgerseq
        .ok_or(IngestionError::MissingLedgerSequence)?;
    info!("Ingesting ledger {}", sequence);

    let ledger: ledger::ActiveModel = ledger::ActiveModel::try_from(ledger).unwrap();

    ledger.insert(db).await?;

    Ok(sequence)
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

pub async fn ingest(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    ingestion: &Ingestion,
    metrics: &Registry,
) {
    let ingested_ledgers = setup_metrics(metrics.clone());
    loop {
        sleep(ingestion).await;

        // Handle ledgers
        let ingestion_needed = new_ledgers_available(node_database, quasar_database).await;

        match ingestion_needed {
            Ok(IngestionNeeded::Yes {
                last_ingested_ledger_sequence,
            }) => {
                debug!("New ledgers available");
                let ingestion_result = ingest_ledgers(
                    node_database,
                    quasar_database,
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
