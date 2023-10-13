use async_graphql::ComplexObject;
use sea_orm::entity::prelude::*;
use sea_orm::Set;
use stellar_node_entities::contractdata;
use stellar_xdr::{Error, LedgerEntry, ReadXdr};

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
    pub last_modified: DateTime,
}

#[ComplexObject]
impl Model {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

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
            last_modified: Set(DateTime::from_timestamp_millis(model.lastmodified.into()).unwrap()),
        })
    }
}
