use async_graphql::{
    connection::{Connection, DefaultConnectionName, DefaultEdgeName, EmptyFields},
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use log::info;
use quasar_entities::ledger;
use sea_orm::{
    prelude::DateTimeWithTimeZone, ColumnTrait, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder, QuerySelect,
};

use crate::{
    databases::QuasarDatabase,
    pagination::{cursor_pagination, ConnectionParams, DateTimeCursor},
};

use self::{filter::LedgerFilter, sort::LedgerSort};

pub(crate) type ServiceSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub type PaginationCursorConnection<
    Cursor,
    Node,
    ConnectionName = DefaultConnectionName,
    EdgeName = DefaultEdgeName,
> = Connection<Cursor, Node, EmptyFields, EmptyFields, ConnectionName, EdgeName>;

pub(crate) struct QueryRoot;

mod filter;
mod sort;

#[Object]
impl QueryRoot {
    async fn ledger_by_hash(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "ledger hash")] hash: String,
    ) -> Result<Option<ledger::Model>> {
        let database = ctx.data::<DatabaseConnection>()?;
        Ok(ledger::Entity::find()
            .filter(ledger::Column::Hash.eq(hash))
            .one(database)
            .await?)
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
        page: Option<ConnectionParams>,
    ) -> Result<PaginationCursorConnection<DateTimeCursor, ledger::Model>> {
        cursor_pagination(
            page,
            |after: Option<DateTimeCursor>, before, limit| {
                let database = ctx.data::<DatabaseConnection>().unwrap();
                let query = ledger::Entity::find().limit(limit.unwrap_or(10));
                let query = filter.map_or(query.clone(), |filter| filter.apply(query));

                let (sort_column, sort_order) = match sort {
                    Some(LedgerSort::Sequence(order)) => (ledger::Column::Sequence, order.into()),
                    None => (ledger::Column::CreatedAt, Order::Desc),
                };

                let mut query = query.order_by(sort_column, sort_order);

                if let Some(after) = after {
                    query = query.filter(sort_column.lt::<DateTimeWithTimeZone>(after.into()));
                }
                if let Some(before) = before {
                    query = query.filter(sort_column.gt::<DateTimeWithTimeZone>(before.into()));
                }
                query.all(database)
            },
            |node: &ledger::Model| node.created_at.into(),
        )
        .await
    }

    /*
        async fn ledgers(
            &self,
            ctx: &Context<'_>,
            sort: Option<LedgerSort>,
            filter: Option<LedgerFilter>,
        ) -> Result<Vec<ledger::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = ledger::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(LedgerSort::Sequence(SortOrder::Asc)) => {
                    query.order_by_asc(ledger::Column::Sequence)
                }
                Some(LedgerSort::Sequence(SortOrder::Desc)) => {
                    query.order_by_desc(ledger::Column::Sequence)
                }
                None => {
                    query.order_by(ledger::Column::CreatedAt, Order::Desc)
                },
            };

            Ok(query.all(database).await?)
        }



        async fn contracts(
            &self,
            ctx: &Context<'_>,
            page: Option<ConnectionParams>,
        ) -> Result<PaginationCursorConnection<DateTimeCursor, contract::Model>> {
            cursor_pagination(
                page,
                |after: Option<DateTimeCursor>, before, limit| {
                    let database = ctx.data::<DatabaseConnection>().unwrap();
                    let mut query = contract::Entity::find()
                        .limit(limit.unwrap_or(10))
                        .order_by(contract::Column::CreatedAt, Order::Desc);
                    if let Some(after) = after {
                        query = query.filter(
                            contract::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()),
                        );
                    }
                    if let Some(before) = before {
                        query = query.filter(
                            contract::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()),
                        );
                    }
                    query.all(database)
                },
                |node: &contract::Model| node.created_at.into(),
            )
            .await
        }

        async fn accounts(
            &self,
            ctx: &Context<'_>,
            page: Option<ConnectionParams>,
        ) -> Result<PaginationCursorConnection<DateTimeCursor, account::Model>> {
            cursor_pagination(
                page,
                |after: Option<DateTimeCursor>, before, limit| {
                    let database = ctx.data::<DatabaseConnection>().unwrap();
                    let mut query = account::Entity::find()
                        .limit(limit.unwrap_or(10))
                        .order_by(account::Column::CreatedAt, Order::Desc);
                    if let Some(after) = after {
                        query = query.filter(
                            account::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()),
                        );
                    }
                    if let Some(before) = before {
                        query = query.filter(
                            account::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()),
                        );
                    }
                    query.all(database)
                },
                |node: &account::Model| node.created_at.into(),
            )
            .await
        }

        async fn events(
            &self,
            ctx: &Context<'_>,
            page: Option<ConnectionParams>,
        ) -> Result<PaginationCursorConnection<DateTimeCursor, event::Model>> {
            cursor_pagination(
                page,
                |after: Option<DateTimeCursor>, before, limit| {
                    let database = ctx.data::<DatabaseConnection>().unwrap();
                    let mut query = event::Entity::find()
                        .limit(limit.unwrap_or(10))
                        .order_by(event::Column::CreatedAt, Order::Desc);
                    if let Some(after) = after {
                        query = query
                            .filter(event::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()));
                    }
                    if let Some(before) = before {
                        query = query
                            .filter(event::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()));
                    }
                    query.all(database)
                },
                |node: &event::Model| node.created_at.into(),
            )
            .await
        }

        async fn transactions(
            &self,
            ctx: &Context<'_>,
            page: Option<ConnectionParams>,
        ) -> Result<PaginationCursorConnection<DateTimeCursor, transaction::Model>> {
            cursor_pagination(
                page,
                |after: Option<DateTimeCursor>, before, limit| {
                    let database = ctx.data::<DatabaseConnection>().unwrap();
                    let mut query = transaction::Entity::find()
                        .limit(limit.unwrap_or(10))
                        .order_by(transaction::Column::CreatedAt, Order::Desc);
                    if let Some(after) = after {
                        query = query.filter(
                            transaction::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()),
                        );
                    }
                    if let Some(before) = before {
                        query = query.filter(
                            transaction::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()),
                        );
                    }
                    query.all(database)
                },
                |node: &transaction::Model| node.created_at.into(),
            )
            .await
        }

        async fn operations(
            &self,
            ctx: &Context<'_>,
            page: Option<ConnectionParams>,
        ) -> Result<PaginationCursorConnection<DateTimeCursor, operation::Model>> {
            cursor_pagination(
                page,
                |after: Option<DateTimeCursor>, before, limit| {
                    let database = ctx.data::<DatabaseConnection>().unwrap();
                    let mut query = operation::Entity::find()
                        .limit(limit.unwrap_or(10))
                        .order_by(operation::Column::CreatedAt, Order::Desc);
                    if let Some(after) = after {
                        query = query.filter(
                            operation::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()),
                        );
                    }
                    if let Some(before) = before {
                        query = query.filter(
                            operation::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()),
                        );
                    }
                    query.all(database)
                },
                |node: &operation::Model| node.created_at.into(),
            )
            .await
    =======

        async fn contract(
            &self,
            ctx: &Context<'_>,
            #[graphql(desc = "contract address")] address: String,
        ) -> Result<Option<contract::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            Ok(contract::Entity::find()
                .filter(contract::Column::Address.eq(address))
                .one(database)
                .await?)
        }

        async fn contracts(
            &self,
            ctx: &Context<'_>,
            sort: Option<ContractSort>,
            filter: Option<ContractFilter>,
        ) -> Result<Vec<contract::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = contract::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(ContractSort::Address(SortOrder::Asc)) => {
                    query.order_by_asc(contract::Column::Address)
                }
                Some(ContractSort::Address(SortOrder::Desc)) => {
                    query.order_by_desc(contract::Column::Address)
                }
                None => query,
            };

            Ok(query.all(database).await?)
        }

        async fn account(
            &self,
            ctx: &Context<'_>,
            #[graphql(desc = "account id")] id: String,
        ) -> Result<Option<account::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            Ok(account::Entity::find()
                .filter(Column::Id.eq(id))
                .one(database)
                .await?)
        }

        async fn accounts(
            &self,
            ctx: &Context<'_>,
            sort: Option<AccountSort>,
            filter: Option<AccountFilter>,
        ) -> Result<Vec<account::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = account::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(AccountSort::Id(SortOrder::Asc)) => query.order_by_asc(Column::Id),
                Some(AccountSort::Id(SortOrder::Desc)) => query.order_by_desc(Column::Id),
                Some(AccountSort::Balance(SortOrder::Asc)) => query.order_by_asc(Column::Balance),
                Some(AccountSort::Balance(SortOrder::Desc)) => query.order_by_desc(Column::Balance),
                None => query,
            };

            Ok(query.all(database).await?)
        }

        async fn event(
            &self,
            ctx: &Context<'_>,
            #[graphql(desc = "event id")] id: i32,
        ) -> Result<Option<event::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            Ok(event::Entity::find()
                .filter(event::Column::Id.eq(id))
                .one(database)
                .await?)
        }

        async fn events(
            &self,
            ctx: &Context<'_>,
            sort: Option<EventSort>,
            filter: Option<EventFilter>,
        ) -> Result<Vec<event::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = event::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(EventSort::Id(SortOrder::Asc)) => query.order_by_asc(event::Column::Id),
                Some(EventSort::Id(SortOrder::Desc)) => query.order_by_desc(event::Column::Id),
                Some(EventSort::Type(SortOrder::Asc)) => query.order_by_asc(event::Column::Type),
                Some(EventSort::Type(SortOrder::Desc)) => query.order_by_desc(event::Column::Type),
                None => query,
            };

            Ok(query.all(database).await?)
        }

        async fn transaction(
            &self,
            ctx: &Context<'_>,
            #[graphql(desc = "transaction id")] id: String,
        ) -> Result<Option<transaction::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            Ok(transaction::Entity::find()
                .filter(transaction::Column::Id.eq(id))
                .one(database)
                .await?)
        }

        async fn transactions(
            &self,
            ctx: &Context<'_>,
            sort: Option<TransactionSort>,
            filter: Option<TransactionFilter>,
        ) -> Result<Vec<transaction::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = transaction::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(TransactionSort::Id(SortOrder::Asc)) => {
                    query.order_by_asc(transaction::Column::Id)
                }
                Some(TransactionSort::Id(SortOrder::Desc)) => {
                    query.order_by_desc(transaction::Column::Id)
                }
                Some(TransactionSort::LedgerSequence(SortOrder::Asc)) => {
                    query.order_by_asc(transaction::Column::LedgerSequence)
                }
                Some(TransactionSort::LedgerSequence(SortOrder::Desc)) => {
                    query.order_by_desc(transaction::Column::LedgerSequence)
                }
                None => query,
            };

            Ok(query.all(database).await?)
        }

        async fn operation(
            &self,
            ctx: &Context<'_>,
            #[graphql(desc = "operation id")] id: i32,
        ) -> Result<Option<operation::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            Ok(operation::Entity::find()
                .filter(operation::Column::Id.eq(id))
                .one(database)
                .await?)
        }

        async fn operations(
            &self,
            ctx: &Context<'_>,
            sort: Option<OperationSort>,
            filter: Option<OperationFilter>,
        ) -> Result<Vec<operation::Model>> {
            let database = ctx.data::<DatabaseConnection>()?;
            let query = operation::Entity::find();

            let query = filter.map_or(query.clone(), |filter| filter.apply(query));

            let query = match sort {
                Some(OperationSort::Id(SortOrder::Asc)) => query.order_by_asc(operation::Column::Id),
                Some(OperationSort::Id(SortOrder::Desc)) => query.order_by_desc(operation::Column::Id),
                Some(OperationSort::Type(SortOrder::Asc)) => {
                    query.order_by_asc(operation::Column::Type)
                }
                Some(OperationSort::Type(SortOrder::Desc)) => {
                    query.order_by_desc(operation::Column::Type)
                }
                None => query,
            };

            Ok(query.all(database).await?)
    >>>>>>> 0650db9 (More queries, filters and sorting)
        }
        */
}

pub(super) fn build_schema(
    depth_limit: usize,
    complexity_limit: usize,
    database: QuasarDatabase,
) -> ServiceSchema {
    let database = database.as_inner().clone();
    let mut schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data(database);

    if cfg!(debug_assertions) {
        info!("Debugging enabled, no limits on query");
    } else {
        schema = schema.limit_depth(depth_limit);
        schema = schema.limit_complexity(complexity_limit);
    }

    schema.finish()
}
