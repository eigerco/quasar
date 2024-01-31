use std::collections::HashMap;

use stellar_xdr::curr::AccountEntry;
use thiserror::Error;

use async_graphql::{dataloader::Loader, ComplexObject, Context};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Condition, Set};
use std::sync::Arc;

use crate::{ledger, transaction, QuasarDataLoader};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, async_graphql::SimpleObject)]
#[sea_orm(table_name = "accounts")]
#[graphql(complex)]
#[graphql(name = "Accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub balance: i64,
    pub buying_liabilities: Option<i64>,
    pub selling_liabilities: Option<i64>,
    pub sequence_number: i64,
    pub number_of_subentries: i32,
    pub inflation_destination: Option<String>,
    pub home_domain: String,
    pub master_weight: i16,
    pub threshold_low: i16,
    pub threshold_medium: i16,
    pub threshold_high: i16,
    pub last_modified: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::transaction::Entity",
        to = "super::transaction::Column::AccountId",
        from = "Column::Id"
    )]
    Transaction,
    #[sea_orm(
        belongs_to = "super::ledger::Entity",
        to = "super::ledger::Column::Sequence",
        from = "Column::LastModified"
    )]
    Ledger,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl Related<super::ledger::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ledger.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[ComplexObject]
impl Model {
    pub async fn transactions<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> Result<Vec<transaction::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(transaction::Entity).all(database).await
    }

    pub async fn ledger<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Option<ledger::Model>, DbErr> {
        let database = ctx
            .data::<DatabaseConnection>()
            .expect("DatabaseConnection missing from GraphQL context");
        self.find_related(ledger::Entity).one(database).await
    }
}

#[derive(Error, Debug, Clone)]
pub enum AccountError {
    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("UTF-8 decoding error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid account thresholds")]
    InvalidThresholds,
    #[error("Base64 decoding error: {0}")]
    DatabaseError(#[from] Arc<sea_orm::DbErr>),
}

impl TryFrom<AccountEntry> for ActiveModel {
    type Error = AccountError;

    fn try_from(account: AccountEntry) -> Result<Self, Self::Error> {
        let home_domain = account.home_domain.to_string();

        let thresholds_base64_decoded = account.thresholds;

        let (master_weight, threshold_low, threshold_medium, threshold_high) = if let [master_weight, threshold_low, threshold_medium, threshold_high] =
            thresholds_base64_decoded.as_slice()
        {
            (
                master_weight,
                threshold_low,
                threshold_medium,
                threshold_high,
            )
        } else {
            return Err(AccountError::InvalidThresholds);
        };

        Ok(Self {
            id: Set(account.account_id.to_string()),
            balance: Set(account.balance),
            buying_liabilities: Set(None),
            selling_liabilities: Set(None),
            sequence_number: Set(account.seq_num.into()),
            number_of_subentries: Set(account.num_sub_entries.try_into().unwrap()),
            inflation_destination: Set(account.inflation_dest.map(|i| i.to_string())),
            home_domain: Set(home_domain),
            master_weight: Set(*master_weight as i16),
            threshold_low: Set(*threshold_low as i16),
            threshold_medium: Set(*threshold_medium as i16),
            threshold_high: Set(*threshold_high as i16),
            last_modified: Set(0), //Now
            created_at: NotSet,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountId(pub String);

#[async_trait::async_trait]
impl Loader<AccountId> for QuasarDataLoader {
    type Value = Model;
    type Error = AccountError;

    async fn load(
        &self,
        keys: &[AccountId],
    ) -> Result<HashMap<AccountId, Self::Value>, Self::Error> {
        let mut condition = Condition::any();

        for AccountId(id) in keys {
            condition = condition.add(Column::Id.eq(id.clone()));
        }
        let accounts = Entity::find()
            .filter(condition)
            .all(&self.pool)
            .await
            .map_err(Arc::new)?;
        Ok(accounts
            .into_iter()
            .map(|account| (AccountId(account.id.clone()), account))
            .collect())
    }
}
