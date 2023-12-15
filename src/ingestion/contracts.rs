use log::info;
use migration::OnConflict;
use quasar_entities::{
    contract,
    contract_spec::{self},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use stellar_node_entities::{
    contractcode, contractdata, prelude::Contractcode, prelude::Contractdata,
};

use crate::{
    databases::{NodeDatabase, QuasarDatabase},
    ingestion::IngestionError,
};

use super::IngestionMetrics;

pub(super) async fn ingest_contracts(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    sequence: i32,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    while let Some(next_contract) = next_contract_to_ingest(node_database, sequence).await? {
        ingest_contract(next_contract.clone(), quasar_database).await?;
        metrics.contracts.inc();
    }

    Ok(())
}

pub(super) async fn ingest_contract_specs(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    sequence: i32,
) -> Result<(), IngestionError> {
    while let Some(next_contract_spec) =
        next_contract_spec_to_ingest(node_database, sequence).await?
    {
        ingest_contract_spec(next_contract_spec, quasar_database).await?;
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
    contract::Entity::insert(contract)
        .on_conflict(
            OnConflict::column(contract::Column::Address)
                .update_columns([contract::Column::LastModified])
                .to_owned(),
        )
        .exec(database.as_inner())
        .await?;
    Ok(sequence)
}

async fn ingest_contract_spec(
    contract: contractcode::Model,
    database: &QuasarDatabase,
) -> Result<i32, IngestionError> {
    let sequence = contract.lastmodified;
    let contract_spec = contract_spec::ActiveModel::try_from(contract).unwrap();
    contract_spec::Entity::insert(contract_spec)
        .on_conflict(
            OnConflict::column(contract_spec::Column::Address)
                .update_columns([
                    contract_spec::Column::LastModified,
                    contract_spec::Column::Spec,
                ])
                .to_owned(),
        )
        .exec(database.as_inner())
        .await?;
    Ok(sequence)
}
async fn next_contract_spec_to_ingest(
    node_database: &DatabaseConnection,
    last_ingested: i32,
) -> Result<Option<contractcode::Model>, IngestionError> {
    let next_contract_spec =
        Contractcode::find().filter(contractcode::Column::Lastmodified.gt(last_ingested));

    let next_contract_spec = next_contract_spec
        .order_by_asc(contractcode::Column::Lastmodified)
        .one(node_database)
        .await?;

    Ok(next_contract_spec)
}

async fn next_contract_to_ingest(
    node_database: &DatabaseConnection,
    last_ingested: i32,
) -> Result<Option<contractdata::Model>, IngestionError> {
    let next_contract =
        Contractdata::find().filter(contractdata::Column::Lastmodified.gt(last_ingested));
    let next_contract = next_contract
        .order_by_asc(contractdata::Column::Lastmodified)
        .one(node_database)
        .await?;

    Ok(next_contract)
}
