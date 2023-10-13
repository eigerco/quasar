use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub accountid: String,
    pub balance: i64,
    pub buyingliabilities: Option<i64>,
    pub sellingliabilities: Option<i64>,
    pub seqnum: i64,
    pub numsubentries: i32,
    pub inflationdest: Option<String>,
    pub homedomain: String,
    #[sea_orm(column_type = "Text")]
    pub thresholds: String,
    pub flags: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub signers: Option<String>,
    pub lastmodified: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub extension: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub ledgerext: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
