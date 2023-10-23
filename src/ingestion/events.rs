// Events are stored as operations in transactions.
// https://github.com/stellar/go/blob/776596261aa19191100a42c1534e87346567f3f4/exp/services/soroban-rpc/internal/methods/get_events.go#L283

use quasar_entities::event;
use sea_orm::DatabaseConnection;
use stellar_node_entities::txhistory;
use stellar_xdr::{ReadXdr, TransactionMeta};

pub struct Transaction(TransactionMeta);

pub async fn ingest_events(
    db: &DatabaseConnection,
    transaction_meta: TransactionMeta,
) -> Result<(), IngestionError> {
    // let transaction_meta = TransactionMeta::from_xdr_base64(stellar_node_transaction.txmeta)?;
    match transaction_meta {
        TransactionMeta::V3(v3) => match v3.soroban_meta {
            None => return,
            Some(meta) => {
                let events = meta.events;
                for event in events.into_iter() {
                    let mut event: event::ActiveModel = event::ActiveModel::try_from(event)?;
                    event.ledger = Set(stellar_node_transaction.ledgerseq);
                    event.insert(db).await?;
                }
            }
        },
        _ => unimplemented!("We only consume soroban events"),
    }
    Ok(())
}
