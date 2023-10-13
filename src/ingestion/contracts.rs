use log::info;
use prometheus::IntCounter;
use quasar_entities::contract;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use stellar_node_entities::{contractdata, prelude::Contractdata};

use crate::ingestion::IngestionError;

pub async fn ingest_contracts(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    mut last_ingested_contract_sequence: Option<i32>,
    counter: &IntCounter,
) -> Result<(), IngestionError> {
    while let Some(next_contract) =
        next_contract_to_ingest(node_database, last_ingested_contract_sequence).await?
    {
        let ingested_sequence = ingest_contract(next_contract, quasar_database).await?;
        last_ingested_contract_sequence = Some(ingested_sequence);

        counter.inc();
    }

    Ok(())
}

async fn ingest_contract(
    contract: contractdata::Model,
    db: &DatabaseConnection,
) -> Result<i32, IngestionError> {
    let sequence = contract
        .contractid
        .parse()
        .map_err(|_| IngestionError::MissingLedgerSequence)?;
    info!("Ingesting contract {}", sequence);

    let contract: contract::ActiveModel = contract::ActiveModel::try_from(contract)?;

    contract.insert(db).await?;

    Ok(sequence)
}

async fn next_contract_to_ingest(
    node_database: &DatabaseConnection,
    last_ingested_contract_sequence: Option<i32>,
) -> Result<Option<contractdata::Model>, IngestionError> {
    let next_contract = Contractdata::find();

    let next_contract = match last_ingested_contract_sequence {
        Some(last_ingested_contract_sequence) => next_contract
            .filter(contractdata::Column::Lastmodified.gt(last_ingested_contract_sequence)),

        None => next_contract,
    };

    let next_contract = next_contract
        .order_by_asc(contractdata::Column::Lastmodified)
        .one(node_database)
        .await?;

    Ok(next_contract)
}
