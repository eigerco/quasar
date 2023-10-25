use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Result, Schema};
use log::info;
use quasar_entities::{account, contract, event, ledger, operation, transaction};
use sea_orm::EntityTrait;

use crate::databases::QuasarDatabase;

pub(crate) type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }

    async fn ledgers(&self, ctx: &Context<'_>) -> Result<Vec<ledger::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(ledger::Entity::find().all(database).await?)
    }
    async fn contracts(&self, ctx: &Context<'_>) -> Result<Vec<contract::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(contract::Entity::find().all(database).await?)
    }

    async fn accounts(&self, ctx: &Context<'_>) -> Result<Vec<account::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(account::Entity::find().all(database).await?)
    }

    async fn events(&self, ctx: &Context<'_>) -> Result<Vec<event::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(event::Entity::find().all(database).await?)
    }

    async fn transactions(&self, ctx: &Context<'_>) -> Result<Vec<transaction::Model>> {
        let database = ctx.data::<QuasarDatabase>()?.as_inner();
        Ok(transaction::Entity::find().all(database).await?)
    }

    async fn operations(&self, ctx: &Context<'_>) -> Result<Vec<operation::Model>> {
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
