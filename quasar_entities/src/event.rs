use async_graphql::{dataloader::Loader, ComplexObject, Context};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Condition, Set};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use stellar_xdr::curr::{
    ContractEvent, ContractEventBody, Error as StellarXdrError, Limits, ScVal, WriteXdr,
};
use thiserror::Error;

use crate::{contract, transaction, QuasarDataLoader};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "events")]
#[graphql(complex)]
#[graphql(name = "Events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub topic: String,
    pub contract_id: String,
    pub transaction_id: String,
    pub value: Json,
    pub r#type: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] StellarXdrError),
    #[error("Serde error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Invalid event")]
    Invalid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
    #[sea_orm(
        belongs_to = "super::contract::Entity",
        from = "Column::ContractId",
        to = "super::contract::Column::Address"
    )]
    Contract,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl Related<super::contract::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contract.def()
    }
}

#[ComplexObject]
impl Model {
    pub async fn contract<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Option<super::contract::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(contract::Entity).one(database).await
    }

    pub async fn transaction<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Option<super::transaction::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(transaction::Entity).one(database).await
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<ContractEvent> for ActiveModel {
    type Error = EventError;

    fn try_from(event: ContractEvent) -> Result<Self, Self::Error> {
        let (topic, value) = match &event.body {
            ContractEventBody::V0(body) => {
                let topic = &body.topics[0];
                let topic = match topic {
                    ScVal::Symbol(topic) => topic.to_string(),
                    _ => Err(EventError::Invalid)?,
                };
                let value = val_to_json(&body.data).map_err(|_| EventError::Invalid)?;
                (topic, value)
            }
        };
        Ok(Self {
            id: NotSet,
            topic: Set(topic),
            contract_id: Set(stellar_strkey::Contract(
                event.contract_id.ok_or(EventError::Invalid)?.0,
            )
            .to_string()),
            transaction_id: NotSet,
            value: Set(value),
            r#type: Set(event.type_.to_string()),
            created_at: NotSet,
        })
    }
}

fn val_to_json(val: &ScVal) -> Result<Json, EventError> {
    let res = match val {
        ScVal::Bool(val) => json!(val),
        ScVal::Error(e) => e
            .to_xdr_base64(Limits::none())
            .map(|x| json!({ "error": x }))
            .map_err(|_| EventError::Invalid)?,
        ScVal::U32(val) => json!(val),
        ScVal::I32(val) => json!(val),
        ScVal::U64(val) => json!(val),
        ScVal::I64(val) => json!(val),
        ScVal::Timepoint(t) => json!(t.0),
        ScVal::Duration(d) => json!(d.0),
        ScVal::U128(val) => json!({
            "hi": val.hi,
            "low": val.lo
        }),
        ScVal::I128(val) => json!({
            "hi": val.hi,
            "low": val.lo
        }),
        ScVal::U256(val) => json!({
            "hi_hi": val.hi_hi,
            "hi_lo": val.hi_lo,
            "lo_hi": val.lo_hi,
            "lo_lo": val.lo_lo,
        }),
        ScVal::I256(val) => json!({
            "hi_hi": val.hi_hi,
            "hi_lo": val.hi_lo,
            "lo_hi": val.lo_hi,
            "lo_lo": val.lo_lo,
        }),
        ScVal::Bytes(val) => val
            .to_xdr_base64(Limits::none())
            .map(|x| json!({ "bytes_xdr": x }))
            .map_err(|_| EventError::Invalid)?,
        ScVal::String(val) => json!(val.to_string()),
        ScVal::Symbol(s) => json!({
            "symbol": s.to_string()
        }),
        ScVal::Vec(Some(val)) => val
            .iter()
            .map(val_to_json)
            .collect::<Result<Json, EventError>>()?,
        ScVal::Map(Some(map)) => map
            .iter()
            .map(|v| {
                let key = val_to_json(&v.key)?.to_string();
                let val = val_to_json(&v.val)?;
                Ok(json!({
                    key: val
                }))
            })
            .collect::<Result<Json, EventError>>()?,

        ScVal::Address(a) => json!({
            "address": println!("{a:?}")
        }),
        _ => Json::Null,
    };
    Ok(res)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventId(pub i32);

#[async_trait::async_trait]
impl Loader<EventId> for QuasarDataLoader {
    type Value = Model;
    type Error = Arc<DbErr>;

    async fn load(&self, keys: &[EventId]) -> Result<HashMap<EventId, Self::Value>, Self::Error> {
        let mut condition = Condition::any();

        for EventId(id) in keys {
            condition = condition.add(Column::Id.eq(*id));
        }
        let events = Entity::find()
            .filter(condition)
            .all(&self.pool)
            .await
            .map_err(Arc::new)?;
        Ok(events
            .into_iter()
            .map(|event| (EventId(event.id), event))
            .collect())
    }
}
