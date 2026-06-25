use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "networks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub cidr: String,
    pub overlay_type: String,
    pub status: String,
    pub organization_id: Option<Uuid>,
    pub resource_group_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::network_policies::Entity")]
    Policies,
    #[sea_orm(has_many = "super::network_members::Entity")]
    Members,
    #[sea_orm(
        belongs_to = "super::resource_groups::Entity",
        from = "Column::ResourceGroupId",
        to = "super::resource_groups::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ResourceGroup,
}

impl Related<super::network_policies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Policies.def()
    }
}

impl Related<super::network_members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl Related<super::resource_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResourceGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
