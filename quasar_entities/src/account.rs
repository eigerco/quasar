use thiserror::Error;

use async_graphql::{ComplexObject, Context};
use base64::{engine::general_purpose, Engine};
use sea_orm::{entity::prelude::*, Set, ActiveValue::NotSet};

use stellar_node_entities::accounts;

use crate::{ledger, transaction};

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

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("Base64 decoding error: {0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("UTF-8 decoding error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid account thresholds")]
    InvalidThresholds,
}

impl TryFrom<accounts::Model> for ActiveModel {
    type Error = AccountError;

    fn try_from(accounts: accounts::Model) -> Result<Self, Self::Error> {
        let home_domain =
            String::from_utf8(general_purpose::STANDARD.decode(accounts.homedomain)?)?;

        let thresholds_base64_decoded = general_purpose::STANDARD.decode(accounts.thresholds)?;

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
            id: Set(accounts.accountid),
            balance: Set(accounts.balance),
            buying_liabilities: Set(accounts.buyingliabilities),
            selling_liabilities: Set(accounts.sellingliabilities),
            sequence_number: Set(accounts.seqnum),
            number_of_subentries: Set(accounts.numsubentries),
            inflation_destination: Set(accounts.inflationdest),
            home_domain: Set(home_domain),
            master_weight: Set(*master_weight as i16),
            threshold_low: Set(*threshold_low as i16),
            threshold_medium: Set(*threshold_medium as i16),
            threshold_high: Set(*threshold_high as i16),
            last_modified: Set(accounts.lastmodified),
            created_at: NotSet

        })
    }
}
