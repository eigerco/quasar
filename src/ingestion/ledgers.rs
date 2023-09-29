use log::info;
use quasar_entities::ledger;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use stellar_node_entities::{ledgerheaders, prelude::Ledgerheaders};

use crate::ingestion::IngestionError;

pub async fn ingest_ledgers(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    mut last_ingested_ledger_sequence: Option<i32>,
) -> Result<(), IngestionError> {
    while let Some(next_ledger) =
        next_ledger_to_ingest(node_database, last_ingested_ledger_sequence).await?
    {
        let ingested_sequence = ingest_ledger(next_ledger, quasar_database).await?;
        last_ingested_ledger_sequence = Some(ingested_sequence);
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

    let ledger: ledger::ActiveModel = ledger::ActiveModel::try_from(ledger)?;

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
