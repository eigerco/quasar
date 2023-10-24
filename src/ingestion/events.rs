use quasar_entities::event;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use stellar_xdr::TransactionMeta;

use super::IngestionError;

pub async fn ingest_events(
    db: &DatabaseConnection,
    transaction_meta: TransactionMeta,
    transaction_id: &str,
) -> Result<(), IngestionError> {
    match transaction_meta {
        TransactionMeta::V3(v3) => match v3.soroban_meta {
            None => return Ok(()),
            Some(meta) => {
                let events = meta.events;
                let event_count = events.len();
                log::info!("Consuming {event_count} events from transaction: {transaction_id}");
                for event in events.iter() {
                    let mut event: event::ActiveModel =
                        event::ActiveModel::try_from(event.clone())?;
                    event.transaction_id = Set(transaction_id.to_owned());
                    event.insert(db).await.unwrap();
                }
            }
        },
        _ => log::warn!("We only consume soroban events"),
    }
    Ok(())
}
