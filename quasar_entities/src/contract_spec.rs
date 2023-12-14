use crate::QuasarDataLoader;
use async_graphql::dataloader::Loader;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, FromJsonQueryResult, Set, Condition};
use std::collections::HashMap;
use std::sync::Arc;
use stellar_node_entities::contractcode;
use stellar_xdr::curr::{Error, LedgerEntry, Limits, ReadXdr};
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    async_graphql::SimpleObject,
    serde::Deserialize,
    FromJsonQueryResult,
)]
pub struct Function {
    name: String,
    input: HashMap<String, serde_json::Value>,
    output: HashMap<String, serde_json::Value>,
    docs: String,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    async_graphql::SimpleObject,
    serde::Deserialize,
    FromJsonQueryResult,
)]
pub struct FunctionSpec {
    functions: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "contracts_spec")]
// #[graphql(complex)]
#[graphql(name = "ContractSpec")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    #[sea_orm(column_type = "Json")]
    pub spec: FunctionSpec,
    pub last_modified: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contract::Entity",
        from = "Column::Address",
        to = "super::contract::Column::Address"
    )]
    Contract,
}

impl Related<super::contract::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contract.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<contractcode::Model> for ActiveModel {
    type Error = Error;

    fn try_from(model: contractcode::Model) -> Result<Self, Self::Error> {
        let entry = LedgerEntry::from_xdr_base64(model.ledgerentry, Limits::none())?;
        let address = match &entry.data {
            soroban_env_host::xdr::LedgerEntryData::ContractCode(c) => {
                Ok(stellar_strkey::Contract(c.hash.0).to_string())
            }
            _ => Err(Error::Invalid),
        }?;
        let mut functions = Vec::new();
        match &entry.data {
            stellar_xdr::curr::LedgerEntryData::ContractCode(c) => {
                let wasm = c.code.as_slice();
                let spec = soroban_spec::read::from_wasm(wasm).unwrap();

                for item in spec {
                    match &item {
                        stellar_xdr::curr::ScSpecEntry::FunctionV0(func) => {
                            functions.push(Function {
                                docs: func.doc.to_string(),
                                name: func.name.to_utf8_string_lossy(),
                                input: func
                                    .inputs
                                    .iter()
                                    .map(|input| {
                                        (
                                            input.name.to_string(),
                                            serde_json::to_value(input).unwrap(),
                                        )
                                    })
                                    .collect(),
                                output: func
                                    .outputs
                                    .iter()
                                    .map(|output| {
                                        (
                                            output.name().to_string(),
                                            serde_json::to_value(output).unwrap(),
                                        )
                                    })
                                    .collect(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        };
        Ok(Self {
            address: Set(address),
            spec: Set(FunctionSpec { functions }),
            last_modified: Set(model.lastmodified),
            created_at: NotSet,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContractSpec(pub String);

#[async_trait::async_trait]
impl Loader<ContractSpec> for QuasarDataLoader {
    type Value = Model;
    type Error = Arc<DbErr>;

    async fn load(
        &self,
        keys: &[ContractSpec],
    ) -> Result<HashMap<ContractSpec, Self::Value>, Self::Error> {
        let mut condition = Condition::any();

        for ContractSpec(address) in keys {
            condition = condition.add(Column::Address.eq(address.clone()));
        }
        let contracts = Entity::find()
            .filter(condition)
            .all(&self.pool)
            .await
            .map_err(Arc::new)?;
        Ok(contracts
            .into_iter()
            .map(|contract| (ContractSpec(contract.address.clone()), contract))
            .collect())
    }
}
