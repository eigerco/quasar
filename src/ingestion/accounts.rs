use crate::databases::QuasarDatabase;

use super::IngestionError;

use migration::OnConflict;
use quasar_entities::account;
use sea_orm::EntityTrait;
use stellar_xdr::curr::AccountEntry;

pub(super) async fn ingest_account(
    db: &QuasarDatabase,
    account: AccountEntry,
) -> Result<(), IngestionError> {
    let account: account::ActiveModel = account::ActiveModel::try_from(account)?;

    account::Entity::insert(account)
        .on_conflict(
            OnConflict::column(account::Column::Id)
                .update_columns([
                    account::Column::LastModified,
                    account::Column::Balance,
                    account::Column::BuyingLiabilities,
                    account::Column::HomeDomain,
                    account::Column::InflationDestination,
                    account::Column::MasterWeight,
                    account::Column::NumberOfSubentries,
                    account::Column::SellingLiabilities,
                    account::Column::SequenceNumber,
                ])
                .to_owned(),
        )
        .exec(db.as_inner())
        .await?;
    Ok(())
}
