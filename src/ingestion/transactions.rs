use log::info;
use quasar_entities::transaction;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use stellar_node_entities::{prelude::Txhistory, txhistory};
use stellar_xdr::{ReadXdr, TransactionEnvelope};

use crate::databases::{NodeDatabase, QuasarDatabase};

use super::{operations::ingest_operations, IngestionError};

pub(super) async fn ingest_transactions(
    node_database: &NodeDatabase,
    quasar_database: &QuasarDatabase,
    ledger_sequence: i32,
) -> Result<(), IngestionError> {
    // Query all transactions with lastmodified = ledger_sequence
    let updated_transactions = Txhistory::find()
        .filter(stellar_node_entities::txhistory::Column::Ledgerseq.eq(ledger_sequence))
        .all(node_database.as_inner())
        .await?;

    let count = updated_transactions.len();

    // Ingest all updated transactions
    for transaction in updated_transactions {
        ingest_transaction(quasar_database, transaction).await?;
    }

    info!("Ingested {} transactions", count);

    Ok(())
}

pub(super) async fn ingest_transaction(
    db: &QuasarDatabase,
    stellar_node_transaction: txhistory::Model,
) -> Result<(), IngestionError> {
    let transaction_body = TransactionEnvelope::from_xdr_base64(stellar_node_transaction.txbody)?;

    let mut transaction: transaction::ActiveModel =
        transaction::ActiveModel::try_from(transaction_body.clone())?;

    transaction.id = Set(stellar_node_transaction.txid.clone());
    transaction.application_order = Set(stellar_node_transaction.txindex);
    transaction.ledger_sequence = Set(stellar_node_transaction.ledgerseq);
    transaction.insert(db.as_inner()).await?;

    ingest_operations(db, stellar_node_transaction.txid, transaction_body).await?;

    Ok(())
}
