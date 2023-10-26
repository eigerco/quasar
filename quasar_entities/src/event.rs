use async_graphql::ComplexObject;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde_json::json;
use stellar_xdr::{ContractEvent, Error, ScVal, WriteXdr};
use thiserror::Error;

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

#[ComplexObject]
impl Model {}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("XDR decoding error: {0}")]
    XdrError(#[from] stellar_xdr::Error),
    #[error("Serde error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Invalid event")]
    Invalid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<ContractEvent> for ActiveModel {
    type Error = EventError;

    fn try_from(event: ContractEvent) -> Result<Self, Self::Error> {
        let (topic, value) = match &event.body {
            stellar_xdr::ContractEventBody::V0(body) => {
                let topic = &body.topics[0];
                let topic = match topic {
                    ScVal::Symbol(topic) => topic.to_string()?,
                    _ => Err(EventError::Invalid)?,
                };
                let value = val_to_json(&body.data).map_err(|_| Error::Invalid)?;
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
            created_at: NotSet

        })
    }
}

fn val_to_json(val: &ScVal) -> Result<Json, EventError> {
    let res = match val {
        ScVal::Bool(val) => json!(val),
        ScVal::Error(e) => json!({
            "error": e.to_xdr_base64()?
        }),
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
        ScVal::Bytes(val) => json!({
            "bytes_xdr": val.to_xdr_base64()?
        }),
        ScVal::String(val) => json!(val.to_string()?),
        ScVal::Symbol(s) => json!({
            "symbol": s.to_string()?
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
