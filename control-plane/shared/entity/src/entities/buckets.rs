use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "buckets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub garage_bucket_id: Option<String>,
    pub global_alias: String,
    pub exposure: String,
    pub quota_max_size: Option<i64>,
    pub quota_max_objects: Option<i64>,
    pub status: String,
    pub master_key_id: Option<String>,
    pub master_key_secret_encrypted: Option<Vec<u8>>,
    pub organization_id: Option<Uuid>,
    pub resource_group_id: Option<Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::bucket_access_keys::Entity")]
    AccessKeys,
    #[sea_orm(
        belongs_to = "super::resource_groups::Entity",
        from = "Column::ResourceGroupId",
        to = "super::resource_groups::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    ResourceGroup,
}

impl Related<super::bucket_access_keys::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccessKeys.def()
    }
}

impl Related<super::resource_groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ResourceGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
