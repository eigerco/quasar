use quasar_entities::operation;
use sea_orm::{ActiveModelTrait, Set};
use stellar_xdr::curr::{Operation, TransactionEnvelope};

use crate::databases::QuasarDatabase;

use super::{IngestionError, IngestionMetrics};

pub(super) async fn ingest_operations(
    db: &QuasarDatabase,
    transaction_id: &str,
    transaction_tx_body: TransactionEnvelope,
    metrics: &IngestionMetrics,
) -> Result<(), IngestionError> {
    let operations: Vec<Operation> = match transaction_tx_body {
        TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.to_vec(),
        TransactionEnvelope::Tx(envelope) => envelope.tx.operations.to_vec(),
        TransactionEnvelope::TxFeeBump(_) => vec![],
    };

    for (index, operation) in operations.into_iter().enumerate() {
        let mut operation: operation::ActiveModel = operation::ActiveModel::try_from(operation)?;

        operation.transaction_id = Set(transaction_id.to_owned());
        operation.application_order = Set(index as i32 + 1);

        operation.insert(db.as_inner()).await?;

        metrics.operations.inc();
    }

    Ok(())
}
