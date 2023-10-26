use async_graphql::{ComplexObject, Context};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use stellar_strkey::ed25519::PublicKey;
use stellar_xdr::{Error, TransactionEnvelope};

use crate::{account, event, operation};

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
pub enum Relation {
    #[sea_orm(
        has_many = "super::operation::Entity",
        to = "super::operation::Column::TransactionId",
        from = "Column::Id"
    )]
    Operation,
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::AccountId",
        to = "super::account::Column::Id"
    )]
    Account,
    #[sea_orm(
        has_many = "super::event::Entity",
        to = "super::event::Column::TransactionId",
        from = "Column::Id"
    )]
    Event,
}

impl Related<super::operation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Operation.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    pub async fn account<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Option<super::account::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(account::Entity).one(database).await
    }

    pub async fn operations<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Vec<operation::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(operation::Entity).all(database).await
    }

    pub async fn events<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<event::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(event::Entity).all(database).await
    }
}

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
