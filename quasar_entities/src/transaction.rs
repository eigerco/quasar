use async_graphql::ComplexObject;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::{Error, TransactionEnvelope};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "transactions")]
#[graphql(complex)]
#[graphql(name = "Transactions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub ledger_sequence: i32,
    pub application_order: i32,
    pub account_id: String,
    pub account_sequence: i64,
    pub operation_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {}

impl TryFrom<TransactionEnvelope> for ActiveModel {
    type Error = Error;

    fn try_from(transaction: TransactionEnvelope) -> Result<Self, Self::Error> {
        let account = match &transaction {
            TransactionEnvelope::TxV0(envelope) => envelope.tx.source_account_ed25519.clone(),
            TransactionEnvelope::Tx(envelope) => {
                let muxed_account = envelope.tx.source_account.clone();

                match muxed_account {
                    stellar_xdr::MuxedAccount::Ed25519(account) => account,
                    stellar_xdr::MuxedAccount::MuxedEd25519(account) => account.ed25519,
                }
            }
            TransactionEnvelope::TxFeeBump(bump) => {
                let muxed_account = bump.tx.fee_source.clone();

                match muxed_account {
                    stellar_xdr::MuxedAccount::Ed25519(account) => account,
                    stellar_xdr::MuxedAccount::MuxedEd25519(account) => account.ed25519,
                }
            }
        };

        let account_str_key = PublicKey(account.0);

        let operation_count: i32 = match &transaction {
            TransactionEnvelope::TxV0(envelope) => envelope.tx.operations.len() as i32,
            TransactionEnvelope::Tx(envelope) => envelope.tx.operations.len() as i32,
            TransactionEnvelope::TxFeeBump(_) => 1,
        };

        let seq_num = match transaction {
            TransactionEnvelope::TxV0(envelope) => envelope.tx.seq_num.0,
            TransactionEnvelope::Tx(envelope) => envelope.tx.seq_num.0,
            TransactionEnvelope::TxFeeBump(envelope) => match envelope.tx.inner_tx {
                stellar_xdr::FeeBumpTransactionInnerTx::Tx(tx) => tx.tx.seq_num.0,
            },
        };

        Ok(Self {
            id: NotSet,
            ledger_sequence: NotSet,
            application_order: NotSet,
            account_id: Set(account_str_key.to_string()),
            account_sequence: Set(seq_num),
            operation_count: Set(operation_count),
        })
    }
}
