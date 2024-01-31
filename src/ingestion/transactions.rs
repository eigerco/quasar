use quasar_entities::transaction;
use sea_orm::{ActiveModelTrait, Set};
use stellar_xdr::curr::{Limits, ReadXdr, TransactionEnvelope, TransactionMeta};

use crate::databases::QuasarDatabase;

use super::{
    events::ingest_events, operations::ingest_operations, IngestionError, IngestionMetrics,
};

pub struct Transaction {
    pub txid: String,
    pub ledgerseq: i32,
    pub txindex: i32,
    pub txbody: String,
    pub txresult: String,
    pub txmeta: String,
}

pub(super) async fn ingest_transaction(
    db: &QuasarDatabase,
    stellar_node_transaction: Transaction,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    let transaction_body =
        TransactionEnvelope::from_xdr_base64(&stellar_node_transaction.txbody, Limits::none())?;
    let transaction_meta =
        TransactionMeta::from_xdr_base64(&stellar_node_transaction.txmeta, Limits::none())?;
    let mut transaction: transaction::ActiveModel =
        transaction::ActiveModel::try_from(transaction_body.clone())?;

    transaction.id = Set(stellar_node_transaction.txid.clone());
    transaction.application_order = Set(stellar_node_transaction.txindex);
    transaction.ledger_sequence = Set(stellar_node_transaction.ledgerseq);
    transaction.insert(db.as_inner()).await?;

    ingest_operations(
        db,
        &stellar_node_transaction.txid,
        transaction_body,
        metrics,
    )
    .await?;

    ingest_events(
        db,
        transaction_meta,
        &stellar_node_transaction.txid,
        metrics,
    )
    .await?;

    Ok(())
}
