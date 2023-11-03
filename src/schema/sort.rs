use async_graphql::{Enum, OneofObject};
use sea_orm::Order;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub(super) enum SortOrder {
    Asc,
    Desc,
}

impl From<SortOrder> for Order {
    fn from(val: SortOrder) -> Self {
        match val {
            SortOrder::Asc => Self::Asc,
            SortOrder::Desc => Self::Desc,
        }
    }
}

#[derive(OneofObject)]
pub(super) enum LedgerSort {
    Sequence(SortOrder),
}

#[derive(OneofObject)]
pub(super) enum AccountSort {
    Id(SortOrder),
    Balance(SortOrder),
}

#[derive(OneofObject)]
pub(super) enum ContractSort {
    Address(SortOrder),
}

#[derive(OneofObject)]
pub(super) enum EventSort {
    Id(SortOrder),
    Type(SortOrder),
}

#[derive(OneofObject)]
pub(super) enum OperationSort {
    Id(SortOrder),
    Type(SortOrder),
}

#[derive(OneofObject)]
pub(super) enum TransactionSort {
    Id(SortOrder),
    LedgerSequence(SortOrder),
}
