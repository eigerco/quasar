use crate::transaction;
use async_graphql::{ComplexObject, Context};
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
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

impl TryFrom<Operation> for ActiveModel {
    type Error = Error;

    fn try_from(operation: Operation) -> Result<Self, Self::Error> {
        Ok(Self {
            id: NotSet,
            transaction_id: NotSet,
            application_order: NotSet,
            r#type: Set(operation.body.name().to_string()),
        })
    }
}
