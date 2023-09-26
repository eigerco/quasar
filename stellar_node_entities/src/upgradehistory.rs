//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "upgradehistory")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub ledgerseq: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub upgradeindex: i32,
    #[sea_orm(column_type = "Text")]
    pub upgrade: String,
    #[sea_orm(column_type = "Text")]
    pub changes: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}