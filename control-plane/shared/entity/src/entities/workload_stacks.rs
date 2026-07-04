use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workload_stacks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub resource_group_id: Uuid,
    pub name: String,
    pub compose_source: Option<String>,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::resource_groups::Entity",
        from = "Column::ResourceGroupId",
        to = "super::resource_groups::Column::Id",
        on_delete = "Cascade"
    )]
    ResourceGroup,
    #[sea_orm(has_many = "super::workloads::Entity")]
    Workloads,
}

impl Related<super::resource_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResourceGroup.def()
    }
}

impl Related<super::workloads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Workloads.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
