use async_graphql::InputObject;
use sea_orm::{EntityTrait, QuerySelect, Select};

#[derive(InputObject)]
pub(super) struct Pagination {
    pub page: u64,
    pub per_page: u64,
}

pub(super) fn apply_pagination<E: EntityTrait>(
    query: Select<E>,
    pagination: Option<Pagination>,
) -> Select<E> {
    if let Some(pagination) = pagination {
        query
            .offset((pagination.page - 1) * pagination.per_page)
            .limit(pagination.per_page)
    } else {
        query
    }
}
