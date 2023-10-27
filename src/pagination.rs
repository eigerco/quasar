use std::{fmt::Display, future::Future, ops::Deref};

use async_graphql::{
    connection::{self, Connection, ConnectionNameType, CursorType, EdgeNameType, EmptyFields},
    ErrorExtensions, InputObject, OutputType,
};
use chrono::{DateTime, FixedOffset};

#[derive(Default, InputObject)]
pub struct ConnectionParams {
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

pub async fn cursor_pagination<
    Cursor,
    Node,
    Query,
    Result,
    Edge,
    EdgeCursor,
    ConnectionName,
    EdgeName,
>(
    params: Option<ConnectionParams>,
    query: Query,
    edge_cursor: EdgeCursor,
) -> async_graphql::Result<
    Connection<Cursor, Node, EmptyFields, EmptyFields, ConnectionName, EdgeName>,
>
where
    Cursor: Clone + CursorType + Send + Sync,
    <Cursor as CursorType>::Error: Display + Send + Sync + 'static,
    Node: OutputType,
    Query: FnOnce(Option<Cursor>, Option<Cursor>, Option<u64>) -> Result,
    Result: Future<Output = std::result::Result<Vec<Node>, Edge>>,
    Edge: Into<Box<dyn std::error::Error>> + Display,
    EdgeCursor: Fn(&Node) -> Cursor,
    ConnectionName: ConnectionNameType,
    EdgeName: EdgeNameType,
{
    let params = params.unwrap_or_default();
    connection::query(
        params.after,
        params.before,
        params.first,
        params.last,
        |after, before, first, last| async move {
            if before.is_some() && last.is_some() {
                return Err::<_, async_graphql::Error>(async_graphql::Error::new(
                    "Cannot query 'before' with 'last'",
                ));
            }

            let nodes: Vec<Node> = query(after, before, first.map(|first| first as u64))
                .await
                .map_err(|err| (&err).extend())?;

            let mut connection = Connection::new(
                first.is_some(),
                first.map(|first| first <= nodes.len()).unwrap_or(false),
            );

            connection.edges.extend(nodes.into_iter().map(|node| {
                connection::Edge::with_additional_fields(edge_cursor(&node), node, EmptyFields)
            }));
            Ok::<_, async_graphql::Error>(connection)
        },
    )
    .await
}

#[derive(Debug, Copy, Clone)]
pub struct DateTimeCursor(DateTime<FixedOffset>);

impl Deref for DateTimeCursor {
    type Target = DateTime<FixedOffset>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DateTimeCursor> for DateTime<FixedOffset> {
    fn from(cursor: DateTimeCursor) -> Self {
        cursor.0
    }
}

impl From<DateTime<FixedOffset>> for DateTimeCursor {
    fn from(dt: DateTime<FixedOffset>) -> Self {
        Self(dt)
    }
}

impl CursorType for DateTimeCursor {
    type Error = chrono::format::ParseError;

    fn decode_cursor(s: &str) -> std::result::Result<Self, Self::Error> {
        DateTime::parse_from_rfc3339(s).map(Self)
    }

    fn encode_cursor(&self) -> String {
        self.0.to_rfc3339()
    }
}
