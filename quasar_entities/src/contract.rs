use async_graphql::dataloader::Loader;
use async_graphql::{ComplexObject, Context};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet};
use sea_orm::{Condition, Set};
use std::collections::HashMap;
use std::sync::Arc;
use stellar_node_entities::contractdata;
use stellar_xdr::{Error, LedgerEntry, ReadXdr};

use crate::{event, QuasarDataLoader};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "contracts")]
#[graphql(complex)]
#[graphql(name = "Contracts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    #[sea_orm(unique)]
    pub hash: String,
    pub key: String,
    pub r#type: String,
    pub last_modified: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::event::Entity",
        to = "super::event::Column::TransactionId",
        from = "Column::Address"
    )]
    Event,
}

impl Related<super::event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Event.def()
    }
}

#[ComplexObject]
impl Model {
    pub async fn events<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<event::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(event::Entity).all(database).await
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<contractdata::Model> for ActiveModel {
    type Error = Error;

    fn try_from(model: contractdata::Model) -> Result<Self, Self::Error> {
        let entry = LedgerEntry::from_xdr_base64(model.ledgerentry)?;
        let address = match entry.data {
            soroban_env_host::xdr::LedgerEntryData::ContractData(c) => match c.contract {
                soroban_env_host::xdr::ScAddress::Contract(hash) => {
                    Ok(stellar_strkey::Contract(hash.0).to_string())
                }
                _ => Err(Error::Invalid),
            },
            _ => Err(Error::Invalid),
        }?;
        Ok(Self {
            r#type: Set(model.r#type.to_string()),
            key: Set(model.key),
            hash: Set(model.contractid.clone()),
            address: Set(address),
            last_modified: Set(model.lastmodified),
            created_at: NotSet,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContractId(pub String);

#[async_trait::async_trait]
impl Loader<ContractId> for QuasarDataLoader {
    type Value = Model;
    type Error = Arc<DbErr>;

    async fn load(
        &self,
        keys: &[ContractId],
    ) -> Result<HashMap<ContractId, Self::Value>, Self::Error> {
        let mut condition = Condition::any();

        for ContractId(address) in keys {
            condition = condition.add(Column::Address.eq(address.clone()));
        }
        let contracts = Entity::find()
            .filter(condition)
            .all(&self.pool)
            .await
            .map_err(Arc::new)?;
        Ok(contracts
            .into_iter()
            .map(|contract| (ContractId(contract.address.clone()), contract))
            .collect())
    }
}
