use log::{debug, error, info};
use prometheus::{IntCounter, Registry};
use quasar_entities::contract;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use stellar_node_entities::{contractdata, prelude::Contractdata};

use crate::{configuration::Ingestion, ingestion::IngestionError};

use super::sleep;

fn setup_contract_metrics(
    metrics: &Registry,
) -> prometheus::core::GenericCounter<prometheus::core::AtomicU64> {
    let ingested_contract_counter =
        IntCounter::new("ingested_contracts", "Number of ingested contracts").unwrap();
    metrics
        .register(Box::new(ingested_contract_counter.clone()))
        .expect("Failed to register counter");
    ingested_contract_counter
}

pub async fn ingest_contracts(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    mut last_ingested: Option<i32>,
    counter: &IntCounter,
) -> Result<(), IngestionError> {
    while let Some(next_contract) = next_contract_to_ingest(node_database, last_ingested).await? {
        let ingested_sequence = ingest_contract(next_contract, quasar_database).await?;
        last_ingested = Some(ingested_sequence);

        counter.inc();
    }

    Ok(())
}

async fn ingest_contract(
    contract: contractdata::Model,
    db: &DatabaseConnection,
) -> Result<i32, IngestionError> {
    let sequence = contract.lastmodified;
    info!("Ingesting contract since {}", sequence);
    let contract: contract::ActiveModel = contract::ActiveModel::try_from(contract).unwrap();
    contract.insert(db).await?;
    Ok(sequence)
}

async fn next_contract_to_ingest(
    node_database: &DatabaseConnection,
    last_ingested: Option<i32>,
) -> Result<Option<contractdata::Model>, IngestionError> {
    let next_contract = Contractdata::find();

    let next_contract = match last_ingested {
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

enum IngestNextContract {
    Yes {
        last_ingested_contract_sequence: Option<i32>,
    },
    No,
}

async fn new_contracts_available(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
) -> Result<IngestNextContract, DbErr> {
    let last_ingested_contract_sequence = last_ingested_contract_sequence(quasar_database).await?;
    let last_stellar_contract_sequence = last_stellar_contract_sequence(node_database).await?;
    let ingestion_needed = if last_ingested_contract_sequence != last_stellar_contract_sequence {
        IngestNextContract::Yes {
            last_ingested_contract_sequence,
        }
    } else {
        IngestNextContract::No
    };

    Ok(ingestion_needed)
}

async fn last_ingested_contract_sequence(
    quasar_database: &DatabaseConnection,
) -> Result<Option<i32>, DbErr> {
    let last_ingested_ledger = contract::Entity::find()
        .order_by_desc(contract::Column::LastModified)
        .one(quasar_database)
        .await?;
    let last_ingested_contract_sequence = last_ingested_ledger.map(|ledger| ledger.last_modified);
    Ok(last_ingested_contract_sequence)
}

async fn last_stellar_contract_sequence(
    node_database: &DatabaseConnection,
) -> Result<Option<i32>, DbErr> {
    let last_stellar_ledger = Contractdata::find()
        .order_by_desc(contractdata::Column::Lastmodified)
        .one(node_database)
        .await?;
    let last_stellar_contract_sequence =
        last_stellar_ledger.map(|ledger| ledger.lastmodified);
    Ok(last_stellar_contract_sequence)
}

pub async fn ingest(
    node_database: &DatabaseConnection,
    quasar_database: &DatabaseConnection,
    ingestion: &Ingestion,
    metrics: &Registry,
) {
    let ingested_contracts = setup_contract_metrics(metrics);
    loop {
        sleep(ingestion).await;

        let ingestion_needed = new_contracts_available(node_database, quasar_database).await;

        match ingestion_needed {
            Ok(IngestNextContract::Yes {
                last_ingested_contract_sequence,
            }) => {
                debug!("New contracts available");
                let ingestion_result = ingest_contracts(
                    node_database,
                    quasar_database,
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
