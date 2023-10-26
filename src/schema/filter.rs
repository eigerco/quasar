use async_graphql::{Enum, InputObject};
use quasar_entities::{account, contract, event, ledger, operation, prelude::*, transaction};
use sea_orm::{ColumnTrait, QueryFilter, Select};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub(super) enum Operator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
}

#[derive(InputObject)]
pub(super) struct I32Filter {
    pub(super) op: Operator,
    pub(super) value: i32,
}

#[derive(InputObject)]
pub(super) struct I64Filter {
    pub(super) op: Operator,
    pub(super) value: i64,
}

#[derive(InputObject)]
pub(super) struct LedgerFilter {
    pub(super) hash: Option<String>,
    pub(super) sequence: Option<I32Filter>,
}

impl LedgerFilter {
    pub(super) fn apply(&self, query: Select<Ledger>) -> Select<Ledger> {
        let mut query = query;

        if let Some(hash) = &self.hash {
            query = query.filter(ledger::Column::Hash.eq(hash));
        }

        if let Some(sequence) = &self.sequence {
            match sequence.op {
                Operator::GreaterThan => {
                    query = query.filter(ledger::Column::Sequence.gt(sequence.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query.filter(ledger::Column::Sequence.gte(sequence.value));
                }
                Operator::LessThan => {
                    query = query.filter(ledger::Column::Sequence.lt(sequence.value));
                }
                Operator::LessThanOrEqual => {
                    query = query.filter(ledger::Column::Sequence.lte(sequence.value));
                }
                Operator::Equal => {
                    query = query.filter(ledger::Column::Sequence.eq(sequence.value));
                }
            }
        }

        query
    }
}

#[derive(InputObject)]
pub(super) struct AccountFilter {
    pub(super) balance: Option<I64Filter>,
    pub(super) buying_liabilities: Option<I64Filter>,
    pub(super) selling_liabilities: Option<I64Filter>,
    pub(super) sequence_number: Option<I64Filter>,
}

impl AccountFilter {
    pub(super) fn apply(&self, query: Select<Account>) -> Select<Account> {
        let mut query = query;

        if let Some(balance) = &self.balance {
            match balance.op {
                Operator::GreaterThan => {
                    query = query.filter(account::Column::Balance.gt(balance.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query.filter(account::Column::Balance.gte(balance.value));
                }
                Operator::LessThan => {
                    query = query.filter(account::Column::Balance.lt(balance.value));
                }
                Operator::LessThanOrEqual => {
                    query = query.filter(account::Column::Balance.lte(balance.value));
                }
                Operator::Equal => {
                    query = query.filter(account::Column::Balance.eq(balance.value));
                }
            }
        }

        if let Some(buying_liabilities) = &self.buying_liabilities {
            match buying_liabilities.op {
                Operator::GreaterThan => {
                    query = query
                        .filter(account::Column::BuyingLiabilities.gt(buying_liabilities.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(account::Column::BuyingLiabilities.gte(buying_liabilities.value));
                }
                Operator::LessThan => {
                    query = query
                        .filter(account::Column::BuyingLiabilities.lt(buying_liabilities.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(account::Column::BuyingLiabilities.lte(buying_liabilities.value));
                }
                Operator::Equal => {
                    query = query
                        .filter(account::Column::BuyingLiabilities.eq(buying_liabilities.value));
                }
            }
        }

        if let Some(selling_liabilities) = &self.selling_liabilities {
            match selling_liabilities.op {
                Operator::GreaterThan => {
                    query = query
                        .filter(account::Column::SellingLiabilities.gt(selling_liabilities.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(account::Column::SellingLiabilities.gte(selling_liabilities.value));
                }
                Operator::LessThan => {
                    query = query
                        .filter(account::Column::SellingLiabilities.lt(selling_liabilities.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(account::Column::SellingLiabilities.lte(selling_liabilities.value));
                }
                Operator::Equal => {
                    query = query
                        .filter(account::Column::SellingLiabilities.eq(selling_liabilities.value));
                }
            }
        }

        if let Some(sequence_number) = &self.sequence_number {
            match sequence_number.op {
                Operator::GreaterThan => {
                    query = query.filter(account::Column::SequenceNumber.gt(sequence_number.value));
                }
                Operator::GreaterThanOrEqual => {
                    query =
                        query.filter(account::Column::SequenceNumber.gte(sequence_number.value));
                }
                Operator::LessThan => {
                    query = query.filter(account::Column::SequenceNumber.lt(sequence_number.value));
                }
                Operator::LessThanOrEqual => {
                    query =
                        query.filter(account::Column::SequenceNumber.lte(sequence_number.value));
                }
                Operator::Equal => {
                    query = query.filter(account::Column::SequenceNumber.eq(sequence_number.value));
                }
            }
        }

        query
    }
}

#[derive(InputObject)]
pub(super) struct ContractFilter {
    pub(super) address: Option<String>,
    pub(super) r#type: Option<String>,
    pub(super) last_modified: Option<I32Filter>,
}

impl ContractFilter {
    pub(super) fn apply(&self, query: Select<Contract>) -> Select<Contract> {
        let mut query = query;

        if let Some(address) = &self.address {
            query = query.filter(contract::Column::Address.eq(address));
        }

        if let Some(r#type) = &self.r#type {
            query = query.filter(contract::Column::Type.eq(r#type));
        }

        if let Some(last_modified) = &self.last_modified {
            match last_modified.op {
                Operator::GreaterThan => {
                    query = query.filter(contract::Column::LastModified.gt(last_modified.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query.filter(contract::Column::LastModified.gte(last_modified.value));
                }
                Operator::LessThan => {
                    query = query.filter(contract::Column::LastModified.lt(last_modified.value));
                }
                Operator::LessThanOrEqual => {
                    query = query.filter(contract::Column::LastModified.lte(last_modified.value));
                }
                Operator::Equal => {
                    query = query.filter(contract::Column::LastModified.eq(last_modified.value));
                }
            }
        }

        query
    }
}

#[derive(InputObject)]
pub(super) struct EventFilter {
    pub(super) topic: Option<String>,
    pub(super) r#type: Option<String>,
}

impl EventFilter {
    pub(super) fn apply(&self, query: Select<Event>) -> Select<Event> {
        let mut query = query;

        if let Some(topic) = &self.topic {
            query = query.filter(event::Column::Topic.eq(topic));
        }

        if let Some(r#type) = &self.r#type {
            query = query.filter(event::Column::Type.eq(r#type));
        }

        query
    }
}

#[derive(InputObject)]
pub(super) struct OperationFilter {
    pub(super) r#type: Option<String>,
    pub(super) application_order: Option<I32Filter>,
}

impl OperationFilter {
    pub(super) fn apply(&self, query: Select<Operation>) -> Select<Operation> {
        let mut query = query;

        if let Some(r#type) = &self.r#type {
            query = query.filter(operation::Column::Type.eq(r#type));
        }

        if let Some(application_order) = &self.application_order {
            match application_order.op {
                Operator::GreaterThan => {
                    query = query
                        .filter(operation::Column::ApplicationOrder.gt(application_order.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(operation::Column::ApplicationOrder.gte(application_order.value));
                }
                Operator::LessThan => {
                    query = query
                        .filter(operation::Column::ApplicationOrder.lt(application_order.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(operation::Column::ApplicationOrder.lte(application_order.value));
                }
                Operator::Equal => {
                    query = query
                        .filter(operation::Column::ApplicationOrder.eq(application_order.value));
                }
            }
        }

        query
    }
}

#[derive(InputObject)]
pub(super) struct TransactionFilter {
    pub(super) ledger_sequence: Option<I32Filter>,
    pub(super) application_order: Option<I32Filter>,
    pub(super) account_sequence: Option<I64Filter>,
    pub(super) operation_count: Option<I32Filter>,
}

impl TransactionFilter {
    pub(super) fn apply(&self, query: Select<Transaction>) -> Select<Transaction> {
        let mut query = query;

        if let Some(ledger_sequence) = &self.ledger_sequence {
            match ledger_sequence.op {
                Operator::GreaterThan => {
                    query =
                        query.filter(transaction::Column::LedgerSequence.gt(ledger_sequence.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(transaction::Column::LedgerSequence.gte(ledger_sequence.value));
                }
                Operator::LessThan => {
                    query =
                        query.filter(transaction::Column::LedgerSequence.lt(ledger_sequence.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(transaction::Column::LedgerSequence.lte(ledger_sequence.value));
                }
                Operator::Equal => {
                    query =
                        query.filter(transaction::Column::LedgerSequence.eq(ledger_sequence.value));
                }
            }
        }

        if let Some(application_order) = &self.application_order {
            match application_order.op {
                Operator::GreaterThan => {
                    query = query
                        .filter(transaction::Column::ApplicationOrder.gt(application_order.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(transaction::Column::ApplicationOrder.gte(application_order.value));
                }
                Operator::LessThan => {
                    query = query
                        .filter(transaction::Column::ApplicationOrder.lt(application_order.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(transaction::Column::ApplicationOrder.lte(application_order.value));
                }
                Operator::Equal => {
                    query = query
                        .filter(transaction::Column::ApplicationOrder.eq(application_order.value));
                }
            }
        }

        if let Some(account_sequence) = &self.account_sequence {
            match account_sequence.op {
                Operator::GreaterThan => {
                    query = query
                        .filter(transaction::Column::AccountSequence.gt(account_sequence.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(transaction::Column::AccountSequence.gte(account_sequence.value));
                }
                Operator::LessThan => {
                    query = query
                        .filter(transaction::Column::AccountSequence.lt(account_sequence.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(transaction::Column::AccountSequence.lte(account_sequence.value));
                }
                Operator::Equal => {
                    query = query
                        .filter(transaction::Column::AccountSequence.eq(account_sequence.value));
                }
            }
        }

        if let Some(operation_count) = &self.operation_count {
            match operation_count.op {
                Operator::GreaterThan => {
                    query =
                        query.filter(transaction::Column::OperationCount.gt(operation_count.value));
                }
                Operator::GreaterThanOrEqual => {
                    query = query
                        .filter(transaction::Column::OperationCount.gte(operation_count.value));
                }
                Operator::LessThan => {
                    query =
                        query.filter(transaction::Column::OperationCount.lt(operation_count.value));
                }
                Operator::LessThanOrEqual => {
                    query = query
                        .filter(transaction::Column::OperationCount.lte(operation_count.value));
                }
                Operator::Equal => {
                    query =
                        query.filter(transaction::Column::OperationCount.eq(operation_count.value));
                }
            }
        }

        query
    }
}
