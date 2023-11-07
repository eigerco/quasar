use async_graphql::{
    dataloader::{DataLoader, Loader},
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use log::info;
use quasar_entities::account::AccountId;
use quasar_entities::contract::ContractId;
use quasar_entities::event::EventId;
use quasar_entities::operation::OperationId;
use quasar_entities::transaction::TransactionId;
use quasar_entities::{
    account::{self},
    contract, event,
    ledger::{self, LedgerHash},
    operation, transaction, QuasarDataLoader,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder};

use crate::databases::QuasarDatabase;

use self::{
    filter::{
        AccountFilter, ContractFilter, EventFilter, LedgerFilter, OperationFilter,
        TransactionFilter,
    },
    pagination::{apply_pagination, Pagination},
    sort::{AccountSort, ContractSort, EventSort, LedgerSort, OperationSort, TransactionSort},
};

pub(crate) type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub(crate) struct QueryRoot;

mod filter;
mod pagination;
mod sort;

#[Object]
impl QueryRoot {
    async fn ledger_by_hash(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "ledger hash")] hash: String,
    ) -> Result<Option<ledger::Model>> {
        let id = LedgerHash(hash);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn ledger_by_sequence(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "ledger sequence")] sequence: i32,
    ) -> Result<Option<ledger::Model>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(ledger::Entity::find()
            .filter(ledger::Column::Sequence.eq(sequence))
            .one(database)
            .await?)
    }

    async fn ledgers(
        &self,
        ctx: &Context<'_>,
        filter: Option<LedgerFilter>,
        sort: Option<LedgerSort>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<ledger::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = ledger::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(LedgerSort::Sequence(order)) => (ledger::Column::Sequence, order.into()),
            None => (ledger::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }

    async fn contract(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "contract address")] address: String,
    ) -> Result<Option<contract::Model>> {
        let id = ContractId(address);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn contracts(
        &self,
        ctx: &Context<'_>,
        sort: Option<ContractSort>,
        filter: Option<ContractFilter>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<contract::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = contract::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(ContractSort::Address(order)) => (contract::Column::Address, order.into()),
            None => (contract::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }

    async fn account(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "account id")] id: String,
    ) -> Result<Option<account::Model>> {
        let id = AccountId(id);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn accounts(
        &self,
        ctx: &Context<'_>,
        sort: Option<AccountSort>,
        filter: Option<AccountFilter>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<account::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = account::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(AccountSort::Id(order)) => (account::Column::Id, order.into()),
            Some(AccountSort::Balance(order)) => (account::Column::Balance, order.into()),
            None => (account::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }

    async fn event(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "event id")] id: i32,
    ) -> Result<Option<event::Model>> {
        let id = EventId(id);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn events(
        &self,
        ctx: &Context<'_>,
        sort: Option<EventSort>,
        filter: Option<EventFilter>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<event::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = event::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(EventSort::Id(order)) => (event::Column::Id, order.into()),
            Some(EventSort::Type(order)) => (event::Column::Type, order.into()),
            None => (event::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }

    async fn transaction(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "transaction id")] id: String,
    ) -> Result<Option<transaction::Model>> {
        let id = TransactionId(id);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
        sort: Option<TransactionSort>,
        filter: Option<TransactionFilter>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<transaction::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = transaction::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(TransactionSort::Id(order)) => (transaction::Column::Id, order.into()),
            Some(TransactionSort::LedgerSequence(order)) => {
                (transaction::Column::LedgerSequence, order.into())
            }
            None => (transaction::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }

    async fn operation(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "operation id")] id: i32,
    ) -> Result<Option<operation::Model>> {
        let id = OperationId(id);
        let loader = ctx.data::<QuasarDataLoader>()?;
        let res = loader.load(&[id.clone()]).await?;
        Ok(res.get(&id).cloned())
    }

    async fn operations(
        &self,
        ctx: &Context<'_>,
        sort: Option<OperationSort>,
        filter: Option<OperationFilter>,
        pagination: Option<Pagination>,
    ) -> Result<Vec<operation::Model>> {
        let database = ctx.data::<DatabaseConnection>().unwrap();
        let query = operation::Entity::find();
        let query = filter.map_or(query.clone(), |filter| filter.apply(query));

        let (sort_column, sort_order) = match sort {
            Some(OperationSort::Id(order)) => (operation::Column::Id, order.into()),
            Some(OperationSort::Type(order)) => (operation::Column::Type, order.into()),
            None => (operation::Column::CreatedAt, Order::Desc),
        };

        let mut query = query.order_by(sort_column, sort_order);

        query = apply_pagination(query, pagination);

        Ok(query.all(database).await?)
    }
}

pub(super) fn build_schema(
    depth_limit: usize,
    complexity_limit: usize,
    database: QuasarDatabase,
) -> ServiceSchema {
    let database = database.as_inner().clone();
    let mut schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(DataLoader::new(
            QuasarDataLoader::new(database.clone()),
            tokio::spawn,
        ))
        .data(database);

    if cfg!(debug_assertions) {
        info!("Debugging enabled, no limits on query");
    } else {
        schema = schema.limit_depth(depth_limit);
        schema = schema.limit_complexity(complexity_limit);
    }

    schema.finish()
}
