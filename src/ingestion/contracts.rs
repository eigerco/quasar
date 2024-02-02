use migration::OnConflict;
use quasar_entities::contract;
use sea_orm::EntityTrait;
use stellar_xdr::curr::ContractDataEntry;

use crate::databases::QuasarDatabase;

use super::IngestionError;

pub async fn ingest_contract(
    database: &QuasarDatabase,
    contract: ContractDataEntry,
) -> Result<(), IngestionError> {
    let contract: contract::ActiveModel = contract::ActiveModel::try_from(contract).unwrap();
    contract::Entity::insert(contract)
        .on_conflict(
            OnConflict::column(contract::Column::Address)
                .update_columns([
                    contract::Column::LastModified,
                    contract::Column::Hash,
                    contract::Column::Key,
                    contract::Column::Type,
                ])
                .to_owned(),
        )
        .exec(&**database)
        .await?;
    Ok(())
}
