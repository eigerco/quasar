use quasar_entities::event;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use stellar_xdr::TransactionMeta;

use super::IngestionError;

pub async fn ingest_events(
    db: &DatabaseConnection,
    transaction_meta: TransactionMeta,
    ledger_seq: i32,
) -> Result<(), IngestionError> {
    match transaction_meta {
        TransactionMeta::V3(v3) => match v3.soroban_meta {
            None => return Ok(()),
            Some(meta) => {
                let events = meta.events;
                for event in events.iter() {
                    let mut event: event::ActiveModel =
                        event::ActiveModel::try_from(event.clone())?;
                    event.ledger = Set(ledger_seq);
                    event.insert(db).await?;
                }
            }
        },
        _ => log::warn!("We only consume soroban events"),
    }
    Ok(())
}
