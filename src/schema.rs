use async_graphql::{
    connection::{Connection, DefaultConnectionName, DefaultEdgeName, EmptyFields},
    Context, EmptyMutation, EmptySubscription, Object, Result, Schema,
};
use log::info;
use quasar_entities::{account, contract, event, ledger, operation, transaction};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ColumnTrait, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder, QuerySelect,
};

use crate::{
    databases::QuasarDatabase,
    pagination::{cursor_pagination, ConnectionParams, DateTimeCursor},
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
    async fn ledgers(
        &self,
        ctx: &Context<'_>,
        page: Option<ConnectionParams>,
    ) -> Result<PaginationCursorConnection<DateTimeCursor, ledger::Model>> {
        cursor_pagination(
            page,
            |after: Option<DateTimeCursor>, before, limit| {
                let database = ctx.data::<DatabaseConnection>().unwrap();
                let mut query = ledger::Entity::find()
                    .limit(limit.unwrap_or(10))
                    .order_by(ledger::Column::CreatedAt, Order::Desc);
                if let Some(after) = after {
                    query = query
                        .filter(ledger::Column::CreatedAt.lt::<DateTimeWithTimeZone>(after.into()));
                }
                if let Some(before) = before {
                    query = query.filter(
                        ledger::Column::CreatedAt.gt::<DateTimeWithTimeZone>(before.into()),
                    );
                }
                query.all(database)
            },
            |node: &ledger::Model| node.created_at.into(),
        )
        .await
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
    }
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
