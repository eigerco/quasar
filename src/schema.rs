use async_graphql::{
    connection::{Connection, DefaultConnectionName, DefaultEdgeName, EmptyFields},
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use log::info;
use quasar_entities::{account, contract, event, ledger, operation, transaction};
use sea_orm::{EntityTrait, QuerySelect};

use crate::{
    databases::QuasarDatabase,
    pagination::{cursor_pagination, ConnectionParams},
};

pub(crate) type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub type PaginationCursorConnection<
    Cursor,
    Node,
    ConnectionName = DefaultConnectionName,
    EdgeName = DefaultEdgeName,
> = Connection<Cursor, Node, EmptyFields, EmptyFields, ConnectionName, EdgeName>;

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }

    async fn ledgers(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<Vec<ledger::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(ledger::Entity::find().all(database).await?)
    }
    async fn contracts(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<Vec<contract::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(contract::Entity::find().all(database).await?)
    }

    async fn accounts(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<Vec<account::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(account::Entity::find().all(database).await?)
    }

    async fn events(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<PaginationCursorConnection<i32, event::Model>> {
        cursor_pagination(
            page,
            |after: Option<i32>, before, limit| {
                let database = ctx.data::<QuasarDatabase>().unwrap().as_inner();
                event::Entity::find()
                    .limit(limit.unwrap_or(10))
                    .offset(after.map(|after| after.try_into().unwrap()))
                    .all(database)
            },
            |node: &event::Model| node.id.try_into().unwrap(),
        )
        .await
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<Vec<transaction::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(transaction::Entity::find().all(database).await?)
    }

    async fn operations(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<Vec<operation::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(operation::Entity::find().all(database).await?)
    }
}

pub(crate) fn build_schema(
    depth_limit: usize,
    complexity_limit: usize,
    database: QuasarDatabase,
) -> ServiceSchema {
    let mut schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data(database);

    if cfg!(debug_assertions) {
        info!("Debugging enabled, no limits on query");
    } else {
        schema = schema.limit_depth(depth_limit);
        schema = schema.limit_complexity(complexity_limit);
    }

    schema.finish()
}
