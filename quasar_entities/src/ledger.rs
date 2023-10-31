use crate::{account, QuasarDataLoader};
use async_graphql::{dataloader::Loader, ComplexObject, Context};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Condition, Set};
use std::{collections::HashMap, sync::Arc};
use stellar_node_entities::ledgerheaders;
use stellar_xdr::{Error, LedgerHeader, ReadXdr};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "ledgers")]
#[graphql(complex)]
#[graphql(name = "Ledgers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub hash: String,
    pub previous_ledger_hash: String,
    pub protocol_version: i32,
    pub sequence: i32,
    pub total_coins: i64,
    pub fee_pool: i64,
    pub inflation_seq: i32,
    pub id_pool: i64,
    pub base_fee: i32,
    pub base_reserve: i32,
    pub max_tx_set_size: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::account::Entity",
        to = "super::account::Column::LastModified",
        from = "Column::Sequence"
    )]
    Account,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    pub async fn accounts<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<account::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(account::Entity).all(database).await
    }
}

impl TryFrom<ledgerheaders::Model> for ActiveModel {
    type Error = Error;

    fn try_from(ledgerheaders: ledgerheaders::Model) -> Result<Self, Self::Error> {
        let ledgerheader_data = LedgerHeader::from_xdr_base64(ledgerheaders.data)?;

        Ok(Self {
            hash: Set(ledgerheaders.ledgerhash),
            previous_ledger_hash: Set(ledgerheaders.prevhash),
            protocol_version: Set(ledgerheader_data.ledger_version as i32),
            sequence: Set(ledgerheaders.ledgerseq.expect("ledgerseq is missing")),
            total_coins: Set(ledgerheader_data.total_coins),
            fee_pool: Set(ledgerheader_data.fee_pool),
            inflation_seq: Set(ledgerheader_data.inflation_seq as i32),
            id_pool: Set(ledgerheader_data.id_pool as i64),
            base_fee: Set(ledgerheader_data.base_fee as i32),
            base_reserve: Set(ledgerheader_data.base_reserve as i32),
            max_tx_set_size: Set(ledgerheader_data.max_tx_set_size as i32),
            created_at: NotSet,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LedgerHash(String);

#[async_trait::async_trait]
impl Loader<LedgerHash> for QuasarDataLoader {
    type Value = Model;
    type Error = Arc<DbErr>;

    async fn load(
        &self,
        keys: &[LedgerHash],
    ) -> Result<HashMap<LedgerHash, Self::Value>, Self::Error> {
        let mut condition = Condition::any();

        for LedgerHash(id) in keys {
            condition = condition.add(Column::Hash.eq(id.clone()));
        }
        let ledgers = Entity::find()
            .filter(condition)
            .all(&self.pool)
            .await
            .map_err(Arc::new)?;
        Ok(ledgers
            .into_iter()
            .map(|ledger| (LedgerHash(ledger.hash.clone()), ledger))
            .collect())
    }
}
