use async_graphql::ComplexObject;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use stellar_xdr::{Error, Operation};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "operations")]
#[graphql(complex)]
#[graphql(name = "Operations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    id: i32,
    transaction_id: String,
    application_order: i32,
    r#type: String,
    pub created_at: DateTimeWithTimeZone, 
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {}

impl TryFrom<Operation> for ActiveModel {
    type Error = Error;

    fn try_from(operation: Operation) -> Result<Self, Self::Error> {
        Ok(Self {
            id: NotSet,
            transaction_id: NotSet,
            application_order: NotSet,
            r#type: Set(operation.body.name().to_string()),
            created_at: NotSet
        })
    }
}
