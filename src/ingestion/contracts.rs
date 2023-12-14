use log::{debug, info};
use quasar_entities::{contract, contract_spec};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};
use stellar_node_entities::{contractdata, prelude::Contractdata, contractcode};

use crate::{
    databases::{NodeDatabase, QuasarDatabase},
    ingestion::IngestionError,
};

use super::IngestionMetrics;

pub(super) async fn ingest_contracts(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    let ingestion_needed = new_contracts_available(node_database, quasar_database).await?;

    match ingestion_needed {
        IngestNextContract::Yes {
            last_ingested_contract_sequence,
        } => {
            debug!("New contracts available");
            let mut last_ingested = last_ingested_contract_sequence;
            while let Some(next_contract) =
                next_contract_to_ingest(node_database, last_ingested).await?
            {
                let ingested_sequence = ingest_contract(next_contract.clone(), quasar_database).await?;
                ingest_contract_spec(next_contract, quasar_database, node_database).await?;
                last_ingested = Some(ingested_sequence);

                metrics.contracts.inc();
            }
        }
        IngestNextContract::No => {}
    }

    Ok(())
}

async fn ingest_contract(
    contract: contractdata::Model,
    database: &QuasarDatabase,
) -> Result<i32, IngestionError> {
    let sequence = contract.lastmodified;
    info!("Ingesting contract since {}", sequence);
    let contract: contract::ActiveModel = contract::ActiveModel::try_from(contract).unwrap();
    contract.insert(&**database).await?;
    Ok(sequence)
}

async fn ingest_contract_spec(
    contract: contractdata::Model,
    database: &QuasarDatabase,
    stellar_core: &NodeDatabase,
) -> Result<(), IngestionError> {
    let contract_code = contractcode::Entity::find_by_id(contract.contractid).one(&**stellar_core).await?.unwrap();
    let contract_spec = contract_spec::ActiveModel::try_from(contract_code).unwrap();
    contract_spec.insert(&**database).await?;
    Ok(())
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

pub enum IngestNextContract {
    Yes {
        last_ingested_contract_sequence: Option<i32>,
    },
    No,
}

pub(super) async fn new_contracts_available(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
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
    let last_stellar_contract_sequence = last_stellar_ledger.map(|ledger| ledger.lastmodified);
    Ok(last_stellar_contract_sequence)
}
