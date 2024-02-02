use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "txhistory")]
pub struct Model {
    pub txid: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub ledgerseq: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub txindex: i32,
    #[sea_orm(column_type = "Text")]
    pub txbody: String,
    #[sea_orm(column_type = "Text")]
    pub txresult: String,
    #[sea_orm(column_type = "Text")]
    pub txmeta: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
